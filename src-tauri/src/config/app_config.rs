use serde_inline_default::serde_inline_default;

use serde::{Serialize, Deserialize};
use tauri::{Manager, State};
use crate::util::file_util;

use super::app_state::AppState;

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde_inline_default("info".to_string())]
    pub log_level: String,
}

#[tauri::command]
pub fn get_config_command(state: State<'_, AppState>) -> Config {
    state.app_config.clone()
}

pub fn get_config(app: &tauri::App) -> anyhow::Result<Config> {
    let config_path = app.path().resolve("loemby/config/app-config.json", tauri::path::BaseDirectory::AppLocalData)?;
    if !config_path.exists() {
        let resource_path = app.path().resolve("resources/config/app-config.default.json", tauri::path::BaseDirectory::Resource)?;
        file_util::copy(&resource_path, &config_path)?;
    }
    let content = file_util::read_file(&config_path)?;
    anyhow::Ok(serde_json::from_str(&content)?)
}

// #[tauri::command]
// pub fn save_config(app: &tauri::App, config: Config) {
//     let config_path = app.path().resolve("loemby/config/app-config.json", tauri::path::BaseDirectory::AppLocalData).unwrap();
//     let content = serde_json::to_string(&config).unwrap();
//     file_util::write_file(config_path, &content)
// }
