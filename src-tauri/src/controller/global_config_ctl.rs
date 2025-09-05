use crate::config::app_state::AppState;
use crate::mapper::global_config_mapper;
use crate::mapper::global_config_mapper::GlobalConfig;

#[tauri::command]
pub async fn get_global_config(id: String, state: tauri::State<'_, AppState>) -> Result<GlobalConfig, String> {
    let res = global_config_mapper::get_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_all_global_config(state: tauri::State<'_, AppState>) -> Result<Vec<GlobalConfig>, String> {
    let res = global_config_mapper::list_all(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_global_config(body: GlobalConfig, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = global_config_mapper::create(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_global_config(body: GlobalConfig, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = global_config_mapper::update_by_id(body, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn delete_global_config(id: String, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = global_config_mapper::delete_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}
