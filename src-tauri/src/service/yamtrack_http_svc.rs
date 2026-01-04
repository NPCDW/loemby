use std::collections::HashMap;

use axum::http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{config::{app_state::AppState, http_pool}, mapper::{global_config_mapper, proxy_server_mapper}, service::{emby_http_svc::{EpisodeItem, SeriesItem}, trakt_http_svc}};

pub async fn track(param: &YamTrackParam, state: &tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let yamtrack_sync_url = global_config_mapper::get_cache("yamtrack_sync_url", state).await;
    if yamtrack_sync_url.is_none() {
        return Err(anyhow::anyhow!("yamtrack_sync_url 为空"));
    }
    let yamtrack_proxy_id = global_config_mapper::get_cache("yamtrack_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(yamtrack_proxy_id, state).await;
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_str(&format!("loemby/{}", env!("CARGO_PKG_VERSION"))).unwrap());

    let body_str = serde_json::to_string(param)?;
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(yamtrack_sync_url.unwrap())
        .headers(headers)
        .body(body_str.clone());
    let builder_print = format!("{:?} {}", &builder, body_str);
    let response = builder.send().await;
    tracing::debug!("yamtrack 同步 request {} response {:?}", builder_print, &response);
    let response = response?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("{}", response.status()));
    }
    let text = response.text().await?;
    tracing::debug!("yamtrack 同步 response text {}", text);
    Ok(text)
}

pub fn get_scrobble_yamtrack_param(episode: &EpisodeItem, series: &Option<SeriesItem>, playing: bool, completion: bool) -> Option<YamTrackParam> {
    let trakt_param = trakt_http_svc::get_scrobble_trakt_param(episode, series, 0.0);
    if trakt_param.is_none() {
        return None;
    }
    let trakt_param = trakt_param.unwrap();
    let trakt_episode_ids = if let Some(movie) = trakt_param.movie {
        Some(movie.ids)
    } else if let Some(episode) = trakt_param.episode {
        episode.ids
    } else {
        None
    };
    let  mut episode_external_ids = HashMap::new();
    if let Some(ids) = trakt_episode_ids {
        if ids.imdb.is_some() {
            episode_external_ids.insert("imdb_id".to_string(), ids.imdb.clone().unwrap());
        }
        if ids.tmdb.is_some() {
            episode_external_ids.insert("tmdb_id".to_string(), ids.tmdb.clone().unwrap());
        }
        if ids.tvdb.is_some() {
            episode_external_ids.insert("tvdb_id".to_string(), ids.tvdb.clone().unwrap());
        }
    }
    let trakt_tv_ids = if let Some(show) = trakt_param.show {
        Some(show.ids)
    } else {
        None
    };
    let  mut tv_external_ids = HashMap::new();
    if let Some(ids) = trakt_tv_ids {
        if ids.imdb.is_some() {
            tv_external_ids.insert("imdb_id".to_string(), ids.imdb.clone().unwrap());
        }
        if ids.tmdb.is_some() {
            tv_external_ids.insert("tmdb_id".to_string(), ids.tmdb.clone().unwrap());
        }
        if ids.tvdb.is_some() {
            tv_external_ids.insert("tvdb_id".to_string(), ids.tvdb.clone().unwrap());
        }
    }
    return Some(YamTrackParam {
        event: if playing { "start".to_string() } else { "stop".to_string() },
        external_ids: episode_external_ids,
        played: completion,
        type_: episode.type_.clone(),
        title: if episode.type_ == "Movie" {
            format!("{} ({})", episode.name.clone(), episode.production_year.clone().unwrap_or(0))
        } else {
            format!("{} S{}E{}. {}", episode.series_name.clone().unwrap_or_default(), episode.parent_index_number.clone().unwrap_or_default(), episode.index_number.clone().unwrap_or_default(), episode.name.clone())
        },
        tv_info: if episode.type_ == "Movie" { None } else {
            Some(YamTrackTvInfoParam {
                ids: tv_external_ids,
                season_number: episode.parent_index_number.clone(),
                episode_number: episode.index_number.clone(),
            })
        },
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamTrackParam {
    pub event: String,
    pub external_ids: HashMap<String, String>,
    pub played: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub tv_info: Option<YamTrackTvInfoParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamTrackTvInfoParam {
    pub ids: HashMap<String, String>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
}
