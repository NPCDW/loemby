use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::{
    config::{app_state::{AppState, TauriNotify}, http_pool}, mapper::{global_config_mapper::{self, GlobalConfig}, proxy_server_mapper}, service::trakt_http_svc::TraktScrobbleParam
};

static SIMKL_WEBSITE_BASE_URL: &str = "https://simkl.com";
static SIMKL_API_BASE_URL: &str = "https://api.simkl.com";
static SIMKL_TOKEN_EXCHANGE_URL: &str = "https://token-exchange.i101.workers.dev/simkl";
static SIMKL_CLIENT_ID: &str = "31fbc68631721a73fdebcfe4987ae1f07ca83363d7f2ffa3f5d6f9f2f836b6df";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimklTokenResponse {
    pub access_token: String,
}

pub async fn save_access_token(response: SimklTokenResponse, state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("simkl_access_token".to_string()),
        config_value: Some(response.access_token.clone()),
        ..Default::default()}, state).await?;
    let user_info = Box::pin(get_user_info(state)).await;
    if user_info.is_err() {
        state.app_handle.emit("tauri_notify", TauriNotify {
            event_type: "SimklNotify".to_string(),
            message_type: "error".to_string(),
            title: None,
            message: format!("获取simkl用户信息错误: {:?}", user_info),
        }).unwrap();
        return Err(anyhow::anyhow!("获取simkl用户信息错误: {:?}", user_info));
    }
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("simkl_username".to_string()),
        config_value: Some(user_info.unwrap().user.name),
        ..Default::default()}, state).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("simkl_sync_switch".to_string()),
        config_value: Some("on".to_string()),
        ..Default::default()}, state).await?;
    Ok(())
}

pub async fn get_cache_access_token(state: &tauri::State<'_, AppState>) -> Option<String> {
    return global_config_mapper::get_cache("simkl_access_token", state).await;
}

#[derive(Serialize, Deserialize)]
pub struct SimklHttpTokenParam {
    pub redirect_uri: String,
    pub code: String,
}

pub async fn token(param: SimklHttpTokenParam, state: &tauri::State<'_, AppState>, app_handle: &tauri::AppHandle) -> anyhow::Result<SimklTokenResponse> {
    let simkl_proxy_id = global_config_mapper::get_cache("simkl_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(simkl_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "code": param.code,
            "redirect_uri": param.redirect_uri,
        }).to_string();
    let builder = client
        .post(SIMKL_TOKEN_EXCHANGE_URL)
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("获取simkl token request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        app_handle.emit("tauri_notify", TauriNotify {
            event_type: "SimklNotify".to_string(),
            message_type: "error".to_string(),
            title: None,
            message: "Simkl 授权失败，请重新授权".to_string(),
        }).unwrap();
        return Err(anyhow::anyhow!("Simkl 授权失败，请重新授权"));
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取simkl token response text {}", text);
    Ok(serde_json::from_str::<SimklTokenResponse>(&text)?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimklUserInfoUserResponse {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimklUserInfoResponse {
    pub user: SimklUserInfoUserResponse,
}

pub async fn get_user_info(state: &tauri::State<'_, AppState>) -> anyhow::Result<SimklUserInfoResponse> {
    let access_token = get_cache_access_token(state).await.ok_or(anyhow::anyhow!("Failed to get access token"))?;
    let simkl_proxy_id = global_config_mapper::get_cache("simkl_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(simkl_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("simkl-api-key").unwrap(), HeaderValue::from_str(SIMKL_CLIENT_ID).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/users/settings", SIMKL_API_BASE_URL))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取simkl用户信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取simkl用户信息 response text {}", text);
    Ok(serde_json::from_str::<SimklUserInfoResponse>(&text)?)
}

pub async fn start(body: &TraktScrobbleParam, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let access_token = get_cache_access_token(state).await.ok_or(anyhow::anyhow!("Failed to get access token"))?;
    let simkl_proxy_id = global_config_mapper::get_cache("simkl_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(simkl_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("simkl-api-key").unwrap(), HeaderValue::from_str(SIMKL_CLIENT_ID).unwrap());

    let body_str = serde_json::to_string(body)?;
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/scrobble/start", SIMKL_API_BASE_URL))
        .headers(headers)
        .body(body_str.clone());
    let builder_print = format!("{:?} {}", &builder, body_str);
    let response = builder.send().await;
    tracing::debug!("simkl开始播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 429 {
        if retry > 3 {
            return Err(anyhow::anyhow!("Simkl 请求太多或太快，三次重试均失败"));
        }
        tracing::warn!("Simkl 请求太多或太快，开始播放 Api 将在 1 秒钟后重试 {}/3", retry + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(start(body, state, retry + 1)).await;
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("simkl开始播放 response text {}", text);
    Ok(text)
}

pub async fn stop(body: &TraktScrobbleParam, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let access_token = get_cache_access_token(state).await.ok_or(anyhow::anyhow!("Failed to get access token"))?;
    let simkl_proxy_id = global_config_mapper::get_cache("simkl_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(simkl_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("simkl-api-key").unwrap(), HeaderValue::from_str(SIMKL_CLIENT_ID).unwrap());

    let body_str = serde_json::to_string(body)?;
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/scrobble/stop", SIMKL_API_BASE_URL))
        .headers(headers)
        .body(body_str.clone());
    let builder_print = format!("{:?} {}", &builder, body_str);
    let response = builder.send().await;
    tracing::debug!("simkl停止播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 429 {
        if retry > 3 {
            return Err(anyhow::anyhow!("Simkl 请求太多或太快，三次重试均失败"));
        }
        tracing::warn!("Simkl 请求太多或太快，开始播放 Api 将在 1 秒钟后重试 {}/3", retry + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(stop(body, state, retry + 1)).await;
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("simkl停止播放 response text {}", text);
    Ok(text)
}

pub async fn go_simkl_auth(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let auxm_app_state = state.auxm_app_state.read().await.clone().unwrap();

    let redirect_uri = format!("http://127.0.0.1:{}/simkl_auth", auxm_app_state.port);
    let state = uuid::Uuid::new_v4().to_string();
    let url = format!("{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&state={}", SIMKL_WEBSITE_BASE_URL, SIMKL_CLIENT_ID, redirect_uri, state);
    auxm_app_state.simkl_auth_state.write().await.push(state);
    let res = open::that(&url);
    if let Err(err) = res {
        return Err(anyhow::anyhow!("打开浏览器失败: {} 您可尝试手动复制链接到浏览器中打开 {}", err.to_string(), &url));
    }
    tracing::debug!("打开浏览器成功: {}", &url);
    Ok(())
}
