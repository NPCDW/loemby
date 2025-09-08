use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::{
    config::{app_state::AppState, http_pool},
    controller::{emby_http_ctl::{EmbyAuthenticateByNameParam, EmbyGetServerInfoParam}},
    mapper::{emby_server_mapper, proxy_server_mapper}
};

pub async fn get_server_info(param: EmbyGetServerInfoParam, state: tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(param.emby_server_id, &state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, &state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(&emby_server.base_url.as_ref().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/System/Info/Public", emby_server.base_url.as_ref().unwrap()))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("reqwest response {} {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}

pub async fn authenticate_by_name(param: EmbyAuthenticateByNameParam, state: tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(param.emby_server_id, &state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, &state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(&emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Authorization").unwrap(), HeaderValue::from_str(&format!(r#"Emby Client="{}", Device="{}", DeviceId="{}", Version="{}""#, emby_server.client.unwrap(), emby_server.device.unwrap(), emby_server.device_id.unwrap(), emby_server.client_version.unwrap())).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/emby/Users/AuthenticateByName", emby_server.base_url.as_ref().unwrap()))
        .headers(headers)
        .body(serde_json::json!({
            "Username": emby_server.username,
            "Pw": emby_server.password,
        }).to_string());
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("reqwest response {} {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
}
