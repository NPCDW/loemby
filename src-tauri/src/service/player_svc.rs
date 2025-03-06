use std::path::PathBuf;

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

use crate::{config::{app_state::AppState, http_pool}, controller::invoke_ctl::PlayVideoParam, service::proxy_svc::AxumAppStateConnect};

pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mpv_path = state.app_config.read().await.mpv_path.clone();
    if mpv_path.is_none() {
        return Err("未配置 mpv 播放器路径".to_string());
    }

    let mpv_path = PathBuf::from(mpv_path.as_ref().unwrap());
    if !mpv_path.exists() {
        return Err(format!("mpv 路径不存在: {}", mpv_path.to_str().unwrap()));
    }
    let mpv_parent_path = mpv_path.parent().unwrap();

    let auxm_app_state = state.auxm_app_state.clone();
    let mut app_state = auxm_app_state.read().await.clone();
    let app_state = app_state.as_mut().unwrap();
    
    let client = http_pool::get_http_client(body.proxy.clone(), state).await;
    if client.is_err() {
        return Err(client.unwrap_err().to_string());
    }
    let client = client.unwrap();

    let uuid = uuid::Uuid::new_v4().to_string();
    app_state.connect.write().await.insert(uuid.clone(), AxumAppStateConnect {stream_url: body.path.clone(), client: client.clone(), user_agent: body.user_agent.clone()});
    let video_path = format!("http://127.0.0.1:{}/stream/{}", &app_state.port, &uuid);

    let pipe_name = r"\\.\pipe\mpvsocket";
    let pipe_name = format!("{}-{}-{}", &pipe_name, &body.server_id, &body.media_source_id);
    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap());
    command.current_dir(&mpv_parent_path.as_os_str().to_str().unwrap())
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        .arg(&format!("--user-agent={}", &body.user_agent))
        .arg(&format!("--title={}", &body.title))
        .arg(&format!("--force-media-title={}", &body.title))
        .arg(&format!("--start=+{}", body.playback_position_ticks / 1000_0000))
        .arg(&video_path);

    for audio in &body.external_audio {
        let uuid = uuid::Uuid::new_v4().to_string();
        app_state.connect.write().await.insert(uuid.clone(), AxumAppStateConnect {stream_url: audio.clone(), client: client.clone(), user_agent: body.user_agent.clone()});
        command.arg(&format!("--audio-file={}", format!("http://127.0.0.1:{}/stream/{}", &app_state.port, &uuid)));
    }
    for subtitle in &body.external_subtitle {
        let uuid = uuid::Uuid::new_v4().to_string();
        app_state.connect.write().await.insert(uuid.clone(), AxumAppStateConnect {stream_url: subtitle.clone(), client: client.clone(), user_agent: body.user_agent.clone()});
        command.arg(&format!("--sub-file={}", format!("http://127.0.0.1:{}/stream/{}", &app_state.port, &uuid)));
    }
    if body.vid == -1 {
        command.arg(&format!("--vid=no"));
    } else if body.vid == 0 {
        command.arg(&format!("--vid=auto"));
    } else {
        command.arg(&format!("--vid={}", body.aid));
    }
    if body.aid == -1 {
        command.arg(&format!("--aid=no"));
    } else if body.aid == 0 {
        command.arg(&format!("--aid=auto"));
    } else {
        command.arg(&format!("--aid={}", body.aid));
    }
    if body.sid == -1 {
        command.arg(&format!("--sid=no"));
    } else if body.sid == 0 {
        command.arg(&format!("--sid=auto"));
    } else {
        command.arg(&format!("--sid={}", body.sid));
    }
    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("播放视频: {} {:?}", &video_path, &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    let body_clone = body.clone();
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let res = playback_progress(&pipe_name, &mut player.unwrap(), body_clone, app_handle_clone).await;
        if res.is_err() {
            tracing::error!("播放进度失败: {:?}", res.unwrap_err());
            save_playback_progress(&body, &app_handle, Decimal::from_u64(body.playback_position_ticks).unwrap(), 0);
        }
    });

    Ok(())
}

