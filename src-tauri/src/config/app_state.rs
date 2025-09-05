use std::{collections::HashMap, sync::Arc};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{config, service::proxy_svc::AxumAppState};

pub struct AppState {
    pub app_config: config::app_config::Config,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub api_reqwest_pool: RwLock<HashMap<String, Client>>,
    pub image_reqwest_pool: RwLock<HashMap<String, Client>>,
    pub db_pool: sqlx::Pool<sqlx::Sqlite>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TauriNotify {
    pub alert_type: String,
    pub message_type: String,
    pub title: Option<String>,
    pub message: String,
}