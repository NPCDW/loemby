use std::os::unix::fs::MetadataExt;

use tauri::Manager;

use crate::controller::invoke_ctl::CleanCacheParam;

pub async fn clean(body: CleanCacheParam, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let cutoff_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() - body.cutoff_day as u64 * 24 * 60 * 60;
    clean_r(body.dir, cutoff_time as i64, body.force_clean, &app_handle).await?;
    anyhow::Ok(())
}

async fn clean_r(dir: String, cutoff_time: i64, force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let absolute_dir = app_handle.path().resolve(&dir, tauri::path::BaseDirectory::AppLocalData)?;
    let mut files = tokio::fs::read_dir(&absolute_dir).await?;
    while let Some(file) = files.next_entry().await? {
        let metadata = file.metadata().await?;

        if metadata.is_dir() {
            Box::pin(clean_r(format!("{}/{}", &dir, file.file_name().to_str().unwrap()), cutoff_time, force_clean, app_handle)).await?;
            continue
        }

        if force_clean {
            tokio::fs::remove_file(file.path()).await?;
            tracing::debug!("Force deleted cache file: {}", file.path().display());
            continue
        }
        
        // 如果文件早于截止时间，则删除
        if metadata.mtime() < cutoff_time {
            tokio::fs::remove_file(file.path()).await?;
            tracing::debug!("Deleted cache file: {}", file.path().display());
        }
    }
    let mut files = tokio::fs::read_dir(&absolute_dir).await?;
    if files.next_entry().await?.is_none() {
        tokio::fs::remove_dir(&absolute_dir).await?;
        tracing::debug!("Deleted empty directory: {}", absolute_dir.display());
    }
    anyhow::Ok(())
}
