use tauri_plugin_updater::UpdaterExt;

use crate::controller::invoke_ctl::UpdaterParam;

pub async fn update(body: UpdaterParam, app: tauri::AppHandle) -> tauri_plugin_updater::Result<bool> {
    let mut update = app.updater_builder()
        .timeout(std::time::Duration::from_secs(30))
        .header("Content-Type", body.user_agent).unwrap();
    if let Some(proxy_url) = body.proxy_url {
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
