use serde::{Deserialize, Serialize};

use crate::config::app_state::AppState;
use crate::mapper::play_history_mapper;
use crate::mapper::play_history_mapper::PlayHistory;

#[derive(Serialize, Deserialize)]
pub struct GetPlayHistoryParam {
    pub emby_server_id: String,
    pub item_id: String
}

#[tauri::command]
pub async fn get_play_history(body: GetPlayHistoryParam, state: tauri::State<'_, AppState>) -> Result<Option<PlayHistory>, String> {
    let res = play_history_mapper::get(body.emby_server_id, body.item_id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct PagePlayHistoryParam {
    pub page_number: i32,
    pub page_size: i32,
    pub emby_server_id: Option<String>,
    pub series_name: Option<String>,
    pub item_name: Option<String>,
}

#[tauri::command]
pub async fn page_play_history(body: PagePlayHistoryParam, state: tauri::State<'_, AppState>) -> Result<(u32, Vec<PlayHistory>), String> {
    let res = play_history_mapper::page(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_play_history(body: PlayHistory, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = play_history_mapper::create(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_play_history(body: PlayHistory, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = play_history_mapper::update_by_id(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[derive(Serialize, Deserialize)]
pub struct CancelPinnedHistoryParam {
    pub emby_server_id: String,
    pub series_id: String
}

#[tauri::command]
pub async fn cancel_pinned_play_history(body: CancelPinnedHistoryParam, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = play_history_mapper::cancel_pinned(body.emby_server_id, body.series_id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}
