use serde::{Deserialize, Serialize};

use crate::{config::app_state::AppState, service::trakt_http_svc};

#[derive(Serialize, Deserialize)]
pub struct TraktHttpTokenParam {
    pub redirect_uri: String,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
}

#[tauri::command]
pub async fn trakt_http_token(body: TraktHttpTokenParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<String, String> {
    let res = trakt_http_svc::token(body, &state, app_handle).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct TraktHttpGetUserInfoParam {
    pub access_token: String,
}

#[tauri::command]
pub async fn trakt_http_get_user_info(body: TraktHttpGetUserInfoParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = trakt_http_svc::get_user_info(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct TraktHttpStartParam {
    pub access_token: String,
    pub body: String,
}

#[tauri::command]
pub async fn trakt_http_start(body: TraktHttpStartParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = trakt_http_svc::start(body, &state, 0).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct TraktHttpStopParam {
    pub access_token: String,
    pub body: String,
}

#[tauri::command]
pub async fn trakt_http_stop(body: TraktHttpStopParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = trakt_http_svc::stop(body, &state, 0).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}
