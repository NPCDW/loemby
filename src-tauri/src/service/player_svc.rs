use std::path::PathBuf;

use rust_decimal::prelude::*;

use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;

use crate::{config::app_state::AppState, util::file_util};

#[tauri::command]
pub async fn play_video(path: String, server_id: String, item_id: String, media_source_id: String, playback_position_ticks: u64,
    play_session_id: String, aid: i32, sid: i32, external_audio: Vec<String>, external_subtitle: Vec<String>,
    state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mpv_path = state.app_config.read().await.mpv_path.clone();
    if mpv_path.is_none() {
        return Err("未配置 mpv 播放器路径".to_string());
    }

    let watch_later_dir = state.root_dir.join("watch_later");
    file_util::mkdir(&watch_later_dir).expect("Failed to create watch_later dir");

    let mpv_path = PathBuf::from(mpv_path.as_ref().unwrap());
    let mpv_parent_path = mpv_path.parent().unwrap();

    let video_path = path.clone();
    let mut command = app_handle.shell().command(&mpv_path.as_os_str().to_str().unwrap())
        .current_dir(&mpv_parent_path.as_os_str().to_str().unwrap())
        .arg("--force-window=immediate")
        .arg("--save-position-on-quit")
        .arg(&format!("--watch-later-directory={}", &watch_later_dir.as_os_str().to_str().unwrap()))
        .arg(&format!("--start=+{}", playback_position_ticks / 1000_0000))
        .arg(&video_path);

    for audio in external_audio {
        command = command.arg(&format!("--audio-file={}", &audio));
    }
    for subtitle in external_subtitle {
        command = command.arg(&format!("--sub-file={}", &subtitle));
    }
    if aid == -1 {
        command = command.arg(&format!("--aid=no"));
    } else {
        command = command.arg(&format!("--aid={}", aid));
    }
    if sid == -1 {
        command = command.arg(&format!("--sid=no"));
    } else {
        command = command.arg(&format!("--sid={}", sid));
    }
    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("播放视频: {} {:?}", &video_path, &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

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
                            server_id: &server_id,
                            item_id: &item_id,
                            media_source_id: &media_source_id,
                            play_session_id: &play_session_id,
                            progress: position,
                        }).unwrap();
                    }
                });
            
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