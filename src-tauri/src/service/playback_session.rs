use std::{collections::{HashMap, HashSet}, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};

use super::*;
use crate::service::mpv_ipc::Client;

#[derive(Default)]
pub(super) struct Players {
    #[cfg(target_os = "android")]
    pub launching: Mutex<()>,
    pub sessions: RwLock<HashMap<String, Arc<Session>>>,
}

pub(super) struct Session {
    pub id: String,
    #[cfg(target_os = "android")]
    pub directory: PathBuf,
    pub ids: Vec<String>,
    media: Mutex<HashMap<(String, usize), PlaybackProcessParam>>,
    requests: Mutex<HashSet<String>>,
    #[cfg(desktop)]
    pub child: Mutex<Option<tokio::process::Child>>,
}

impl Session {
    pub fn new(id: String, directory: PathBuf, ids: Vec<String>) -> Self {
        #[cfg(not(target_os = "android"))]
        let _ = directory;
        Self {
            id,
            #[cfg(target_os = "android")]
            directory,
            ids,
            media: Mutex::new(HashMap::new()),
            requests: Mutex::new(HashSet::new()),
            #[cfg(desktop)]
            child: Mutex::new(None),
        }
    }
}

pub(super) async fn register_media(param: PlaybackProcessParam, request_id: String, video_url: String) -> Result<()> {
    let app = param.app_handle.clone();
    let players = app.state::<Players>();
    let sessions = players.sessions.read().await;
    let Some(session) = sessions.get(&param.params.mpv_ipc) else {
        param.axum_app_state.request.write().await.remove(&request_id);
        anyhow::bail!("播放会话已结束");
    };
    anyhow::ensure!(session.ids.contains(&param.id), "媒体不属于当前播放会话");
    session.requests.lock().await.insert(request_id);
    param.axum_app_state.play_media_redirect_cache.write().await.insert(format!("{}-{}", param.id, param.media_source_select), video_url);
    session.media.lock().await.insert((param.id.clone(), param.media_source_select), param);
    Ok(())
}

pub(super) async fn register_request(app: &AppHandle, socket: &str, request_id: &str) -> Result<()> {
    let players = app.state::<Players>();
    let sessions = players.sessions.read().await;
    if let Some(session) = sessions.get(socket) {
        session.requests.lock().await.insert(request_id.to_string());
        return Ok(());
    }
    if let Some(axum) = app.state::<AppState>().auxm_app_state.read().await.clone() {
        axum.request.write().await.remove(request_id);
    }
    anyhow::bail!("播放会话已结束")
}

struct CurrentMedia {
    param: PlaybackProcessParam,
    position: Decimal,
    entry_id: Option<u64>,
    saved_at: chrono::DateTime<chrono::Local>,
}

fn media_key(filename: &str, port: u16) -> Option<(String, usize)> {
    let url = url::Url::parse(filename).ok()?;
    if url.scheme() != "http" || url.host_str() != Some("127.0.0.1") || url.port() != Some(port) { return None; }
    let parts = url.path_segments()?.collect::<Vec<_>>();
    if parts.len() != 3 || parts[0] != "play_media" { return None; }
    Some((parts[1].to_string(), parts[2].parse().ok()?))
}

fn end_status(reason: &str, index: usize, total: usize) -> Option<(PlayingProgressEnum, bool)> {
    match reason {
        "redirect" => None,
        "stop" => Some((PlayingProgressEnum::Next, false)),
        "eof" if index < total => Some((PlayingProgressEnum::Next, false)),
        "eof" => Some((PlayingProgressEnum::EndOfPlaylist, true)),
        "quit" => Some((PlayingProgressEnum::Quit, true)),
        _ => Some((PlayingProgressEnum::Error, true)),
    }
}

fn playlist_has_next(playlist: &Value, ended: Option<u64>) -> Option<bool> {
    let entries = playlist.as_array()?;
    if let Some(index) = entries.iter().position(|entry| entry.get("id").and_then(Value::as_u64) == ended) {
        return Some(index + 1 < entries.len());
    }
    current_entry(playlist).and_then(|entry| entry.get("id")).and_then(Value::as_u64)
        .map(|active| Some(active) != ended)
}

fn matches_entry(event: Option<u64>, current: Option<u64>) -> bool {
    event.is_none() || current.is_none() || event == current
}

async fn active_entry(client: &Client) -> Result<Option<u64>> {
    let playlist = client.request(json!(["get_property", "playlist"])).await?;
    Ok(current_entry(&playlist).and_then(|entry| entry.get("id")).and_then(Value::as_u64))
}

