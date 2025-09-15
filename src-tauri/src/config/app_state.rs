use std::{collections::HashMap, sync::Arc};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{config, service::axum_svc::AxumAppState};

pub struct AppState {
    pub app_handle: tauri::AppHandle,
    pub app_config: config::app_config::Config,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub api_reqwest_pool: Arc::<RwLock<HashMap<String, Client>>>,
    pub image_reqwest_pool: Arc::<RwLock<HashMap<String, Client>>>,
    pub db_pool: sqlx::Pool<sqlx::Sqlite>,
    pub emby_server_chache: Arc::<RwLock<HashMap<String, crate::mapper::emby_server_mapper::EmbyServer>>>,
    pub global_config_chache: Arc::<RwLock<HashMap<String, String>>>,
    pub proxy_server_chache: Arc::<RwLock<HashMap<String, String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TauriNotify {
    pub alert_type: String,
    pub message_type: String,
    pub title: Option<String>,
    pub message: String,
}