use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{
    config::{app_state::AppState, http_pool},
    controller::emby_http_ctl::{EmbyAuthenticateByNameParam, EmbyCountParam, EmbyEpisodesParam, EmbyGetContinuePlayListParam, EmbyGetFavoriteListParam, EmbyGetMediaLibraryChildLatestParam, EmbyGetMediaLibraryChildParam, EmbyGetMediaLibraryListParam, EmbyGetServerInfoParam, EmbyHideFromResumeParam, EmbyItemsParam, EmbyLogoutParam, EmbyNextUpParam, EmbyPlaybackInfoParam, EmbyPlayedParam, EmbySearchParam, EmbySeasonsParam, EmbyStarParam, EmbyUnplayedParam, EmbyUnstarParam},
    mapper::{emby_server_mapper, global_config_mapper, proxy_server_mapper}
};

pub async fn get_server_info(param: EmbyGetServerInfoParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/System/Info/Public", emby_server.base_url.clone().unwrap()))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取emby服务器信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取emby服务器信息 response text {}", text);
    Ok(text)
}

pub async fn authenticate_by_name(param: EmbyAuthenticateByNameParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Authorization").unwrap(), HeaderValue::from_str(&format!(r#"Emby Client="{}", Device="{}", DeviceId="{}", Version="{}""#, emby_server.client.unwrap(), emby_server.device.unwrap(), emby_server.device_id.unwrap(), emby_server.client_version.unwrap())).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "Username": emby_server.username,
            "Pw": emby_server.password,
        }).to_string();
    let builder = client
        .post(format!("{}/emby/Users/AuthenticateByName", emby_server.base_url.clone().unwrap()))
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("登录emby服务器 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("登录emby服务器 response text {}", text);
    Ok(text)
}

pub async fn logout(param: EmbyLogoutParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/emby/Sessions/Logout", emby_server.base_url.clone().unwrap()))
        .headers(headers)
        .body(serde_json::json!({}).to_string());
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("登出emby服务器 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("登出emby服务器 response text {}", text);
    Ok(text)
}

pub async fn search(param: EmbySearchParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Items?SearchTerm={}&IncludeItemTypes={}&Recursive=true&Fields=AlternateMediaSources,MediaSources,ProductionYear,EndDate&StartIndex={}&Limit={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.search_str, param.item_types.join(","), param.start_index, param.limit))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("搜索媒体库 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("搜索媒体库 response text {}", text);
    Ok(text)
}

pub async fn get_continue_play_list(param: EmbyGetContinuePlayListParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Items/Resume?MediaTypes=Video&Recursive=true&StartIndex={}&Limit={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.start_index, param.limit))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("首页继续播放列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("首页继续播放列表 response text {}", text);
    Ok(text)
}

pub async fn get_favorite_list(param: EmbyGetFavoriteListParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Items?Filters=IsFavorite&Recursive=true&IncludeItemTypes=Episode,Series,Movie,Season&Fields=AlternateMediaSources,MediaSources,ProductionYear,EndDate,Overview&StartIndex={}&Limit={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.start_index, param.limit))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("收藏列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("收藏列表 response text {}", text);
    Ok(text)
}

pub async fn next_up(param: EmbyNextUpParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Shows/NextUp?UserId={}&SeriesId={}&StartIndex={}&Limit={}&Fields=AlternateMediaSources,MediaSources", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.series_id, param.start_index, param.limit))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("接下来播放的剧集 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("接下来播放的剧集 response text {}", text);
    Ok(text)
}

pub async fn get_media_library_list(param: EmbyGetMediaLibraryListParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Views", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap()))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("首页媒体库列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("首页媒体库列表 response text {}", text);
    Ok(text)
}

pub async fn get_media_library_child_latest(param: EmbyGetMediaLibraryChildLatestParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Items/Latest?Limit={}&ParentId={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.limit, param.parent_id))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("首页媒体库子项目最新几条 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("首页媒体库子项目最新几条 response text {}", text);
    Ok(text)
}

pub async fn get_media_library_child(param: EmbyGetMediaLibraryChildParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Items?Recursive=true&IncludeItemTypes=Series,Movie&ParentId={}&StartIndex={}&Limit={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.parent_id, param.start_index, param.limit))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("首页媒体库子项目 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("首页媒体库子项目 response text {}", text);
    Ok(text)
}

pub async fn count(param: EmbyCountParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Items/Counts?UserId={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap()))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("剧集数量统计 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("剧集数量统计 response text {}", text);
    Ok(text)
}

pub async fn items(param: EmbyItemsParam, state: &tauri::State<'_, AppState>, use_cache: bool) -> anyhow::Result<String> {
    let cache_key = format!("{}-{}", param.emby_server_id, param.item_id);
    if use_cache {
        if let Some(cache) = state.emby_http_cache.read().await.get(&cache_key) {
            return Ok(cache.clone());
        }
    }
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Users/{}/Items/{}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.item_id))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("item详情 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("item详情 response text {}", text);
    state.emby_http_cache.write().await.insert(cache_key, text.clone());
    Ok(text)
}

