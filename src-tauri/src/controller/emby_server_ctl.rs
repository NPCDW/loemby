use serde::{Deserialize, Serialize};

use crate::config::app_state::AppState;
use crate::mapper::emby_server_mapper;
use crate::mapper::emby_server_mapper::EmbyServer;

#[tauri::command]
pub async fn get_emby_server(id: String, state: tauri::State<'_, AppState>) -> Result<Option<EmbyServer>, String> {
    let res = emby_server_mapper::get_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_all_emby_server(state: tauri::State<'_, AppState>) -> Result<Vec<EmbyServer>, String> {
    let res = emby_server_mapper::list_all(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_emby_server(body: EmbyServer, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_server_mapper::create(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_emby_server(body: EmbyServer, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_server_mapper::update_by_id(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmbyServerOrderParam {
    pub removed_id: String,
    pub removed_index: u32,
    pub added_index: u32,
}

#[tauri::command]
pub async fn update_emby_server_order(body: UpdateEmbyServerOrderParam, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_server_mapper::update_order(body.removed_id, body.removed_index, body.added_index, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn defer_emby_server_order(state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_server_mapper::defer_order(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn delete_emby_server(id: String, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_server_mapper::delete_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}
