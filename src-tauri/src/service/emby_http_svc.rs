use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{
    config::{app_state::AppState, http_pool},
    controller::emby_http_ctl::{EmbyAuthenticateByNameParam, EmbyCountParam, EmbyEpisodesParam, EmbyGetAudioStreamUrlParam, EmbyGetContinuePlayListParam, EmbyGetDirectStreamUrlParam, EmbyGetFavoriteListParam, EmbyGetMediaLibraryChildLatestParam, EmbyGetMediaLibraryChildParam, EmbyGetMediaLibraryListParam, EmbyGetServerInfoParam, EmbyGetSubtitleStreamUrlParam, EmbyHideFromResumeParam, EmbyItemsParam, EmbyLogoutParam, EmbyNextUpParam, EmbyPlaybackInfoParam, EmbyPlayedParam, EmbySearchParam, EmbySeasonsParam, EmbyStarParam, EmbyUnplayedParam, EmbyUnstarParam},
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
        .get(format!("{}/emby/Users/{}/Items/Resume?MediaTypes=Video&Recursive=true&Fields=AlternateMediaSources,MediaSources&StartIndex={}&Limit={}", emby_server.base_url.clone().unwrap(), emby_server.user_id.clone().unwrap(), param.start_index, param.limit))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("首页继续播放列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
}

pub async fn items(param: EmbyItemsParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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

    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .get(format!("{}/emby/Shows/{}/Episodes?StartIndex={}&Limit={}&SeasonId={}&StartItemId={}&Fields=AlternateMediaSources,MediaSources&UserId={}", emby_server.base_url.clone().unwrap(), param.series_id, param.start_index, param.limit, param.season_id, param.start_item_id.unwrap_or("".to_string()), emby_server.user_id.clone().unwrap()))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("季下的剧列表 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
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
    Ok(response.text().await?)
}

pub async fn get_direct_stream_url(param: EmbyGetDirectStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let url = format!("{}/emby{}", emby_server.base_url.clone().unwrap(), param.direct_stream_url);
    Ok(url)
}

pub async fn get_audio_stream_url(param: EmbyGetAudioStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    if !param.media_streams_is_external {
        return Err(anyhow::anyhow!("media_streams audio not external"));
    }
    let url = format!("{}/emby/Audio/{}/stream.{}?AudioStreamIndex={}&Static=true", emby_server.base_url.clone().unwrap(), if param.media_source_item_id.is_some() {param.media_source_item_id.unwrap()} else {param.item_id}, param.media_streams_codec, param.media_streams_index);
    Ok(url)
}

pub async fn get_subtitle_stream_url(param: EmbyGetSubtitleStreamUrlParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let emby_server = match emby_server_mapper::get_cache(&param.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    if !param.media_streams_is_external {
        return Err(anyhow::anyhow!("media_streams Subtitles not external"));
    }
    let url = format!("{}/emby/Videos/{}/{}/Subtitles/{}/Stream.{}", emby_server.base_url.clone().unwrap(), if param.media_source_item_id.is_some() {param.media_source_item_id.unwrap()} else {param.item_id}, param.media_source_id, param.media_streams_index, param.media_streams_codec);
    Ok(url)
}

pub fn get_image_url(base_url: &str, item_id: &str, image_type: &str) -> String {
    format!("{}/emby/Items/{}/Images/{}", base_url, item_id, image_type)
}
