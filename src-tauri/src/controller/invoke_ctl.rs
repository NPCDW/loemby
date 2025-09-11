use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::config::app_state::AppState;
use crate::config::runtime_config;
use crate::service::{cache_svc, http_forward_svc, player_svc, updater_svc};

#[tauri::command]
pub async fn get_sys_info() -> Result<String, String> {
    let hostname = sys_info::hostname();
    if hostname.is_err() {
        return Err(hostname.unwrap_err().to_string());
    }
    Ok(hostname.unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayVideoPlaylistParam {
    pub item_id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayVideoParam {
    pub path: String,
    pub title: String,
    pub item_id: String,
    pub item_type: String,
    pub item_name: String,
    pub emby_server_id: String,
    pub emby_server_name: String,
    pub series_id: Option<String>,
    pub series_name: Option<String>,
    pub media_source_id: String,
    pub play_session_id: String,
    pub playback_position_ticks: u64,
    pub run_time_ticks: u64,
    pub bitrate: Option<u64>,
    pub vid: i32,
    pub aid: i32,
    pub sid: i32,
    pub external_audio: Vec<String>,
    pub external_subtitle: Vec<String>,
    pub scrobble_trakt_param: Option<String>,
    pub start_time: u64,
}

#[tauri::command]
pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    player_svc::play_video(body, &state, app_handle).await
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpForwardParam {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub proxy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpForwardResult {
    pub status_code: u16,
    pub status_text: Option<&'static str>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[tauri::command]
pub async fn http_forward(param: HttpForwardParam, state: tauri::State<'_, AppState>) -> Result<HttpForwardResult, String> {
    let res = http_forward_svc::forward(param, &state).await;
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }
    let res = res.unwrap();
    Ok(HttpForwardResult {
        status_code: res.status().as_u16(),
        status_text: res.status().canonical_reason(),
        headers: res.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string())).collect(),
        body: res.text().await.unwrap_or_else(|e| {
            return format!("http 转发解析响应体失败: {:?}", e);
        }),
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadImageParam {
    pub image_url: String,
    pub proxy_url: Option<String>,
    pub user_agent: String,
}

#[tauri::command]
pub async fn go_trakt_auth(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let auxm_app_state = state.auxm_app_state.clone();
    let auxm_app_state = auxm_app_state.read().await.clone();
    let auxm_app_state = auxm_app_state.as_ref().unwrap();

    let client_id = "05521c50a5a5ac1fb238648a15e8da57ea7c708127e49711303c9b9691913572";
    let redirect_uri = format!("http://127.0.0.1:{}/trakt_auth", auxm_app_state.port);
    let state = uuid::Uuid::new_v4().to_string();
    let url = format!("https://api.trakt.tv/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&state={}", client_id, redirect_uri, state);
    auxm_app_state.trakt_auth_state.write().await.push(state);
    let res = webbrowser::open(&url);
    if let Err(err) = res {
        return Err(format!("打开浏览器失败: {} 您可尝试手动复制链接到浏览器中打开 {}", err.to_string(), &url));
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdaterParam {
    pub proxy_url: Option<String>,
    pub user_agent: String,
}

#[tauri::command]
pub async fn updater(body: UpdaterParam, app_handle: tauri::AppHandle) -> Result<bool, String> {
    let res = updater_svc::update(body, app_handle).await;
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
