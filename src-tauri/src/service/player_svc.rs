use tauri_plugin_shell::ShellExt;

use crate::{config::app_state::AppState, util::file_util};

#[tauri::command]
pub async fn play_video(path: String, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), ()> {
    let watch_later_dir = state.root_dir.join("watch_later");
    file_util::create_file_if_not_exist(&watch_later_dir).expect("Failed to create watch_later dir");

    let shell = app_handle.shell();
    let video_path = path.clone();
    shell.command("mpv.exe")
        .current_dir("C:\\App\\mpv_config-2024.12.04")
        .arg("--save-position-on-quit")
        .arg("--watch-later-directory")
        .arg(&watch_later_dir)
        .arg(&video_path)
        .status()
        .await
        .expect("Failed to start MPV");

    // 读取保存的播放进度
    // let progress_path = format!("{}.watch_later", video_path);
    // let progress = std::fs::read_to_string(progress_path).unwrap_or_default();

    // // 发送播放进度回 Tauri 应用
    // window.emit("playback_progress", progress).unwrap();

    Ok(())
}