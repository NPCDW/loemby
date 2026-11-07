use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::watch;

use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{config::{self, db_pool::DbPool}, service::axum_svc::AxumAppState};

#[derive(Clone, Debug, PartialEq)]
enum StartupStatus {
    Pending,
    Ready,
    Failed(String),
}

pub struct StartupState {
    status: watch::Sender<StartupStatus>,
}

impl Default for StartupState {
    fn default() -> Self {
        let (status, _) = watch::channel(StartupStatus::Pending);
        Self { status }
    }
}

impl StartupState {
    pub fn ready(&self) {
        self.status.send_replace(StartupStatus::Ready);
    }

    pub fn fail(&self, message: String) {
        self.status.send_replace(StartupStatus::Failed(message));
    }

    pub async fn wait_ready(&self, timeout: Duration) -> Result<(), String> {
        let mut status = self.status.subscribe();
        let wait = async {
            loop {
                let current = status.borrow_and_update().clone();
                match current {
                    StartupStatus::Ready => return Ok(()),
                    StartupStatus::Failed(message) => return Err(message),
                    StartupStatus::Pending => {}
                }
                status.changed().await.map_err(|_| "应用初始化状态已关闭，请重启应用".to_string())?;
            }
        };
        tokio::time::timeout(timeout, wait).await
            .map_err(|_| "应用初始化超时，请检查网络或配置后重试".to_string())?
    }
}

pub struct AppState {
    pub app_handle: tauri::AppHandle,
    pub app_config: config::app_config::Config,
    pub auxm_app_state: Arc::<RwLock<Option<AxumAppState>>>,
    pub api_reqwest_pool: Arc::<RwLock<HashMap<String, ClientWithMiddleware>>>,
    pub image_reqwest_pool: Arc::<RwLock<HashMap<String, ClientWithMiddleware>>>,
    pub stream_reqwest_pool: Arc::<RwLock<HashMap<String, ClientWithMiddleware>>>,
    pub db_pool: DbPool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn early_requests_wait_for_ready() {
        let startup = Arc::new(StartupState::default());
        let waiting = startup.clone();
        let request = tokio::spawn(async move { waiting.wait_ready(Duration::from_secs(1)).await });
        tokio::task::yield_now().await;
        assert!(!request.is_finished());
        startup.ready();
        assert_eq!(request.await.unwrap(), Ok(()));
    }

    #[tokio::test]
    async fn ready_is_retained_without_subscribers() {
        let startup = StartupState::default();
        startup.ready();
        assert_eq!(startup.wait_ready(Duration::from_secs(1)).await, Ok(()));
        assert_eq!(startup.wait_ready(Duration::from_secs(1)).await, Ok(()));
    }

    #[tokio::test]
    async fn ready_wakes_all_waiters() {
        let startup = StartupState::default();
        let ready = async { tokio::task::yield_now().await; startup.ready(); };
        let (first, second, _) = tokio::join!(
            startup.wait_ready(Duration::from_secs(1)),
            startup.wait_ready(Duration::from_secs(1)),
            ready,
        );
        assert_eq!(first, Ok(()));
        assert_eq!(second, Ok(()));
    }

    #[tokio::test]
    async fn failure_wakes_waiters_and_is_retained() {
        let startup = StartupState::default();
        let failed = async { tokio::task::yield_now().await; startup.fail("初始化失败".to_string()); };
        let (result, _) = tokio::join!(startup.wait_ready(Duration::from_secs(1)), failed);
        assert_eq!(result, Err("初始化失败".to_string()));
        assert_eq!(startup.wait_ready(Duration::from_secs(1)).await, result);
    }

    #[tokio::test]
    async fn timeout_does_not_prevent_a_later_retry() {
        let startup = StartupState::default();
        assert!(startup.wait_ready(Duration::from_millis(10)).await.unwrap_err().contains("超时"));
        startup.ready();
        assert_eq!(startup.wait_ready(Duration::from_secs(1)).await, Ok(()));
    }

    #[tokio::test]
    async fn ready_between_subscription_and_wait_is_not_lost() {
        for _ in 0..100 {
            let startup = StartupState::default();
            let (result, _) = tokio::join!(startup.wait_ready(Duration::from_secs(1)), async { startup.ready(); });
            assert_eq!(result, Ok(()));
        }
    }
}