fn current_entry(playlist: &Value) -> Option<&Value> {
    playlist.as_array()?.iter().find(|entry| entry.get("current").and_then(Value::as_bool) == Some(true))
}

async fn settle(current: &mut Option<CurrentMedia>, status: PlayingProgressEnum) {
    if let Some(media) = current.take() {
        let key = format!("{}-{}", media.param.id, media.param.media_source_select);
        media.param.axum_app_state.play_media_redirect_cache.write().await.remove(&key);
        save_playback_progress_async(media.param, media.position, status);
    }
}

pub(super) async fn playback_process(app: &AppHandle, session: &Session, client: &Client, mut events: tokio::sync::mpsc::UnboundedReceiver<Value>) -> Result<()> {
    let mut current: Option<CurrentMedia> = None;
    let mut entry_id = None;
    let mut entry_media = HashMap::<u64, (String, usize)>::new();
    let mut deferred = std::collections::VecDeque::new();
    #[cfg(target_os = "android")]
    let mut loading_token = String::new();
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    let result = async {
        loop {
            let message = if let Some(message) = deferred.pop_front() { message } else { tokio::select! {
                event = events.recv() => match event { Some(event) => event, None => break },
                _ = interval.tick(), if current.is_some() => {
                    let media = current.as_mut().unwrap();
                    if active_entry(client).await? != media.entry_id { continue; }
                    let position = client.request(json!(["get_property", "playback-time"])).await.ok().and_then(|value| value.as_f64());
                    let duration = client.request(json!(["get_property", "duration"])).await.ok().and_then(|value| value.as_f64());
                    if active_entry(client).await? != media.entry_id { continue; }
                    if let Some(value) = position.filter(|value| value.is_finite() && *value >= 0.0) {
                        media.position = Decimal::from_f64(value).unwrap_or_default().round();
                    }
                    if let Some(value) = duration.filter(|value| value.is_finite() && *value > 0.0) { media.param.file_duration = value; }
                    if chrono::Local::now() - media.saved_at >= chrono::Duration::seconds(30) {
                        media.saved_at = chrono::Local::now();
                        save_playback_progress_async(media.param.clone(), media.position, PlayingProgressEnum::Playing);
                    }
                    continue;
                }
            }};
            tracing::debug!("MPV IPC Received: {}", message);
            let event = message.get("event").and_then(Value::as_str);
            if event == Some("ipc-disconnected") {
                if let Some(detail) = message.get("detail").and_then(Value::as_str) { anyhow::bail!("mpv IPC: {detail}"); }
                break;
            }
            if event == Some("shutdown") { break; }
            if event == Some("start-file") {
                entry_id = message.get("playlist_entry_id").and_then(Value::as_u64);
                let playlist = client.request(json!(["get_property", "playlist"])).await?;
                let axum = app.state::<AppState>().auxm_app_state.read().await.clone().context("媒体服务尚未就绪")?;
                if let Some(entries) = playlist.as_array() {
                    for entry in entries {
                        if let (Some(id), Some(key)) = (
                            entry.get("id").and_then(Value::as_u64),
                            entry.get("filename").and_then(Value::as_str).and_then(|filename| media_key(filename, axum.port)),
                        ) { entry_media.insert(id, key); }
                    }
                }
                #[cfg(target_os = "android")]
                {
                    loading_token = uuid::Uuid::new_v4().simple().to_string();
                    client.request(json!(["set_property", "user-data/loemby-loading", loading_token])).await?;
                    client.request(json!(["set_property", "pause", true])).await?;
                }
            }
            if event == Some("file-loaded") {
                let playlist = client.request(json!(["get_property", "playlist"])).await?;
                let Some(entry) = current_entry(&playlist) else { continue; };
                let active = entry.get("id").and_then(Value::as_u64);
                if entry_id.is_some() && active != entry_id { continue; }
                entry_id = active;
                let axum = app.state::<AppState>().auxm_app_state.read().await.clone().context("媒体服务尚未就绪")?;
                let key = entry.get("filename").and_then(Value::as_str).and_then(|filename| media_key(filename, axum.port))
                    .or_else(|| active.and_then(|id| entry_media.get(&id).cloned()))
                    .context("播放器条目不属于本地播放计划")?;
                let mut param = session.media.lock().await.get(&key).cloned().context("当前媒体尚未注册播放信息")?;
                settle(&mut current, PlayingProgressEnum::Next).await;
                param.sender = Some(client.sender.clone());
                param.start_time = chrono::Local::now().timestamp();
                apply_media_cache(&param, client).await?;
                let initialized = {
                let initialization = play_info_init(&param);
                tokio::pin!(initialization);
                loop {
                    tokio::select! {
                        result = &mut initialization => break Some(result?),
                        event = events.recv() => {
                            let Some(event) = event else { break None; };
                            let name = event.get("event").and_then(Value::as_str);
                            let obsolete = matches!(name, Some("start-file" | "end-file" | "shutdown" | "ipc-disconnected"));
                            deferred.push_back(event);
                            if obsolete { break None; }
                        }
                    }
                }
                };
                let Some((episode, trakt, simkl, yamtrack)) = initialized else { continue; };
                param.episode = Some(episode);
                param.scrobble_trakt_param = trakt;
                param.scrobble_simkl_param = simkl;
                param.scrobble_yamtrack_param = yamtrack;
                param.play_info_init_finished = true;
                let position = Decimal::from_u64(param.params.playback_position_ticks / 10_000_000).unwrap_or_default();
                // 续播只应用一次，切集或切版本回来从头开始。
                for ((id, _), registered) in session.media.lock().await.iter_mut() {
                    if *id == param.id { registered.params.playback_position_ticks = 0; }
                }
                if let Some(registered) = axum.playlist.write().await.get_mut(&param.id) { registered.playback_position_ticks = 0; }
                param.params.playback_position_ticks = 0;
                current = Some(CurrentMedia { param, position, entry_id, saved_at: chrono::Local::now() });
                if active_entry(client).await? != entry_id {
                    settle(&mut current, PlayingProgressEnum::Next).await;
                    continue;
                }
                #[cfg(target_os = "android")]
                client.request(json!(["set_property", "user-data/loemby-ready", loading_token])).await?;
            }
            if message.get("id").and_then(Value::as_u64) == Some(10001) {
                if let Some(volume) = message.get("data").and_then(Value::as_f64).filter(|value| value.is_finite() && *value >= 0.0) {
                    let state = app.state::<AppState>();
                    let previous = global_config_mapper::get_cache("mpv_volume", &state).await.and_then(|value| value.parse::<f64>().ok());
                    if previous != Some(volume) {
                        if let Err(error) = global_config_mapper::create_or_update(GlobalConfig {
                            config_key: Some("mpv_volume".to_string()), config_value: Some(volume.to_string()), ..Default::default()
                        }, &state).await { tracing::error!("保存音量失败: {error:#}"); }
                    }
                }
            }
            if event == Some("end-file") {
                let reason = message.get("reason").and_then(Value::as_str).unwrap_or("error");
                let ended = message.get("playlist_entry_id").and_then(Value::as_u64);
                if reason == "redirect" {
                    if let (Some(key), Some(first), Some(count)) = (
                        ended.and_then(|id| entry_media.get(&id).cloned()),
                        message.get("playlist_insert_id").and_then(Value::as_u64),
                        message.get("playlist_insert_num_entries").and_then(Value::as_u64),
                    ) {
                        for id in first..first.saturating_add(count.min(10_000)) { entry_media.insert(id, key.clone()); }
                    }
                    continue;
                }
                if !matches_entry(ended, current.as_ref().map_or(entry_id, |media| media.entry_id)) { continue; }
                let (index, total) = current.as_ref().map_or((1, 1), |media| (media.param.params.playlist_index, media.param.params.playlist_total));
                let Some((mut status, mut terminal)) = end_status(reason, index, total) else { continue; };
                if reason == "eof" {
                    let playlist = client.request(json!(["get_property", "playlist"])).await?;
                    if let Some(has_next) = playlist_has_next(&playlist, ended) {
                        terminal = !has_next;
                        status = if terminal { PlayingProgressEnum::EndOfPlaylist } else { PlayingProgressEnum::Next };
                    }
                }
                settle(&mut current, status).await;
                if terminal {
                    anyhow::ensure!(reason == "eof" || reason == "quit", "mpv 加载或播放失败: {}", message);
                    break;
                }
            }
        }
        Ok(())
    }.await;
    let status = if result.is_ok() { PlayingProgressEnum::Quit } else { PlayingProgressEnum::Error };
    settle(&mut current, status).await;
    result
}

