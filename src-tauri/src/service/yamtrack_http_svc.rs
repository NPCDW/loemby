use std::collections::HashMap;

use axum::http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{config::{app_state::AppState, http_pool}, mapper::{global_config_mapper, proxy_server_mapper}, service::{emby_http_svc::EpisodeItem, trakt_http_svc}};

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
    let form = reqwest::multipart::Form::new()
        .text("data", body_str.clone());
    let client = http_pool::get_api_http_client(proxy_url, state).await?;
    let builder = client
        .post(yamtrack_sync_url.unwrap())
        .headers(headers)
        .multipart(form);
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

pub fn get_scrobble_yamtrack_param(episode: &EpisodeItem, playing: bool, completion: bool) -> Option<YamTrackParam> {
    let ids = trakt_http_svc::get_scrobble_trakt_ids_param(&episode.provider_ids, &episode.external_urls);
    if ids.imdb.is_some() || ids.tmdb.is_some() || ids.tvdb.is_some() {
        let  mut provider_ids = HashMap::new();
        if ids.imdb.is_some() {
            provider_ids.insert("Imdb".to_string(), ids.imdb.clone().unwrap());
        }
        if ids.tmdb.is_some() {
            provider_ids.insert("Tmdb".to_string(), ids.tmdb.clone().unwrap());
        }
        if ids.tvdb.is_some() {
            provider_ids.insert("Tvdb".to_string(), ids.tvdb.clone().unwrap());
        }
        return Some(YamTrackParam {
            event: if playing { "playback.start".to_string() } else { "playback.stop".to_string() },
            playback_info: YamTrackPlaybackInfoParam { played_to_completion: completion },
            item: YamTrackItemParam {
                type_: episode.type_.clone(),
                provider_ids,
                series_name: episode.series_name.clone(),
                parent_index_number: episode.parent_index_number.clone(),
                index_number: episode.index_number.clone(),
                name: episode.name.clone(),
                production_year: episode.production_year.clone(),
            },
        });
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamTrackParam {
    #[serde(rename = "Event")]
    pub event: String,
    #[serde(rename = "PlaybackInfo")]
    pub playback_info: YamTrackPlaybackInfoParam,
    #[serde(rename = "Item")]
    pub item: YamTrackItemParam,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamTrackPlaybackInfoParam {
    #[serde(rename = "PlayedToCompletion")]
    pub played_to_completion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamTrackItemParam {
    #[serde(rename = "Type")]
    pub type_: String,
    #[serde(rename = "ProviderIds")]
    pub provider_ids: HashMap<String, String>,
    #[serde(rename = "SeriesName")]
    pub series_name: Option<String>,
    #[serde(rename = "ParentIndexNumber")]
    pub parent_index_number: Option<u32>,
    #[serde(rename = "IndexNumber")]
    pub index_number: Option<u32>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ProductionYear")]
    pub production_year: Option<u32>,
}
