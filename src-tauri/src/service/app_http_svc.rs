use reqwest::header::{HeaderMap, HeaderValue};

use crate::{
    config::{app_state::AppState, http_pool}, controller::app_http_ctl::{AppHttpGetEmbyIconLibraryParam, AppHttpGetProxyLocationParam, AppHttpGetReverseProxyLocationParam}, mapper::{global_config_mapper, proxy_server_mapper, reverse_proxy_server_mapper}
};

pub async fn get_proxy_location(param: AppHttpGetProxyLocationParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let proxy_url = proxy_server_mapper::get_cache(param.proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get("https://api.ip.sb/geoip")
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取代理服务器IP信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取代理服务器IP信息 response text {}", text);
    Ok(text)
}

/// 检测反代服务器可用性：使用反代地址拼接 IP 查询接口，获取出口 IP 与地区
pub async fn get_reverse_proxy_location(param: AppHttpGetReverseProxyLocationParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let reverse_proxy_url = reverse_proxy_server_mapper::get_reverse_proxy_url(Some(param.reverse_proxy_id), state).await
        .ok_or_else(|| anyhow::anyhow!("反代服务器地址为空，无法检测"))?;

    // 拼接地址：反代地址 + 原始地址（反代地址已在缓存中规范化为以 / 结尾）
    let target_url = format!("{}{}", reverse_proxy_url, "https://api.ip.sb/geoip");

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());

    let client = http_pool::get_api_http_client(None, state).await?;
    let builder = client
        .get(&target_url)
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取反代服务器IP信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取反代服务器IP信息 response text {}", text);
    Ok(text)
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
    tracing::debug!("获取emby图标库 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取emby图标库 response text {}", text);
    Ok(text)
}
