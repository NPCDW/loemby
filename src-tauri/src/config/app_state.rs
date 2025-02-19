use std::path::PathBuf;

use tauri::async_runtime::RwLock;

use crate::config;

pub struct AppState {
    pub app_config: RwLock<config::app_config::Config>,
    #[allow(dead_code)]
    pub app: tauri::AppHandle,
    #[allow(dead_code)]
    pub debug: bool,
    pub root_dir: PathBuf,
}
