use serde::{Deserialize, Serialize};
use crate::config::app_state::AppState;
use crate::config::runtime_config;
use crate::service::{cache_svc, player_svc, trakt_http_svc, updater_svc};

#[tauri::command]
pub async fn get_sys_info() -> Result<String, String> {
    let hostname = sys_info::hostname();
    if hostname.is_err() {
        return Err(hostname.unwrap_err().to_string());
    }
    Ok(hostname.unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayVideoParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub playback_position_ticks: u64,
    pub use_direct_link: bool,
    pub select_policy: String,
    pub video_select: i32,
    pub audio_select: i32,
    pub subtitle_select: i32,
    pub version_select: i32,
}

#[tauri::command]
pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    player_svc::play_video(body, &state, app_handle).await
}

#[tauri::command]
pub async fn go_trakt_auth(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let res = trakt_http_svc::go_trakt_auth(&state).await;
    if let Err(err) = res {
        return Err(format!("{}", err.to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    let res = webbrowser::open(&url);
    if let Err(err) = res {
        return Err(format!("打开浏览器失败: {} ", err.to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn updater(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let res = updater_svc::update(app_handle).await;
    if let Err(err) = res {
        return Err(format!("更新失败: {} ", err.to_string()));
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn restart_app(app_handle: tauri::AppHandle) {
    app_handle.restart()
}

#[tauri::command]
pub async fn get_runtime_config(state: tauri::State<'_, AppState>) -> Result<runtime_config::RuntimeConfig, ()> {
    runtime_config::get_runtime_config(state).await
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleanEmbyCacheParam {
    pub emby_server_id: Option<String>,
}

#[tauri::command]
pub async fn clean_emby_image_cache(body: CleanEmbyCacheParam, app_handle: tauri::AppHandle) -> Result<(), String> {
    let res = cache_svc::clean_emby_image(body.emby_server_id, true, &app_handle).await;
    if let Err(err) = res {
        return Err(format!("清理失败: {} ", err.to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn clean_icon_cache(app_handle: tauri::AppHandle) -> Result<(), String> {
    let res = cache_svc::clean_icon(true, &app_handle).await;
    if let Err(err) = res {
        return Err(format!("清理失败: {} ", err.to_string()));
    }
    Ok(())
}