pub(super) fn start_process(app: AppHandle, socket: String, session: Arc<Session>, client: Client, events: tokio::sync::mpsc::UnboundedReceiver<Value>) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = playback_process(&app, &session, &client, events).await {
            tracing::error!("播放会话失败: {error:#}");
            let _ = app.emit("tauri_notify", TauriNotify {
                event_type: "PlaybackError".to_string(), message_type: "error".to_string(),
                title: Some("播放失败".to_string()), message: error.to_string(),
            });
        }
        #[cfg(desktop)]
        {
            let _ = tokio::time::timeout(Duration::from_secs(2), async {
                let mut writer = client.sender.write().await;
                writer.write_all(b"{\"command\":[\"quit\"]}\n").await?;
                writer.flush().await
            }).await;
            if let Some(window) = app.webview_windows().values().next() {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        cleanup(&app, &socket, &session).await;
    });
}

pub(super) async fn cleanup(app: &AppHandle, socket: &str, session: &Session) {
    let players = app.state::<Players>();
    if players.sessions.read().await.get(socket).is_none_or(|active| active.id != session.id) { return; }
    #[cfg(target_os = "android")]
    if !android_player::close(app, socket, session).await { return; }
    #[cfg(desktop)]
    if let Some(mut child) = session.child.lock().await.take() {
        if !matches!(child.try_wait(), Ok(Some(_))) { let _ = child.kill().await; }
        let _ = child.wait().await;
    }
    finish_cleanup(app, socket, &session.id).await;
}

