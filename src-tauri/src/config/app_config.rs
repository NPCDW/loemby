use serde_inline_default::serde_inline_default;

use crate::util::file_util;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use super::app_state::AppState;

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyServer {
    pub id: String,
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

    pub proxy_id: Option<String>,
}

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde_inline_default("info".to_string())]
    pub log_level: String,
    pub emby_server: Vec<EmbyServer>,
    pub proxy_server: Vec<ProxyServer>,
}

#[tauri::command]
pub async fn get_config_command(state: State<'_, AppState>) -> Result<Config, ()> {
    Ok(state.app_config.read().await.clone())
}

pub fn get_config(app: &tauri::App) -> anyhow::Result<Config> {
    let config_path = app.path().resolve(
        "loemby/config/app-config.json",
        tauri::path::BaseDirectory::AppLocalData,
    )?;
    if !config_path.exists() {
        file_util::mkdir(config_path.parent().unwrap())?;
        let resource_path = app.path().resolve(
            "resources/config/app-config.default.json",
            tauri::path::BaseDirectory::Resource,
        )?;
        file_util::copy(&resource_path, &config_path)?;
    }
    let content = file_util::read_file(&config_path)?;
    anyhow::Ok(serde_json::from_str(&content)?)
}

#[tauri::command]
pub async fn save_config(state: tauri::State<'_, AppState>, config: Config) -> Result<(), ()> {
    let config_path = state
        .app
        .path()
        .resolve(
            "loemby/config/app-config.json",
            tauri::path::BaseDirectory::AppLocalData,
        )
        .unwrap();
    {
        *state.app_config.write().await = config.clone();
    }
    let content = serde_json::to_string(&config).unwrap();
    file_util::write_file(config_path, &content);
    Ok(())
}
