use std::path::Path;

use tauri_plugin_shell::ShellExt;

use crate::{config::app_state::AppState, util::file_util};

#[tauri::command]
pub async fn play_video(path: String, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mpv_path = state.app_config.read().await.mpv_path.clone();
    if mpv_path.is_none() {
        return Err("未配置 mpv 播放器路径".to_string());
    }
    let mpv_path = Path::new(mpv_path.as_ref().unwrap());
    let mpv_parent_path = mpv_path.parent().unwrap();

    let watch_later_dir = state.root_dir.join("watch_later");
    file_util::mkdir(&watch_later_dir).expect("Failed to create watch_later dir");

    let shell = app_handle.shell();
    let video_path = path.clone();
    let player = shell.command(&mpv_path.as_os_str().to_str().unwrap())
        .current_dir(&mpv_parent_path.as_os_str().to_str().unwrap())
        .current_dir("C:\\App\\mpv_config-2024.12.04")
        .arg("--save-position-on-quit")
        .arg(&format!("--watch-later-directory={}", &watch_later_dir.as_os_str().to_str().unwrap()))
        .arg(&video_path)
        .status()
        .await;

    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    // 读取保存的播放进度
    // let progress_path = format!("{}", &watch_later_dir.join(md5(path)).as_os_str().to_str().unwrap());
    // let progress = std::fs::read_to_string(progress_path).unwrap_or_default();

    // // 发送播放进度回 Tauri 应用
    // window.emit("playback_progress", progress).unwrap();

    Ok(())
}