use std::path::PathBuf;

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

use crate::{config::app_state::AppState, controller::invoke_ctl::PlayVideoParam, util::file_util};

pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mpv_path = state.app_config.read().await.mpv_path.clone();
    if mpv_path.is_none() {
        return Err("未配置 mpv 播放器路径".to_string());
    }

    let watch_later_dir = state.root_dir.join("watch_later");
    file_util::mkdir(&watch_later_dir).expect("Failed to create watch_later dir");

    let mpv_path = PathBuf::from(mpv_path.as_ref().unwrap());
    let mpv_parent_path = mpv_path.parent().unwrap();

    let pipe_name = r"\\.\pipe\mpvsocket";
    let pipe_name = format!("{}-{}-{}", &pipe_name, &body.server_id, &body.media_source_id);
    let video_path = body.path.clone();
    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap());
    command.current_dir(&mpv_parent_path.as_os_str().to_str().unwrap())
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        .arg("--save-position-on-quit")
        .arg(&format!("--start=+{}", body.playback_position_ticks / 1000_0000))
        .arg(&video_path);

    for audio in &body.external_audio {
        command.arg(&format!("--audio-file={}", audio));
    }
    for subtitle in &body.external_subtitle {
        command.arg(&format!("--sub-file={}", subtitle));
    }
    if body.aid == -1 {
        command.arg(&format!("--aid=no"));
    } else {
        command.arg(&format!("--aid={}", body.aid));
    }
    if body.sid == -1 {
        command.arg(&format!("--sid=no"));
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
        let res = playback_progress(&pipe_name, body_clone, app_handle_clone).await;
        if res.is_err() {
            tracing::error!("播放进度失败: {:?}", res.unwrap_err());
            save_playback_progress(&body, &app_handle, Decimal::from_u64(body.playback_position_ticks).unwrap(), 0);
        }
    });

    Ok(())
}

async fn playback_progress(pipe_name: &str, body: PlayVideoParam, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
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

    tokio_root::spawn(async move {
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
    let mut last_record_position = Decimal::from_i64(0).unwrap();
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
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        if let Some(10023) = json.request_id {
            let progress = json.data.unwrap_or(0.0);
            tracing::debug!("MPV IPC 播放进度 {}", progress);
            last_record_position = Decimal::from_f64(progress).unwrap() * Decimal::from_i64(1000_0000).unwrap();
            last_record_position = last_record_position.round();
            if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                last_save_time = chrono::Local::now();
                save_playback_progress(&body, &app_handle, last_record_position, 1);
            }
        }
    }
    anyhow::Ok(())
}

fn save_playback_progress(body: &PlayVideoParam, app_handle: &tauri::AppHandle, last_record_position: Decimal, playback_status: u32) {
    app_handle.webview_windows().values().next()
        .expect("Sorry, no window found")
        .set_focus()
        .expect("Can't Bring Window to Focus");
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