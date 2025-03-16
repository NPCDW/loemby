use std::path::PathBuf;

use serde_inline_default::serde_inline_default;

use crate::util::file_util;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde_inline_default("info".to_string())]
    pub log_level: String,
}

const APP_CONFIG_PATH: &'static str = "config/app-config.json";
const RESOURCES_CONFIG_PATH: &'static str = "resources/config/app-config.default.json";

pub fn get_config(app: &tauri::App, config_dir: &PathBuf) -> anyhow::Result<Config> {
    let config_path = config_dir.join(APP_CONFIG_PATH);
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
