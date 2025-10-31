use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::{
    config::{app_state::{AppState, TauriNotify}, http_pool}, mapper::{global_config_mapper::{self, GlobalConfig}, proxy_server_mapper}, service::emby_http_svc::{EpisodeItem, ExternalUrl, SeriesItem}
};

static TRAKT_WEBSITE_BASE_URL: &str = "https://trakt.tv";
static TRAKT_API_BASE_URL: &str = "https://api.trakt.tv";
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
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("trakt_sync_switch".to_string()),
        config_value: Some("on".to_string()),
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
    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
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
    let text = response.text().await?;
    tracing::debug!("获取trakt token response text {}", text);
    Ok(serde_json::from_str::<TraktTokenResponse>(&text)?)
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
        .get(format!("{}/users/settings", TRAKT_API_BASE_URL))
        .headers(headers);
    let builder_print = format!("{:?}", &builder);
    let response = builder.send().await;
    tracing::debug!("获取trakt用户信息 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("获取trakt用户信息 response text {}", text);
    Ok(serde_json::from_str::<TraktUserInfoResponse>(&text)?)
}

pub async fn start(body: &ScrobbleParam, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let access_token = get_cache_access_token(state).await?;
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let body_str = serde_json::to_string(body)?;
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/scrobble/start", TRAKT_API_BASE_URL))
        .headers(headers)
        .body(body_str.clone());
    let builder_print = format!("{:?} {}", &builder, body_str);
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
    let text = response.text().await?;
    tracing::debug!("trakt开始播放 response text {}", text);
    Ok(text)
}

