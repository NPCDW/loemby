use serde_inline_default::serde_inline_default;

use crate::util::file_util;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde_inline_default("info".to_string())]
    pub log_level: String,
    #[serde_inline_default("sqlite".to_string())]
    pub database_type: String,
    #[serde_inline_default("".to_string())]
    pub database_url: String,
}

const APP_CONFIG_PATH: &'static str = "config/app-config.json";

pub fn get_config(app: &tauri::App) -> anyhow::Result<Config> {
    let config_dir = app.path().resolve("", tauri::path::BaseDirectory::AppConfig)?;
    let config_path = config_dir.join(APP_CONFIG_PATH);
    if !config_path.exists() {
        file_util::mkdir(config_path.parent().unwrap())?;
        std::fs::write(
            &config_path,
            include_str!("../../resources/config/app-config.default.json"),
        )?;
    }
    let content = file_util::read_file(&config_path)?;
    anyhow::Ok(serde_json::from_str(&content)?)
}