pub async fn seasons(param: EmbySeasonsParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Shows/{}/Seasons?Fields=ProductionYear,Overview&UserId={}", emby_server.base_url.clone().unwrap(), param.series_id, emby_server.user_id.clone().unwrap()))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("剧下的季列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("剧下的季列表 response text {}", text);
    Ok(text)
}

pub async fn episodes(param: EmbyEpisodesParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let fields = if let Some(false) = param.extend_fields { "".to_string() } else { "&Fields=AlternateMediaSources,MediaSources".to_string() };
    let start_index = if let Some(start_index) = param.start_index { format!("&StartIndex={}", start_index) } else { "".to_string() };
    let limit = if let Some(limit) = param.limit { format!("&Limit={}", limit) } else { "".to_string() };
    let start_item_id = if let Some(start_item_id) = param.start_item_id { format!("&StartItemId={}", start_item_id) } else { "".to_string() };
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Shows/{}/Episodes?SeasonId={}&UserId={}{}{}{}{}", emby_server.base_url.clone().unwrap(), param.series_id, param.season_id, emby_server.user_id.clone().unwrap(), start_index, limit, start_item_id, fields))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("季下的剧列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("季下的剧列表 response text {}", text);
    Ok(text)
}

pub async fn playback_info(param: EmbyPlaybackInfoParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let is_playback = global_config_mapper::get_cache("play_param_IsPlayback", state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "UserId": emby_server.user_id.clone().unwrap(),
            "IsPlayback": Some("true".to_string()) == is_playback || None == is_playback,
            "MaxStreamingBitrate": 1400000000,
            "MaxStaticBitrate": 1400000000,
            "MusicStreamingTranscodingBitrate": 1920000,
            "EnableDirectPlay": true,
            "EnableDirectStream": true,
            "EnableTranscoding": false,
            "DeviceProfile": {
                "DirectPlayProfiles": [
                    {
                        "Container": "",
                        "Type": "Video"
                    },
                    {
                        "Container": "",
                        "Type": "Audio"
                    }
                ]
            }
        }).to_string();
    let builder = client
        .post(format!("{}/emby/Items/{}/PlaybackInfo", emby_server.base_url.clone().unwrap(), param.item_id))
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("播放流媒体详情 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("播放流媒体详情 response text {}", text);
    Ok(text)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyPlayingParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub play_session_id: String,
    pub position_ticks: u64,
}

pub async fn playing(param: EmbyPlayingParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "ItemId": param.item_id,
            "MediaSourceId": param.media_source_id,
            "PlayMethod": "DirectStream",
            "PlaySessionId": param.play_session_id,
            "PositionTicks": param.position_ticks,
        }).to_string();
    let builder = client
        .post(format!("{}/emby/Sessions/Playing?ItemId={}&MediaSourceId={}&PlayMethod=DirectStream&PlaySessionId={}&PositionTicks={}", emby_server.base_url.clone().unwrap(), param.item_id, param.media_source_id, param.play_session_id, param.position_ticks))
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("开始播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("开始播放 response text {}", text);
    Ok(text)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyPlayingProgressParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub play_session_id: String,
    pub position_ticks: u64,
}

pub async fn playing_progress(param: EmbyPlayingProgressParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "ItemId": param.item_id,
            "MediaSourceId": param.media_source_id,
            "PlayMethod": "DirectStream",
            "PlaySessionId": param.play_session_id,
            "PositionTicks": param.position_ticks,
        }).to_string();
    let builder = client
        .post(format!("{}/emby/Sessions/Playing/Progress?ItemId={}&MediaSourceId={}&PlayMethod=DirectStream&PlaySessionId={}&PositionTicks={}", emby_server.base_url.clone().unwrap(), param.item_id, param.media_source_id, param.play_session_id, param.position_ticks))
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("播放进度 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("播放进度 response text {}", text);
    Ok(text)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyPlayingStoppedParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub play_session_id: String,
    pub position_ticks: u64,
}

