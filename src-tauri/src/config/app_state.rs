use tauri::async_runtime::RwLock;

use crate::config;

pub struct AppState {
    pub app_config: RwLock::<config::app_config::Config>,
    pub app: tauri::AppHandle,
}