pub(super) async fn finish_cleanup(app: &AppHandle, socket: &str, id: &str) {
    let players = app.state::<Players>();
    let mut sessions = players.sessions.write().await;
    let Some(session) = sessions.get(socket).cloned().filter(|session| session.id == id) else { return; };
    let axum = app.state::<AppState>().auxm_app_state.read().await.clone();
    if let Some(axum) = axum {
        let requests = session.requests.lock().await;
        axum.request.write().await.retain(|id, _| !requests.contains(id));
        axum.playlist.write().await.retain(|_, param| param.mpv_ipc != socket);
        axum.play_media_redirect_cache.write().await.retain(|key, _| !session.ids.iter().any(|id| key.starts_with(&format!("{id}-"))));
    }
    sessions.remove(socket);
    let _ = tokio::fs::remove_file(socket).await;
    #[cfg(target_os = "android")]
    {
        for name in ["playlist.m3u8", "loemby.mpv.conf", "cache_speed.lua"] {
            let _ = tokio::fs::remove_file(session.directory.join(name)).await;
        }
        let _ = tokio::fs::remove_dir(&session.directory).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_association_uses_local_url_and_version_not_playlist_index() {
        assert_eq!(media_key("http://127.0.0.1:1234/play_media/media-id/2", 1234), Some(("media-id".into(), 2)));
        assert_eq!(media_key("http://127.0.0.1:1234/play_media/media-id/0", 1234), Some(("media-id".into(), 0)));
        assert!(media_key("http://example.com:1234/play_media/media-id/2", 1234).is_none());
        assert!(media_key("http://127.0.0.1:4321/play_media/media-id/2", 1234).is_none());
        assert!(media_key("http://127.0.0.1:1234/play_media/media-id/invalid", 1234).is_none());
    }

    #[test]
    fn redirects_do_not_settle_and_stale_end_events_are_ignored() {
        assert!(end_status("redirect", 1, 2).is_none());
        assert!(!matches_entry(Some(1), Some(2)));
        assert!(matches_entry(Some(2), Some(2)));
        assert!(matches_entry(None, Some(2)));
    }

    #[test]
    fn redirected_last_episode_uses_actual_playlist_completion() {
        let playlist = json!([{"id": 10}, {"id": 11, "current": true}]);
        assert_eq!(playlist_has_next(&playlist, Some(10)), Some(true));
        assert_eq!(playlist_has_next(&playlist, Some(11)), Some(false));
        assert_eq!(playlist_has_next(&json!([{"id": 11, "current": true}]), Some(10)), Some(true));
    }

    #[test]
    fn switching_and_playlist_completion_have_shared_status_rules() {
        assert!(matches!(end_status("stop", 2, 2), Some((PlayingProgressEnum::Next, false))));
        assert!(matches!(end_status("eof", 1, 2), Some((PlayingProgressEnum::Next, false))));
        assert!(matches!(end_status("eof", 2, 2), Some((PlayingProgressEnum::EndOfPlaylist, true))));
        assert!(matches!(end_status("quit", 1, 2), Some((PlayingProgressEnum::Quit, true))));
        assert!(matches!(end_status("error", 1, 2), Some((PlayingProgressEnum::Error, true))));
    }
}
