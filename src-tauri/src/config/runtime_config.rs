use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::Manager;
use crate::config::app_state::{AppState, StartupState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeConfig {
    pub version: String,
    pub app_config: super::app_config::Config,
    pub axum_port: u16,
}

pub async fn get_runtime_config(app: tauri::AppHandle) -> Result<RuntimeConfig, String> {
    let startup = app.try_state::<StartupState>()
        .ok_or_else(|| "应用启动状态未注册，请重启应用".to_string())?;
    startup.wait_ready(Duration::from_secs(30)).await?;
    let state = app.try_state::<AppState>()
        .ok_or_else(|| "应用初始化状态不完整，请重启应用".to_string())?;
    let version = env!("CARGO_PKG_VERSION").to_string();
    let app_config = state.app_config.clone();
    let axum_port = state.auxm_app_state.read().await.as_ref()
        .ok_or_else(|| "本地媒体服务尚未就绪，请重启应用".to_string())?.port;
    Ok(RuntimeConfig {
        version,
        app_config,
        axum_port,
    })
}