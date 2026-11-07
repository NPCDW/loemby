use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use serde_json::json;
use tauri_plugin_mpv::{MpvExt, PrepareOptions};

use super::*;
use crate::service::player_options::CACHE_SPEED_SCRIPT;

pub(super) async fn call_player(body: PlayVideoParam, state: &tauri::State<'_, AppState>, app: AppHandle, server: &EmbyServer, mut options: PlayerOptions) -> Result<()> {
    let players = app.state::<Players>();
    let _launch = players.launching.try_lock().map_err(|_| anyhow::anyhow!("播放器正在准备，请勿重复点击"))?;
    anyhow::ensure!(players.sessions.read().await.is_empty(), "已有播放会话，请先关闭播放器");
    let id = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
    let directory = app.path().app_data_dir()?.join("mpv").join(&id);
    let socket = directory.join("ipc.sock");
    let socket_path = socket.to_str().context("socket 路径不是 UTF-8")?.to_string();
    anyhow::ensure!(socket_path.len() < 104, "Android mpv socket 路径过长");
    tokio::fs::create_dir_all(&directory).await?;
    let plan = prepare_playback_plan(&body, state, server, &socket_path).await?;
    let playlist_path = directory.join("playlist.m3u8");
    let config_path = directory.join("loemby.mpv.conf");
    let script_path = directory.join("cache_speed.lua");
    let session = Arc::new(Session::new(id.clone(), directory, plan.ids));
    players.sessions.write().await.insert(socket_path.clone(), session.clone());
    let setup = async {
        tokio::fs::write(&playlist_path, plan.m3u).await?;
        tokio::fs::write(&config_path, options.config).await?;
        tokio::fs::write(&script_path, CACHE_SPEED_SCRIPT).await?;
        options.options.push(("script".to_string(), script_path.to_str().context("脚本路径不是 UTF-8")?.to_string()));
        tokio::time::timeout(Duration::from_secs(30), app.mpv().prepare(&id, &socket, PrepareOptions {
            title: Some("Loemby".to_string()), start_paused: true,
            config_path: Some(config_path), mpv_options: options.options,
        })).await.context("Android 播放器准备超时")??;
        let (client, events) = mpv_ipc::connect(&socket_path).await?;
        client.request(json!(["observe_property", 10001, "volume"])).await?;
        playback_session::start_process(app.clone(), socket_path.clone(), session.clone(), client.clone(), events);
        client.request(json!(["loadlist", playlist_path.to_str().context("播放列表路径不是 UTF-8")?, "replace", plan.start])).await?;
        Ok::<(), anyhow::Error>(())
    }.await;
    if let Err(error) = setup {
        playback_session::cleanup(&app, &socket_path, &session).await;
        return Err(error);
    }
    Ok(())
}

pub(super) async fn close(app: &AppHandle, socket: &str, session: &Session) -> bool {
    let closed = tokio::time::timeout(Duration::from_secs(10), app.mpv().close(&session.id)).await;
    if matches!(closed, Ok(Ok(()))) { return true; }
    tracing::error!("关闭 Android mpv 尚未完成，保留会话隔离并后台等待");
    let app = app.clone();
    let socket = socket.to_string();
    let id = session.id.clone();
    tauri::async_runtime::spawn(async move {
        if app.mpv().close(&id).await.is_ok() {
            playback_session::finish_cleanup(&app, &socket, &id).await;
        }
    });
    false
}
