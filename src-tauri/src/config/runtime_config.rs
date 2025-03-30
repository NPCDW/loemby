use serde::{Deserialize, Serialize};
use crate::config::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeConfig {
    pub version: String,
    pub app_config: super::app_config::Config,
    pub axum_port: u16,
}

pub async fn get_runtime_config(state: tauri::State<'_, AppState>) -> Result<RuntimeConfig, ()> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let app_config = state.app_config.clone();
    let axum_port = state.auxm_app_state.read().await.clone().unwrap().port;
    Ok(RuntimeConfig {
        version,
        app_config,
        axum_port,
    })
}