use std::{collections::HashMap, sync::Arc};

use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{config::{self, db_pool::DbPool}, service::axum_svc::AxumAppState};

pub struct AppState {
    pub app_handle: tauri::AppHandle,
    pub app_config: config::app_config::Config,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub api_reqwest_pool: Arc::<RwLock<HashMap<String, ClientWithMiddleware>>>,
    pub image_reqwest_pool: Arc::<RwLock<HashMap<String, ClientWithMiddleware>>>,
    pub stream_reqwest_pool: Arc::<RwLock<HashMap<String, ClientWithMiddleware>>>,
    /// 数据库连接池，连接不上时为 None，此时应用以只读/受限状态运行并提示用户
    pub db_pool: Option<DbPool>,
    pub emby_server_cache: Arc::<RwLock<HashMap<String, crate::mapper::emby_server_mapper::EmbyServer>>>,
    pub global_config_cache: Arc::<RwLock<HashMap<String, String>>>,
    pub proxy_server_cache: Arc::<RwLock<HashMap<String, String>>>,
    pub reverse_proxy_server_cache: Arc::<RwLock<HashMap<String, String>>>,
    pub emby_http_cache: Arc::<RwLock<HashMap<String, String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TauriNotify {
    pub event_type: String,
    pub message_type: String,
    pub title: Option<String>,
    pub message: String,
}

impl AppState {
    /// 获取数据库连接池，未就绪时返回用户可读的错误（而不是 panic 或静默卡住）
    pub fn db(&self) -> Result<&DbPool, String> {
        self.db_pool
            .as_ref()
            .ok_or_else(|| self.db_unavailable_reason())
    }

    /// 数据库不可用的原因，给用户看的文案
    pub fn db_unavailable_reason(&self) -> String {
        match crate::config::db_pool::db_state() {
            crate::config::db_pool::DbState::Failed(reason) => reason,
            crate::config::db_pool::DbState::Ready => "数据库连接不可用".to_string(),
        }
    }
}

/// 数据库初始化失败时发给前端的提示
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbFatalNotify {
    pub reason: String,
}
