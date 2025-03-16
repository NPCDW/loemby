use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::config::app_state::AppState;
use crate::service::{http_forward, player_svc};

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
    pub mpv_path: String,
    pub path: String,
    pub proxy: Option<String>,
    pub title: String,
    pub user_agent: String,
    pub server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub playback_position_ticks: u64,
    pub play_session_id: String,
    pub vid: i32,
    pub aid: i32,
    pub sid: i32,
    pub external_audio: Vec<String>,
    pub external_subtitle: Vec<String>,
}

#[tauri::command]
pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    player_svc::play_video(body, state, app_handle).await
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
    let res = http_forward::forward(param, state).await;
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
pub async fn load_image(body: LoadImageParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = http_forward::load_image(body, state).await;
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }
    Ok(res.unwrap())
}