pub async fn stop(body: &ScrobbleParam, state: &tauri::State<'_, AppState>, retry: u32) -> anyhow::Result<String> {
    let access_token = get_cache_access_token(state).await?;
    let trakt_proxy_id = global_config_mapper::get_cache("trakt_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(trakt_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_str("application/json; charset=UTF-8").unwrap());
    headers.insert(HeaderName::from_str("Authorization").unwrap(), HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(HeaderName::from_str("trakt-api-version").unwrap(), HeaderValue::from_str("2").unwrap());
    headers.insert(HeaderName::from_str("trakt-api-key").unwrap(), HeaderValue::from_str(TRAKT_CLIENT_ID).unwrap());

    let body_str = serde_json::to_string(body)?;
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(format!("{}/scrobble/stop", TRAKT_API_BASE_URL))
        .headers(headers)
        .body(body_str.clone());
    let builder_print = format!("{:?} {}", &builder, body_str);
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
    let text = response.text().await?;
    tracing::debug!("trakt停止播放 response text {}", text);
    Ok(text)
}

pub async fn go_trakt_auth(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let auxm_app_state = state.auxm_app_state.read().await.clone().unwrap();

    let redirect_uri = format!("http://127.0.0.1:{}/trakt_auth", auxm_app_state.port);
    let state = uuid::Uuid::new_v4().to_string();
    let url = format!("{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&state={}", TRAKT_WEBSITE_BASE_URL, TRAKT_CLIENT_ID, redirect_uri, state);
    auxm_app_state.trakt_auth_state.write().await.push(state);
    let res = webbrowser::open(&url);
    if let Err(err) = res {
        return Err(anyhow::anyhow!("打开浏览器失败: {} 您可尝试手动复制链接到浏览器中打开 {}", err.to_string(), &url));
    }
    tracing::debug!("打开浏览器成功: {}", &url);
    Ok(())
}

pub fn get_scrobble_trakt_param(episode: &EpisodeItem, series: &Option<SeriesItem>, progress: f64) -> Option<ScrobbleParam> {
    if episode.type_ == "Movie" {
        let ids = get_scrobble_trakt_ids_param(&episode.provider_ids, &episode.external_urls);
        if has_valid_ids(&ids) {
            return Some(ScrobbleParam {
                movie: Some(ScrobbleIdsParam {ids}),
                progress: progress,
                ..Default::default()
            });
        }
    } else if episode.type_ == "Episode" {
        let ids = get_scrobble_trakt_ids_param(&episode.provider_ids, &episode.external_urls);
        if has_valid_ids(&ids) {
            return Some(ScrobbleParam {
                episode: Some(ScrobbleEpisodeParam {ids: Some(ids), ..Default::default()}),
                progress: progress,
                ..Default::default()
            });
        } else if let (Some(series), Some(index_number), Some(parent_index_number)) = (
            series,
            episode.index_number,
            episode.parent_index_number,
        ) {
            let series_ids = get_scrobble_trakt_ids_param(&series.provider_ids, &series.external_urls);
            if has_valid_ids(&series_ids) {
                return Some(ScrobbleParam {
                    show: Some(ScrobbleIdsParam {ids: series_ids}),
                    episode: Some(ScrobbleEpisodeParam {
                        season: Some(parent_index_number),
                        number: Some(index_number),
                        ..Default::default()}),
                    progress: progress,
                    ..Default::default()
                });
            }
        }
    }
    None
}

// 检查是否有有效的 ID
fn has_valid_ids(ids: &TraktIds) -> bool {
    ids.imdb.is_some() || ids.tmdb.is_some() || ids.tvdb.is_some() || ids.trakt.is_some()
}

// 从 MediaItem 获取 Trakt IDs
fn get_scrobble_trakt_ids_param(provider_ids: &Option<HashMap<String, String>>, external_urls: &Option<Vec<ExternalUrl>>) -> TraktIds {
    let mut ids = TraktIds::default();
    
    // 从 ProviderIds 提取
    if let Some(provider_ids) = provider_ids {
        for (key, value) in provider_ids {
            match key.to_lowercase().as_str() {
                "imdb" => ids.imdb = Some(value.clone()),
                "tmdb" => ids.tmdb = Some(value.clone()),
                "tvdb" => ids.tvdb = Some(value.clone()),
                "trakt" => ids.trakt = Some(value.clone()),
                _ => {}
            }
        }
    }
    
    // 从 ExternalUrls 提取
    if let Some(external_urls) = external_urls {
        for external_url in external_urls {
            let url_str = external_url.url.to_string();
            if let Ok(url) = url::Url::parse(&url_str) {
                // IMDb
                if url_str.starts_with("https://www.imdb.com") {
                    if !url.path().ends_with('/') && ids.imdb.is_none() {
                        if let Some(last_part) = url.path().split('/').last() {
                            ids.imdb = Some(last_part.to_string());
                        }
                    }
                }
                // TMDb
                else if url_str.starts_with("https://www.themoviedb.org") {
                    if !url.path().ends_with('/') && ids.tmdb.is_none() {
                        if let Some(last_part) = url.path().split('/').last() {
                            ids.tmdb = Some(last_part.to_string());
                        }
                    }
                }
                // TVDB
                else if url_str.starts_with("https://thetvdb.com") {
                    if ids.tvdb.is_none() {
                        if let Some(id) = url.query_pairs().find(|(k, _)| k == "id") {
                            ids.tvdb = Some(id.1.to_string());
                        }
                    }
                }
                // Trakt
                else if url_str.starts_with("https://trakt.tv/search/") {
                    let path_segments: Vec<&str> = url.path().split('/').collect();
                    if path_segments.len() == 4 {
                        let provider = path_segments[2];
                        let id_value = path_segments[3];
                        
                        match provider {
                            "imdb" if ids.imdb.is_none() => ids.imdb = Some(id_value.to_string()),
                            "tmdb" if ids.tmdb.is_none() => ids.tmdb = Some(id_value.to_string()),
                            "tvdb" if ids.tvdb.is_none() => ids.tvdb = Some(id_value.to_string()),
                            "trakt" if ids.trakt.is_none() => ids.trakt = Some(id_value.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    ids
}

// 定义 Trakt ID 类型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TraktIds {
    #[serde(skip_serializing_if = "Option::is_none")]
    imdb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tmdb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tvdb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trakt: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ScrobbleEpisodeParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<TraktIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    season: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScrobbleIdsParam {
    ids: TraktIds,
}

// 定义 Scrobble 参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct  ScrobbleParam {
    pub progress: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    movie: Option<ScrobbleIdsParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    episode: Option<ScrobbleEpisodeParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show: Option<ScrobbleIdsParam>,
}
