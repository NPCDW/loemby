use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tauri::Emitter;

use crate::{
    config::{app_state::{AppState, TauriNotify}, http_pool}, controller::trakt_http_ctl::{TraktHttpGetUserInfoParam, TraktHttpStartParam, TraktHttpStopParam, TraktHttpTokenParam}, mapper::{global_config_mapper, proxy_server_mapper}
};

static TRAKT_TOKEN_EXCHANGE_URL: &str = "https://token-exchange.i101.workers.dev/trakt";
static TRAKT_CLIENT_ID: &str = "05521c50a5a5ac1fb238648a15e8da57ea7c708127e49711303c9b9691913572";

pub async fn token(param: TraktHttpTokenParam, state: &tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> anyhow::Result<String> {
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(TRAKT_TOKEN_EXCHANGE_URL)
        .headers(headers)
        .body(serde_json::json!({
            "code": param.code,
            "refresh_token": param.refresh_token,
            "redirect_uri": param.redirect_uri,
        }).to_string());
    let builder_print = format!("{:?}", &builder);
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
    Ok(response.text().await?)
}

pub async fn get_user_info(param: TraktHttpGetUserInfoParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", param.access_token)).unwrap());
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
    Ok(response.text().await?)
}

pub async fn start(param: TraktHttpStartParam, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", param.access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post("https://api.trakt.tv/scrobble/start")
        .headers(headers)
        .body(param.body.clone());
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("trakt开始播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 429 {
        if retry > 3 {
            return Err(anyhow::anyhow!("Trakt 请求太多或太快，三次重试均失败"));
        }
        tracing::warn!("Trakt 请求太多或太快，开始播放 Api 将在 1 秒钟后重试 {}/3", retry + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(start(param, state, retry + 1)).await;
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}

pub async fn stop(param: TraktHttpStopParam, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", param.access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post("https://api.trakt.tv/scrobble/stop")
        .headers(headers)
        .body(param.body.clone());
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("trakt停止播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if response.status().as_u16() == 429 {
        if retry > 3 {
            return Err(anyhow::anyhow!("Trakt 请求太多或太快，三次重试均失败"));
        }
        tracing::warn!("Trakt 请求太多或太快，开始播放 Api 将在 1 秒钟后重试 {}/3", retry + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(stop(param, state, retry + 1)).await;
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}
