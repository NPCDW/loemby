use serde::Serialize;
use tauri::{async_runtime::Mutex, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{config::app_state::AppState, mapper::{global_config_mapper, proxy_server_mapper}};

#[derive(Default)]
pub struct UpdaterState {
    pending: Mutex<PendingUpdate>,
}

#[derive(Default)]
struct PendingUpdate {
    update: Option<Update>,
    installed: bool,
}

#[derive(Serialize)]
pub struct UpdateInfo {
    version: String,
}

pub async fn check(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater_state = app.state::<UpdaterState>();
    let mut pending = updater_state.pending.try_lock()
        .map_err(|_| "正在检查或安装更新，请稍后再试".to_string())?;
    if let Some(update) = &pending.update {
        return Ok(Some(UpdateInfo { version: update.version.clone() }));
    }

    let update = check_update(&app).await
        .map_err(|err| format!("检查更新失败: {}", err))?;
    let info = update.as_ref().map(|update| UpdateInfo { version: update.version.clone() });
    pending.update = update;
    Ok(info)
}

async fn check_update(app: &tauri::AppHandle) -> tauri_plugin_updater::Result<Option<Update>> {
    let state = app.state::<AppState>();
    let app_proxy_id = global_config_mapper::get_cache("app_proxy_id", &state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(app_proxy_id, &state).await;
    let user_agent = format!("loemby/{}", env!("CARGO_PKG_VERSION"));

    let mut updater = app.updater_builder()
        .timeout(std::time::Duration::from_secs(30))
        .header(reqwest::header::USER_AGENT, user_agent)?;
    if let Some(proxy_url) = proxy_url {
        updater = updater.proxy(proxy_url.parse()?);
    }
    updater.build()?.check().await
}

pub async fn install(app: tauri::AppHandle, version: String) -> Result<(), String> {
    let updater_state = app.state::<UpdaterState>();
    let mut pending = updater_state.pending.try_lock()
        .map_err(|_| "正在检查或安装更新，请稍后再试".to_string())?;
    let update = pending.update.as_ref()
        .ok_or_else(|| "请先检查更新".to_string())?;
    if update.version != version {
        return Err("更新版本已变化，请重新检查更新".to_string());
    }
    if pending.installed {
        return Ok(());
    }

    let mut downloaded = 0;
    update.download_and_install(
        |chunk_length, content_length| {
            downloaded += chunk_length;
            tracing::debug!("downloaded {downloaded} from {content_length:?}");
        },
        || {
            tracing::debug!("download finished");
        },
    ).await.map_err(|err| format!("更新失败: {}", err))?;

    pending.installed = true;
    tracing::info!("update installed");
    Ok(())
}
