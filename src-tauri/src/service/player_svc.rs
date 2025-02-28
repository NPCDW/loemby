use std::path::PathBuf;

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;

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
    let video_path = body.path.clone();
    let mut command = app_handle.shell().command(&mpv_path.as_os_str().to_str().unwrap())
        .current_dir(&mpv_parent_path.as_os_str().to_str().unwrap())
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        .arg("--save-position-on-quit")
        .arg(&format!("--watch-later-directory={}", &watch_later_dir.as_os_str().to_str().unwrap()))
        .arg(&format!("--start=+{}", body.playback_position_ticks / 1000_0000))
        .arg(&video_path);

    for audio in &body.external_audio {
        command = command.arg(&format!("--audio-file={}", audio));
    }
    for subtitle in &body.external_subtitle {
        command = command.arg(&format!("--sub-file={}", subtitle));
    }
    if body.aid == -1 {
        command = command.arg(&format!("--aid=no"));
    } else {
        command = command.arg(&format!("--aid={}", body.aid));
    }
    if body.sid == -1 {
        command = command.arg(&format!("--sid=no"));
    } else {
        command = command.arg(&format!("--sid={}", body.sid));
    }
    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("播放视频: {} {:?}", &video_path, &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    let body_clone = body.clone();
    let app_handle_clone = app_handle.clone();
    let playback_progress_process = tauri::async_runtime::spawn(async move {
        playback_progress(pipe_name, body_clone, app_handle_clone).await;
    });

    tauri::async_runtime::spawn(async move {
        let (mut rx, mut _child) = player.unwrap();
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Terminated(_payload) = event {
                playback_progress_process.abort();
                // 读取保存的播放进度
                let path_md5 = md5::compute(&video_path);
                let progress_path = format!("{}", &watch_later_dir.join(format!("{:x}", path_md5)).as_os_str().to_str().unwrap());
                let watch_later = std::fs::read_to_string(progress_path).unwrap_or_default();
                tracing::debug!("播放结束 {:?}", watch_later);

                watch_later.split("\n").for_each(|line| {
                    if line.starts_with("start=") {
                        let position = Decimal::from_str(line.split("=").nth(1).unwrap()).unwrap() * Decimal::from_i64(1000_0000).unwrap();
                        let position = position.round();
                        tracing::debug!("播放结束进度 {}", position);
                        app_handle.emit("playback_progress", PlaybackProgress {
                            server_id: &body.server_id,
                            item_id: &body.item_id,
                            media_source_id: &body.media_source_id,
                            play_session_id: &body.play_session_id,
                            progress: position,
                            playback_status: 0,
                        }).unwrap();
                    }
                });
                break;
            }
        }
    });

    Ok(())
}

async fn playback_progress(pipe_name: &str, body: PlayVideoParam, app_handle: tauri::AppHandle) {
    use tokio as tokio_root;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    #[cfg(windows)]
    use interprocess::os::windows::named_pipe::{pipe_mode, tokio::*};
    #[cfg(not(windows))]
    use interprocess::local_socket::{tokio::{prelude::*, Stream}, GenericFilePath, GenericNamespaced};

    let mut retry_count = 0;
    let conn = loop {
        if retry_count >= 10 {
            break None;
        }
        #[cfg(windows)]
        let conn = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_name).await;
        #[cfg(not(windows))]
        let conn = {
            let name = if GenericNamespaced::is_supported() {
                pipe_name.to_ns_name::<GenericNamespaced>()
            } else {
                pipe_name.to_fs_name::<GenericFilePath>()
            };
            if name.is_err() {
                tracing::debug!("Failed to convert pipe name to fs name");
                return;
            }
            let name = name.unwrap();
            Stream::connect(name.clone()).await
        };
        if conn.is_ok() {
            tracing::debug!("mpv IPC connected");
            break Some(conn.unwrap());
        }
        tracing::debug!("Failed to connect to mpv IPC, retrying...");
        tokio_root::time::sleep(std::time::Duration::from_secs(10)).await;
        retry_count += 1;
    };
    let conn = match conn {
        Some(conn) => conn,
        None => {
            tracing::debug!("Failed to connect to mpv IPC");
            return;
        }
    };
    let (recver, mut sender) = conn.split();
    let mut recver = BufReader::new(recver);

    tokio_root::spawn(async move {
        let command = r#"{ "command": ["get_property", "playback-time"], "request_id": 10023 }"#.to_string() + "\n";
        loop {
            let write = sender.write_all(command.as_bytes()).await;
            if write.is_err() {
                tracing::debug!("Failed to write to pipe {:?}", write);
                break;
            }
            tokio_root::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    loop {
        let mut buffer = String::with_capacity(128);
        let read = recver.read_line(&mut buffer).await;
        if read.is_err() {
            tracing::debug!("Failed to read pipe {:?}", read);
            break;
        }
        tracing::debug!("mpv-ipc Server answered: {}", buffer.trim());
        let json = serde_json::from_str::<MpvIpcResponse>(&buffer);
        if json.is_err() {
            tracing::error!("解析 mpv-ipc 响应失败 {:?}", json);
            break;
        }
        let json = json.unwrap();
        if let Some("end-file") = json.event {
            tracing::debug!("播放结束");
            break;
        }
        if let Some(10023) = json.request_id {
            let progress = json.data.unwrap_or(0.0);
            tracing::debug!("播放进度 {}", progress);
            let position = Decimal::from_f64(progress).unwrap() * Decimal::from_i64(1000_0000).unwrap();
            let position = position.round();
            app_handle.emit("playback_progress", PlaybackProgress {
                server_id: &body.server_id,
                item_id: &body.item_id,
                media_source_id: &body.media_source_id,
                play_session_id: &body.play_session_id,
                progress: position,
                playback_status: 1,
            }).unwrap();
        }
        tokio_root::time::sleep(std::time::Duration::from_millis(100)).await;
    }
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