pub async fn playing_stopped(param: EmbyPlayingStoppedParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "ItemId": param.item_id,
            "MediaSourceId": param.media_source_id,
            "PlayMethod": "DirectStream",
            "PlaySessionId": param.play_session_id,
            "PositionTicks": param.position_ticks,
        }).to_string();
    let builder = client
        .post(format!("{}/emby/Sessions/Playing/Stopped?ItemId={}&MediaSourceId={}&PlayMethod=DirectStream&PlaySessionId={}&PositionTicks={}", emby_server.base_url.clone().unwrap(), param.item_id, param.media_source_id, param.play_session_id, param.position_ticks))
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("播放停止 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("播放停止 response text {}", text);
    Ok(text)
}

pub async fn star(param: EmbyStarParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/emby/Users/{}/FavoriteItems/{}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.item_id))
        .headers(headers)
        .body(serde_json::json!({}).to_string());
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("收藏 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("收藏 response text {}", text);
    Ok(text)
}

pub async fn unstar(param: EmbyUnstarParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .delete(format!("{}/emby/Users/{}/FavoriteItems/{}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.item_id))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("取消收藏 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("取消收藏 response text {}", text);
    Ok(text)
}

pub async fn played(param: EmbyPlayedParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/emby/Users/{}/PlayedItems/{}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.item_id))
        .headers(headers)
        .body(serde_json::json!({}).to_string());
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("标记已播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("标记已播放 response text {}", text);
    Ok(text)
}

pub async fn unplayed(param: EmbyUnplayedParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server: emby_server_mapper::EmbyServer = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .delete(format!("{}/emby/Users/{}/PlayedItems/{}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.item_id))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("取消已播放 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("取消已播放 response text {}", text);
    Ok(text)
}

pub async fn hide_from_resume(param: EmbyHideFromResumeParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(emby_server.user_agent.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::REFERER, HeaderValue::from_str(emby_server.base_url.as_ref().unwrap()).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("X-Emby-Token").unwrap(), HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let body = serde_json::json!({
            "Hide": param.hide,
        }).to_string();
    let builder = client
        .post(format!("{}/emby/Users/{}/Items/{}/HideFromResume", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.item_id))
        .headers(headers)
        .body(body.clone());
    let builder_print = format!("{:?} {}", &builder, body);
    let response = builder.send().await;
    tracing::debug!("隐藏观看记录 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("隐藏观看记录 response text {}", text);
    Ok(text)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetDirectStreamUrlParam {
    pub emby_server_id: String,
    pub direct_stream_url: String,
}

pub async fn get_direct_stream_url(param: EmbyGetDirectStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let url = format!("{}/emby{}", emby_server.base_url.clone().unwrap(), param.direct_stream_url);
    tracing::debug!("拼接播放地址 {}", url);
    Ok(url)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetVideoStreamUrlParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub container: String,
    pub media_source_id: String,
    pub play_session_id: String,
}

pub async fn get_video_stream_url(param: EmbyGetVideoStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let url = format!("{}/emby/Videos/{}/stream.{}?Static=true&mediaSourceId={}&playSessionId={}", emby_server.base_url.clone().unwrap(), param.item_id, param.container, param.media_source_id, param.play_session_id);
    tracing::debug!("拼接视频地址 {}", url);
    Ok(url)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetAudioStreamUrlParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_item_id: Option<String>,
    pub media_streams_codec: Option<String>,
    pub media_streams_index: u32,
    pub media_streams_is_external: bool,
}

pub async fn get_audio_stream_url(param: EmbyGetAudioStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    if !param.media_streams_is_external {
        return Err(anyhow::anyhow!("media_streams audio not external"));
    }
    let url = format!("{}/emby/Audio/{}/stream.{}?AudioStreamIndex={}&Static=true", emby_server.base_url.clone().unwrap(), if param.media_source_item_id.is_some() {param.media_source_item_id.unwrap()} else {param.item_id}, param.media_streams_codec.unwrap_or("flac".to_string()), param.media_streams_index);
    tracing::debug!("拼接音频地址 {}", url);
    Ok(url)
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetSubtitleStreamUrlParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub media_source_item_id: Option<String>,
    pub media_streams_codec: Option<String>,
    pub media_streams_index: u32,
    pub media_streams_is_external: bool,
}

pub async fn get_subtitle_stream_url(param: EmbyGetSubtitleStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    if !param.media_streams_is_external {
        return Err(anyhow::anyhow!("media_streams Subtitles not external"));
    }
    let url = format!("{}/emby/Videos/{}/{}/Subtitles/{}/Stream.{}", emby_server.base_url.clone().unwrap(), if param.media_source_item_id.is_some() {param.media_source_item_id.unwrap()} else {param.item_id}, param.media_source_id, param.media_streams_index, param.media_streams_codec.unwrap_or("flac".to_string()));
    tracing::debug!("拼接字幕地址 {}", url);
    Ok(url)
}

pub fn get_image_url(base_url: &str, item_id: &str, image_type: &str) -> String {
    format!("{}/emby/Items/{}/Images/{}", base_url, item_id, image_type)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbyPageList<T> {
    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: u32,
    #[serde(rename = "Items")]
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeItem {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub type_: String,
    #[serde(rename = "SeriesId")]
    pub series_id: Option<String>,
    #[serde(rename = "SeriesName")]
    pub series_name: Option<String>,
    #[serde(rename = "SeasonId")]
    pub season_id: Option<String>,
    #[serde(rename = "SeasonName")]
    pub season_name: Option<String>,
    #[serde(rename = "ProviderIds")]
    pub provider_ids: Option<HashMap<String, String>>,
    #[serde(rename = "ExternalUrls")]
    pub external_urls: Option<Vec<ExternalUrl>>,
    #[serde(rename = "IndexNumber")]
    pub index_number: Option<u32>,
    #[serde(rename = "ParentIndexNumber")]
    pub parent_index_number: Option<u32>,
    #[serde(rename = "ProductionYear")]
    pub production_year: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesItem {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub type_: String,
    #[serde(rename = "ProviderIds")]
    pub provider_ids: Option<HashMap<String, String>>,
    #[serde(rename = "ExternalUrls")]
    pub external_urls: Option<Vec<ExternalUrl>>,
}

// 定义外部链接结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUrl {
    #[serde(rename = "Url")]
    pub url: String,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackInfo {
    #[serde(rename = "PlaySessionId")]
    pub play_session_id: String,

    #[serde(rename = "MediaSources")]
    pub media_sources: Vec<MediaSource>,

    #[serde(rename = "ErrorCode")]
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSource {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "ItemId")]
    pub item_id: Option<String>,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "RunTimeTicks")]
    pub run_time_ticks: Option<u64>,

    #[serde(rename = "Size")]
    pub size: Option<u64>,

    #[serde(rename = "Bitrate")]
    pub bitrate: Option<u64>,

    #[serde(rename = "DirectStreamUrl")]
    pub direct_stream_url: Option<String>,

    #[serde(rename = "MediaStreams")]
    pub media_streams: Vec<MediaStream>,

    #[serde(rename = "IsRemote")]
    pub is_remote: Option<bool>,

    #[serde(rename = "Path")]
    pub path: Option<String>,
    
    #[serde(rename = "Container")]
    pub container: String,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStream {
    #[serde(rename = "Codec")]
    pub codec: Option<String>,

    #[serde(rename = "DisplayTitle")]
    pub display_title: Option<String>,

    #[serde(rename = "DisplayLanguage")]
    pub display_language: Option<String>,

    #[serde(rename = "Title")]
    pub title: Option<String>,

    #[serde(rename = "BitRate")]
    pub bit_rate: Option<u32>,

    #[serde(rename = "Height")]
    pub height: Option<u32>, // 有些媒体流（如音频）可能没有高度

    #[serde(rename = "Width")]
    pub width: Option<u32>, // 同上

    #[serde(rename = "Type")]
    pub type_: String,

    #[serde(rename = "Language")]
    pub language: Option<String>,

    #[serde(rename = "Index")]
    pub index: u32,

    #[serde(rename = "IsDefault")]
    pub is_default: Option<bool>,

    #[serde(rename = "IsExternal")]
    pub is_external: Option<bool>,
}