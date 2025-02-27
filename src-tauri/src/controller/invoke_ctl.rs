use serde::{Deserialize, Serialize};
use tauri::State;
use crate::config::app_state::AppState;
use crate::config::app_config::{self, Config};
use crate::service::player_svc;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, ()> {
    Ok(state.app_config.read().await.clone())
}

#[tauri::command]
pub async fn save_config(state: tauri::State<'_, AppState>, config: Config) -> Result<(), ()> {
    app_config::save_config(state, config).await
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayVideoParam {
    pub path: String,
    pub server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub playback_position_ticks: u64,
    pub play_session_id: String,
    pub aid: i32,
    pub sid: i32,
    pub external_audio: Vec<String>,
    pub external_subtitle: Vec<String>,
}

#[tauri::command]
pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
        player_svc::play_video(body, state, app_handle).await
}
