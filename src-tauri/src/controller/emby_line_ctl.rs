use crate::config::app_state::AppState;
use crate::mapper::emby_line_mapper;
use crate::mapper::emby_line_mapper::EmbyLine;

#[tauri::command]
pub async fn get_emby_line(id: String, state: tauri::State<'_, AppState>) -> Result<EmbyLine, String> {
    let res = emby_line_mapper::get_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_emby_server_line(emby_server_id: String, state: tauri::State<'_, AppState>) -> Result<Vec<EmbyLine>, String> {
    let res = emby_line_mapper::list_emby_server_line(emby_server_id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_all_emby_line(state: tauri::State<'_, AppState>) -> Result<Vec<EmbyLine>, String> {
    let res = emby_line_mapper::list_all(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_emby_line(body: EmbyLine, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_line_mapper::create(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_emby_line(body: EmbyLine, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_line_mapper::update_by_id(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn delete_emby_line(id: String, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = emby_line_mapper::delete_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}
