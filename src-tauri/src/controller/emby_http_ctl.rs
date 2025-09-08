use serde::{Deserialize, Serialize};

use crate::{config::app_state::AppState, http::emby_http};


#[derive(Serialize, Deserialize)]
pub struct EmbyGetServerInfoParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_get_server_info(body: EmbyGetServerInfoParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http::get_server_info(body, state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyAuthenticateByNameParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_authenticate_by_name(body: EmbyAuthenticateByNameParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http::authenticate_by_name(body, state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}
