use std::path::PathBuf;

use rust_decimal::prelude::*;

use serde::Serialize;
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

    for audio in body.external_audio {
        command = command.arg(&format!("--audio-file={}", &audio));
    }
    for subtitle in body.external_subtitle {
        command = command.arg(&format!("--sub-file={}", &subtitle));
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
    
    #[cfg(windows)]
    let playback_progress_process = tauri::async_runtime::spawn(async move {
        use {interprocess::os::windows::named_pipe::*, recvmsg::prelude::*};
        // Preemptively allocate a sizeable buffer for receiving. Keep in mind that this will depend
        // on the specifics of the protocol you're using.
        let mut buffer = MsgBuf::from(Vec::with_capacity(128));
    
        // Create our connection. This will block until the server accepts our connection, but will
        // fail immediately if the server hasn't even started yet; somewhat similar to how happens
        // with TCP, where connecting to a port that's not bound to any server will send a "connection
        // refused" response, but that will take twice the ping, the roundtrip time, to reach the
        // client.
        let mut conn = DuplexPipeStream::<pipe_mode::Messages>::connect_by_path(pipe_name)?;
    
        // Here's our message so that we could check its length later.
        let command = serde_json::json!({
            "command": ["get_property", "time-pos"]
        });
        let MESSAGE = serde_json::to_vec(&command).unwrap();
        // Send the message, getting the amount of bytes that was actually sent in return.
        let sent = conn.send(MESSAGE)?;
        assert_eq!(sent, MESSAGE.len()); // If it doesn't match, something's seriously wrong.
    
        // Use the reliable message receive API, which gets us a `RecvResult` from the
        // `reliable_recv_msg` module.
        conn.recv_msg(&mut buffer, None)?;
    
        // Convert the data that's been received into a string. This checks for UTF-8 validity, and if
        // invalid characters are found, a new buffer is allocated to house a modified version of the
        // received data, where decoding errors are replaced with those diamond-shaped question mark
        // U+FFFD REPLACEMENT CHARACTER thingies: �.
        let received_string = String::from_utf8_lossy(buffer.filled_part());
    
        // Print out the result!
        println!("Server answered: {received_string}");
        // use tokio::io::{AsyncReadExt, AsyncWriteExt};
        // 连接到命名管道
        // let mut retry_count = 0;
        // let client = loop {
        //     if retry_count >= 10 {
        //         break None;
        //     }
        //     let client = tokio::net::windows::named_pipe::ClientOptions::new().open(pipe_name);
        //     if client.is_ok() {
        //         tracing::debug!("mpv IPC connected");
        //         break Some(client.unwrap());
        //     }
        //     tracing::debug!("Failed to connect to mpv IPC, retrying...");
        //     tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        //     retry_count += 1;
        // };
        // let mut client = match client {
        //     Some(client) => client,
        //     None => {
        //         tracing::error!("Failed to connect to mpv IPC");
        //         return;
        //     }
        // };

        // // 发送获取播放进度的命令
        // let command = serde_json::json!({
        //     "command": ["get_property", "time-pos"]
        // });

        // let request = serde_json::to_vec(&command).unwrap();
        // client.write_all(&request).await.expect("Failed to write to pipe");

        // // 读取响应
        // let mut buffer = [0u8; 1024];
        // let n = client.read(&mut buffer).await.expect("Failed to read from pipe");
        // let response = String::from_utf8_lossy(&buffer[..n]);

        // // 解析响应
        // let json: serde_json::Value = serde_json::from_str(&response).unwrap();
        // let progress = json["data"].as_f64().unwrap_or(0.0);

        // tracing::debug!("Current playback position: {} seconds", progress);
    });

    tauri::async_runtime::spawn(async move {
        let (mut rx, mut _child) = player.unwrap();
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Terminated(_payload) = event {
                // 读取保存的播放进度
                let path_md5 = md5::compute(&video_path);
                let progress_path = format!("{}", &watch_later_dir.join(format!("{:x}", path_md5)).as_os_str().to_str().unwrap());
                let watch_later = std::fs::read_to_string(progress_path).unwrap_or_default();
                tracing::debug!("播放结束 {:?}", watch_later);

                watch_later.split("\n").for_each(|line| {
                    if line.starts_with("start=") {
                        let position = Decimal::from_str(line.split("=").nth(1).unwrap()).unwrap() * Decimal::from_i64(1000_0000).unwrap();
                        let position = position.round();
                        tracing::debug!("播放进度 {}", position);
                        app_handle.emit("playback_progress", PlaybackProgress {
                            server_id: &body.server_id,
                            item_id: &body.item_id,
                            media_source_id: &body.media_source_id,
                            play_session_id: &body.play_session_id,
                            progress: position,
                        }).unwrap();
                    }
                });
                #[cfg(windows)]
                playback_progress_process.abort();
                break;
            }
        }
    });

    Ok(())
}

#[derive(Clone, Serialize)]
struct PlaybackProgress<'a> {
    server_id: &'a str,
    item_id: &'a str,
    media_source_id: &'a str,
    play_session_id: &'a str,
    progress: Decimal
}