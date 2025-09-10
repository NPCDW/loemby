use reqwest::header::{HeaderMap, HeaderValue};

use crate::{
    config::{app_state::AppState, http_pool}, controller::app_http_ctl::{AppHttpGetEmbyIconLibraryParam, AppHttpGetProxyLocationParam}, mapper::{global_config_mapper, proxy_server_mapper}
};

pub async fn get_proxy_location(param: AppHttpGetProxyLocationParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let proxy_url = proxy_server_mapper::get_cache(param.proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get("https://api.my-ip.io/v2/ip.json")
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取代理服务器IP信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}

pub async fn get_emby_icon_library(param: AppHttpGetEmbyIconLibraryParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let app_proxy_id = global_config_mapper::get_cache("app_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(app_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(param.url)
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取代理服务器IP信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}
