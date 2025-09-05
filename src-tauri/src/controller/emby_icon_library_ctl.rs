use crate::config::app_state::AppState;
use crate::mapper::emby_icon_library_mapper;
use crate::mapper::emby_icon_library_mapper::EmbyIconLibrary;

#[tauri::command]
pub async fn get_emby_icon_library(id: String, state: tauri::State<'_, AppState>) -> Result<EmbyIconLibrary, String> {
    let res = emby_icon_library_mapper::get_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_all_emby_icon_library(state: tauri::State<'_, AppState>) -> Result<Vec<EmbyIconLibrary>, String> {
    let res = emby_icon_library_mapper::list_all(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_emby_icon_library(body: EmbyIconLibrary, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_icon_library_mapper::create(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_emby_icon_library(body: EmbyIconLibrary, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_icon_library_mapper::update_by_id(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn delete_emby_icon_library(id: String, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_icon_library_mapper::delete_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}
