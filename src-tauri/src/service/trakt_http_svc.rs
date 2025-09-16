use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::{
    config::{app_state::{AppState, TauriNotify}, http_pool}, mapper::{global_config_mapper::{self, GlobalConfig}, proxy_server_mapper}
};

static TRAKT_TOKEN_EXCHANGE_URL: &str = "https://token-exchange.i101.workers.dev/trakt";
static TRAKT_CLIENT_ID: &str = "05521c50a5a5ac1fb238648a15e8da57ea7c708127e49711303c9b9691913572";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraktTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub created_at: u64,
}

pub async fn save_access_token(response: TraktTokenResponse, redirect_uri: String, state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("trakt_access_token".to_string()),
        config_value: Some(response.access_token.clone()),
        ..Default::default()}, state).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("trakt_refresh_token".to_string()),
        config_value: Some(response.refresh_token),
        ..Default::default()}, state).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("trakt_expires_in".to_string()),
        config_value: Some((response.expires_in + response.created_at).to_string()),
        ..Default::default()}, state).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("trakt_redirect_uri".to_string()),
        config_value: Some(redirect_uri),
        ..Default::default()}, state).await?;
    let user_info = Box::pin(get_user_info(state)).await;
    if user_info.is_err() {
        state.app_handle.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessage".to_string(),
            message_type: "error".to_string(),
            title: None,
            message: format!("获取trakt用户信息错误: {:?}", user_info),
        }).unwrap();
        return Err(anyhow::anyhow!("获取trakt用户信息错误: {:?}", user_info));
    }
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("trakt_username".to_string()),
        config_value: Some(user_info.unwrap().user.username),
        ..Default::default()}, state).await?;
    Ok(())
}

pub async fn get_cache_access_token(state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let trakt_expires_in = global_config_mapper::get_cache("trakt_expires_in", state).await;
    if trakt_expires_in.is_none() {
        state.app_handle.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessage".to_string(),
            message_type: "error".to_string(),
            title: None,
            message: format!("Trakt 未授权，或授权失败"),
        }).unwrap();
        return Err(anyhow::anyhow!("Trakt 未授权，或授权失败"));
    }
    let trakt_expires_in = trakt_expires_in.unwrap().parse::<i64>().unwrap();
    let current_time = chrono::Local::now().timestamp();
    if current_time < (trakt_expires_in - 6 * 60 * 60) {
        return Ok(global_config_mapper::get_cache("trakt_access_token", state).await.unwrap());
    } else {
        tracing::info!("Trakt access token 已过期，重新获取");
        let trakt_refresh_token = global_config_mapper::get_cache("trakt_refresh_token", state).await;
        let trakt_redirect_uri = global_config_mapper::get_cache("trakt_redirect_uri", state).await;
        let response = token(TraktHttpTokenParam {
            code: None,
            refresh_token: Some(trakt_refresh_token.unwrap()),
            redirect_uri: trakt_redirect_uri.clone().unwrap(),
        }, state, &state.app_handle.clone()).await?;
        save_access_token(response.clone(), trakt_redirect_uri.unwrap(), state).await?;
        return Ok(response.access_token);
    }
}

#[derive(Serialize, Deserialize)]
pub struct TraktHttpTokenParam {
    pub redirect_uri: String,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
}

pub async fn token(param: TraktHttpTokenParam, state: &tauri::State<'_, AppState>, app_handle: &tauri::AppHandle) -> anyhow::Result<TraktTokenResponse> {
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "code": param.code,
            "refresh_token": param.refresh_token,
            "redirect_uri": param.redirect_uri,
        }).to_string();
    let builder = client
        .post(TRAKT_TOKEN_EXCHANGE_URL)
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("获取trakt token request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 401 {
        app_handle.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessageBox".to_string(),
            message_type: "warning".to_string(),
            title: None,
            message: "您的 Trakt 授权好像失效了，或许应该重新授权".to_string(),
        }).unwrap();
        return Err(anyhow::anyhow!("您的 Trakt 授权好像失效了，或许应该重新授权"));
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.json::<TraktTokenResponse>().await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraktUserInfoUserResponse {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraktUserInfoResponse {
    pub user: TraktUserInfoUserResponse,
}

pub async fn get_user_info(state: &tauri::State<'_, AppState>) -> anyhow::Result<TraktUserInfoResponse> {
    let access_token = get_cache_access_token(state).await?;
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get("https://api.trakt.tv/users/settings")
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取trakt用户信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.json::<TraktUserInfoResponse>().await?)
}

pub async fn start(body: String, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let access_token = get_cache_access_token(state).await?;
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post("https://api.trakt.tv/scrobble/start")
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("trakt开始播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 429 {
        if retry > 3 {
            return Err(anyhow::anyhow!("Trakt 请求太多或太快，三次重试均失败"));
        }
        tracing::warn!("Trakt 请求太多或太快，开始播放 Api 将在 1 秒钟后重试 {}/3", retry + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(start(body, state, retry + 1)).await;
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}

pub async fn stop(body: String, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let access_token = get_cache_access_token(state).await?;
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post("https://api.trakt.tv/scrobble/stop")
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("trakt停止播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 429 {
        if retry > 3 {
            return Err(anyhow::anyhow!("Trakt 请求太多或太快，三次重试均失败"));
        }
        tracing::warn!("Trakt 请求太多或太快，开始播放 Api 将在 1 秒钟后重试 {}/3", retry + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(stop(body, state, retry + 1)).await;
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}
