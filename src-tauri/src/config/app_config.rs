use std::path::PathBuf;

use serde_inline_default::serde_inline_default;

use crate::util::file_util;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use super::app_state::AppState;

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyServer {
    pub id: String,
    pub name: String,
    pub proxy_type: String,
    pub addr: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbyServer {
    pub id: String,

    pub base_url: String,
    pub username: Option<String>,
    pub password: Option<String>,

    pub server_name: Option<String>,
    pub server_id: Option<String>,
    pub auth_token: Option<String>,
    pub user_id: Option<String>,

    pub client: Option<String>,
    pub device: Option<String>,
    pub device_id: Option<String>,
    pub client_version: Option<String>,
    pub user_agent: Option<String>,

    pub proxy_id: Option<String>,

    pub disabled: bool,
}

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde_inline_default("info".to_string())]
    pub log_level: String,
    pub mpv_path: Option<String>,
    pub emby_server: Vec<EmbyServer>,
    pub proxy_server: Vec<ProxyServer>,
}

const APP_CONFIG_PATH: &'static str = "config/app-config.json";
const RESOURCES_CONFIG_PATH: &'static str = "resources/config/app-config.default.json";

pub fn get_config(app: &tauri::App, root_dir: &PathBuf) -> anyhow::Result<Config> {
    let config_path = root_dir.join(APP_CONFIG_PATH);
    if !config_path.exists() {
        file_util::mkdir(config_path.parent().unwrap())?;
        let resource_path = app.path().resolve(
            RESOURCES_CONFIG_PATH,
            tauri::path::BaseDirectory::Resource,
        )?;
        file_util::copy(&resource_path, &config_path)?;
    }
    let content = file_util::read_file(&config_path)?;
    anyhow::Ok(serde_json::from_str(&content)?)
}

pub async fn save_config(state: tauri::State<'_, AppState>, config: Config) -> Result<(), ()> {
    let config_path = state.root_dir.join(APP_CONFIG_PATH);
    {
        *state.app_config.write().await = config.clone();
    }
    let content = serde_json::to_string(&config).unwrap();
    file_util::write_file(config_path, &content);
    Ok(())
}
