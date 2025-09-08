use crate::config::app_state::AppState;
use crate::mapper::proxy_server_mapper;
use crate::mapper::proxy_server_mapper::ProxyServer;

#[tauri::command]
pub async fn get_proxy_server(id: String, state: tauri::State<'_, AppState>) -> Result<Option<ProxyServer>, String> {
    let res = proxy_server_mapper::get_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_all_proxy_server(state: tauri::State<'_, AppState>) -> Result<Vec<ProxyServer>, String> {
    let res = proxy_server_mapper::list_all(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_proxy_server(body: ProxyServer, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = proxy_server_mapper::create(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_proxy_server(body: ProxyServer, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = proxy_server_mapper::update_by_id(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn delete_proxy_server(id: String, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = proxy_server_mapper::delete_by_id(id, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap().rows_affected())
}
