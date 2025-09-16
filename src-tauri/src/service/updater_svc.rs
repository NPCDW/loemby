use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

use crate::{config::app_state::AppState, mapper::{global_config_mapper, proxy_server_mapper}};

pub async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<bool> {
    let state = app.state::<AppState>();
    let app_proxy_id = global_config_mapper::get_cache("app_proxy_id", &state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(app_proxy_id, &state).await;
    let user_agent = format!("loemby/{}", env!("CARGO_PKG_VERSION"));
    
    let mut update = app.updater_builder()
        .timeout(std::time::Duration::from_secs(30))
        .header(reqwest::header::USER_AGENT, user_agent).unwrap();
    if let Some(proxy_url) = proxy_url {
        update = update.proxy(proxy_url.parse().expect("invalid proxy URL"));
    }
    let update = update.build()?.check().await?;
    if let Some(update) = update {
        let mut downloaded = 0;

        // alternatively we could also call update.download() and update.install() separately
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    tracing::debug!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    tracing::debug!("download finished");
                },
            )
            .await?;

        tracing::debug!("update installed");
        return Ok(true);
    }

    Ok(false)
}
