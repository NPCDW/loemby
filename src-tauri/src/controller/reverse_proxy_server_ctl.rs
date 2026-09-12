use crate::config::app_state::AppState;
use crate::mapper::emby_server_mapper;
use crate::mapper::reverse_proxy_server_mapper;
use crate::mapper::reverse_proxy_server_mapper::ReverseProxyServer;

/// 反代服务器变更后，重新加载依赖反代地址的 emby_server 缓存
async fn reload_emby_server_cache(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    emby_server_mapper::load_cache(state).await
}

#[tauri::command]
pub async fn get_reverse_proxy_server(id: String, state: tauri::State<'_, AppState>) -> Result<Option<ReverseProxyServer>, String> {
    let res = reverse_proxy_server_mapper::get_by_id(id, &state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn list_all_reverse_proxy_server(state: tauri::State<'_, AppState>) -> Result<Vec<ReverseProxyServer>, String> {
    let res = reverse_proxy_server_mapper::list_all(&state.db_pool).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[tauri::command]
pub async fn add_reverse_proxy_server(body: ReverseProxyServer, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = reverse_proxy_server_mapper::create(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    if let Err(e) = reload_emby_server_cache(&state).await {
        return Err(e.to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn update_reverse_proxy_server(body: ReverseProxyServer, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = reverse_proxy_server_mapper::update_by_id(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    if let Err(e) = reload_emby_server_cache(&state).await {
        return Err(e.to_string());
    }
    Ok(res.unwrap().rows_affected())
}

#[tauri::command]
pub async fn delete_reverse_proxy_server(id: String, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let res = reverse_proxy_server_mapper::delete_by_id(id, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    if let Err(e) = reload_emby_server_cache(&state).await {
        return Err(e.to_string());
    }
    Ok(res.unwrap().rows_affected())
}
