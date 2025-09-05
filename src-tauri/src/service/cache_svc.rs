use tauri::Manager;

use crate::controller::invoke_ctl::CleanCacheParam;

pub async fn clean(body: CleanCacheParam, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let cutoff_time = std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs(body.cutoff_day as u64 * 24 * 60 * 60)).unwrap();
    clean_r(body.dir, cutoff_time, body.force_clean, &app_handle).await?;
    anyhow::Ok(())
}

async fn clean_r(dir: String, cutoff_time: std::time::SystemTime, force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let absolute_dir = app_handle.path().resolve(&dir, tauri::path::BaseDirectory::AppLocalData)?;
    let files = std::fs::read_dir(&absolute_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for file in &files {
        let metadata = file.metadata()?;

        if metadata.is_dir() {
            Box::pin(clean_r(format!("{}/{}", &dir, file.file_name().unwrap().to_str().unwrap()), cutoff_time, force_clean, app_handle)).await?;
            continue
        }

        if force_clean {
            tokio::fs::remove_file(file).await?;
            tracing::debug!("Force deleted cache file: {}", file.display());
            continue
        }
        
        // 如果文件早于截止时间，则删除
        if let Ok(create_time) = metadata.created() {
            if create_time < cutoff_time {
                tokio::fs::remove_file(file).await?;
                tracing::debug!("Deleted cache file: {}", file.display());
            }
        }
    }
    let mut files = tokio::fs::read_dir(&absolute_dir).await?;
    if files.next_entry().await?.is_none() {
        tokio::fs::remove_dir(&absolute_dir).await?;
        tracing::debug!("Deleted empty directory: {}", absolute_dir.display());
    }
    anyhow::Ok(())
}
