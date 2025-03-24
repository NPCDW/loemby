use std::{collections::HashMap, sync::Arc};

use reqwest::Client;
use tauri::async_runtime::RwLock;

use crate::{config, service::proxy_svc::AxumAppState};

pub struct AppState {
    pub app_config: config::app_config::Config,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub reqwest_pool: RwLock<HashMap<String, Client>>,
}
