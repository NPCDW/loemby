use std::{collections::HashMap, path::PathBuf, sync::Arc};

use reqwest::Client;
use tauri::async_runtime::RwLock;

use crate::{config, service::proxy_svc::AxumAppState};

pub struct AppState {
    #[allow(dead_code)]
    pub app_config: RwLock<config::app_config::Config>,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub reqwest_pool: RwLock<HashMap<String, Client>>,
    #[allow(dead_code)]
    pub config_dir: PathBuf,
    #[allow(dead_code)]
    pub local_data_dir: PathBuf,
}