async fn playback_progress(pipe_name: &str, player: &mut tokio::process::Child, body: PlayVideoParam, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    use tokio as tokio_root;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    #[cfg(windows)]
    use interprocess::os::windows::named_pipe::{pipe_mode, tokio::*};
    #[cfg(not(windows))]
    use interprocess::local_socket::{tokio::{prelude::*, Stream}, GenericFilePath};

    let mut retry_count = 0;
    let conn = loop {
        if retry_count >= 10 {
            break None;
        }
        #[cfg(windows)]
        let conn = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_name).await;
        #[cfg(not(windows))]
        let conn = {
            let name = pipe_name.to_fs_name::<GenericFilePath>();
            if name.is_err() {
                return Err(anyhow::anyhow!("MPV IPC Failed to convert pipe name to fs name"));
            }
            let name = name.unwrap();
            Stream::connect(name.clone()).await
        };
        if conn.is_ok() {
            tracing::debug!("MPV IPC connected");
            break Some(conn.unwrap());
        }
        tracing::debug!("MPV IPC Failed to connect to mpv IPC, retrying...");
        tokio_root::time::sleep(std::time::Duration::from_millis(500)).await;
        retry_count += 1;
    };
    let conn = match conn {
        Some(conn) => conn,
        None => {
            return Err(anyhow::anyhow!("MPV IPC Failed to connect to mpv IPC"));
        }
    };
    let (recver, mut sender) = conn.split();
    let mut recver = BufReader::new(recver);

    let send_task = tokio_root::spawn(async move {
        let command = r#"{ "command": ["get_property", "playback-time"], "request_id": 10023 }"#.to_string() + "\n";
        loop {
            let write = sender.write_all(command.as_bytes()).await;
            if write.is_err() {
                tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
                break;
            }
            tokio_root::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let mut last_save_time = chrono::Local::now();
    let mut last_record_position = Decimal::from_u64(body.playback_position_ticks).unwrap();
    loop {
        let mut buffer = String::with_capacity(128);
        let read = recver.read_line(&mut buffer).await;
        if read.is_err() {
            tracing::error!("MPV IPC Failed to read pipe {:?}", read);
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        tracing::debug!("MPV IPC Server answered: {}", buffer.trim());
        let json = serde_json::from_str::<MpvIpcResponse>(&buffer);
        if json.is_err() {
            tracing::error!("解析 mpv-ipc 响应失败 {:?}", json);
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        let json = json.unwrap();
        if let Some("end-file") = json.event {
            tracing::debug!("MPV IPC 播放结束");
            send_task.abort();
            let _ = player.kill().await;
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        if let Some(10023) = json.request_id {
            let progress = json.data;
            if let Some(progress) = progress {
                tracing::debug!("MPV IPC 播放进度 {}", progress);
                last_record_position = Decimal::from_f64(progress).unwrap() * Decimal::from_i64(1000_0000).unwrap();
                last_record_position = last_record_position.round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&body, &app_handle, last_record_position, 1);
                }
            }
        }
    }
    anyhow::Ok(())
}

fn save_playback_progress(body: &PlayVideoParam, app_handle: &tauri::AppHandle, last_record_position: Decimal, playback_status: u32) {
    if playback_status == 0 {
        let window = app_handle.webview_windows();
        let window = window.values().next().expect("Sorry, no window found");
        window.unminimize().expect("Sorry, no window unminimize");
        window.show().expect("Sorry, no window show");
        window.set_focus().expect("Can't Bring Window to Focus");
    }
    app_handle.emit("playback_progress", PlaybackProgress {
        server_id: &body.server_id,
        item_id: &body.item_id,
        media_source_id: &body.media_source_id,
        play_session_id: &body.play_session_id,
        progress: last_record_position,
        playback_status: playback_status,
    }).unwrap();
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct MpvIpcResponse<'a> {
    event: Option<&'a str>,
    data: Option<f64>,
    request_id: Option<u32>,
    error: Option<&'a str>,
}

#[derive(Clone, Serialize)]
struct PlaybackProgress<'a> {
    server_id: &'a str,
    item_id: &'a str,
    media_source_id: &'a str,
    play_session_id: &'a str,
    progress: Decimal,
    // 0 停止  1 播放中
    playback_status: u32,
}