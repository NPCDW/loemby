use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{config::{self, db_pool::DbPool, http_traffic_middleware::{TrafficScope, TrafficStats}}, service::axum_svc::AxumAppState};

pub struct AppState {
    pub app_handle: tauri::AppHandle,
    pub app_config: config::app_config::Config,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub api_reqwest_pool: Arc::<RwLock<HashMap<String, reqwest_middleware::ClientWithMiddleware>>>,
    pub image_reqwest_pool: Arc::<RwLock<HashMap<String, reqwest_middleware::ClientWithMiddleware>>>,
    pub db_pool: DbPool,
    pub emby_server_cache: Arc::<RwLock<HashMap<String, crate::mapper::emby_server_mapper::EmbyServer>>>,
    pub global_config_cache: Arc::<RwLock<HashMap<String, String>>>,
    pub proxy_server_cache: Arc::<RwLock<HashMap<String, String>>>,
    pub emby_http_cache: Arc::<RwLock<HashMap<String, String>>>,
    pub traffic_stat: Arc::<RwLock<HashMap<TrafficScope, Arc<TrafficStats>>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TauriNotify {
    pub event_type: String,
    pub message_type: String,
    pub title: Option<String>,
    pub message: String,
}