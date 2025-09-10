use serde::{Deserialize, Serialize};

use crate::{config::app_state::AppState, service::app_http_svc};

#[derive(Serialize, Deserialize)]
pub struct AppHttpGetProxyLocationParam {
    pub proxy_id: String,
}

#[tauri::command]
pub async fn app_http_get_proxy_location(body: AppHttpGetProxyLocationParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = app_http_svc::get_proxy_location(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct AppHttpGetEmbyIconLibraryParam {
    pub url: String,
}

#[tauri::command]
pub async fn app_http_get_emby_icon_library(body: AppHttpGetEmbyIconLibraryParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = app_http_svc::get_emby_icon_library(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}
