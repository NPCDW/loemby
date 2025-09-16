use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::{config::app_state::AppState, mapper::global_config_mapper::{self, GlobalConfig}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleanCacheParam {
    pub dir: String,
    pub days_to_keep: u32,
    pub force_clean: bool,
}

pub async fn clean_plan(app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    tracing::info!("60秒后开始清理日志文件");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    clean_logs(false, app_handle).await?;

    tracing::info!("60秒后开始清理图标文件");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    clean_icon(false, app_handle).await?;

    tracing::info!("60秒后开始清理图片文件");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    clean_emby_image(None, false, app_handle).await?;

    anyhow::Ok(())
}

pub async fn clean_logs(force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let state = app_handle.state::<AppState>();
    if !force_clean {
        let clean_logs_config = global_config_mapper::get_cache("lastCleanLogsTime", &state).await;
        if clean_logs_config.is_some() {
            let last_clean_logs_time = clean_logs_config.unwrap().parse::<i64>().unwrap();
            if chrono::Local::now().timestamp() - last_clean_logs_time < 24 * 60 * 60 {
                tracing::info!("日志清理被强制忽略，最后一次执行时间：{}", chrono::DateTime::from_timestamp_secs(last_clean_logs_time).unwrap().with_timezone(&chrono::Local::now().timezone()).format("%Y-%m-%d %H:%M:%S").to_string());
                return anyhow::Ok(());
            }
        }
    }
    let log_stored_days = global_config_mapper::get_cache("logStoredDays", &state).await;
    let days_to_keep = log_stored_days.unwrap_or("30".to_string()).parse::<u32>().unwrap();
    clean("logs".to_string(), days_to_keep, force_clean, app_handle).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("lastCleanLogsTime".to_string()),
        config_value: Some(chrono::Local::now().timestamp().to_string()),
        ..Default::default()
    }, &state).await?;
    tracing::info!("日志文件清理完成");
    Ok(())
}

pub async fn clean_icon(force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let state = app_handle.state::<AppState>();
    if !force_clean {
        let clean_icon_config = global_config_mapper::get_cache("lastCleanIconTime", &state).await;
        if clean_icon_config.is_some() {
            let last_clean_icon_time = clean_icon_config.unwrap().parse::<i64>().unwrap();
            if chrono::Local::now().timestamp() - last_clean_icon_time < 24 * 60 * 60 {
                tracing::info!("图标清理被强制忽略，最后一次执行时间：{}", chrono::DateTime::from_timestamp_secs(last_clean_icon_time).unwrap().with_timezone(&chrono::Local::now().timezone()).format("%Y-%m-%d %H:%M:%S").to_string());
                return anyhow::Ok(());
            }
        }
    }
    let icon_stored_days = global_config_mapper::get_cache("iconStoredDays", &state).await;
    let days_to_keep = icon_stored_days.unwrap_or("365".to_string()).parse::<u32>().unwrap();
    clean("cache/icon".to_string(), days_to_keep, force_clean, app_handle).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("lastCleanIconTime".to_string()),
        config_value: Some(chrono::Local::now().timestamp().to_string()),
        ..Default::default()
    }, &state).await?;
    tracing::info!("图标文件清理完成");
    Ok(())
}

pub async fn clean_emby_image(emby_server_id: Option<String>, force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let mut clean_dir = "cache/image".to_string();
    if emby_server_id.is_some() {
        clean_dir = format!("cache/image/{}", emby_server_id.unwrap());
    }
    let state = app_handle.state::<AppState>();
    if !force_clean {
        let clean_emby_config = global_config_mapper::get_cache("lastCleanEmbyTime", &state).await;
        if clean_emby_config.is_some() {
            let last_clean_emby_time = clean_emby_config.unwrap().parse::<i64>().unwrap();
            if chrono::Local::now().timestamp() - last_clean_emby_time < 24 * 60 * 60 {
                tracing::info!("图片清理被强制忽略，最后一次执行时间：{}", chrono::DateTime::from_timestamp_secs(last_clean_emby_time).unwrap().with_timezone(&chrono::Local::now().timezone()).format("%Y-%m-%d %H:%M:%S").to_string());
                return anyhow::Ok(());
            }
        }
    }
    let cover_image_stored_days = global_config_mapper::get_cache("coverImageStoredDays", &state).await;
    let days_to_keep = cover_image_stored_days.unwrap_or("30".to_string()).parse::<u32>().unwrap();
    clean(clean_dir, days_to_keep, false, app_handle).await?;
    global_config_mapper::create_or_update(GlobalConfig {
        config_key: Some("lastCleanEmbyTime".to_string()),
        config_value: Some(chrono::Local::now().timestamp().to_string()),
        ..Default::default()
    }, &state).await?;
    tracing::info!("图片文件清理完成");
    Ok(())
}

pub async fn clean(dir: String, days_to_keep: u32, force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let cutoff_time = std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs(days_to_keep as u64 * 24 * 60 * 60)).unwrap();
    clean_r(dir, cutoff_time, force_clean, app_handle).await?;
    anyhow::Ok(())
}

async fn clean_r(dir: String, cutoff_time: std::time::SystemTime, force_clean: bool, app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let absolute_dir = app_handle.path().resolve(&dir, tauri::path::BaseDirectory::AppLocalData)?;
    let files = std::fs::read_dir(&absolute_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for file in &files {
        tracing::debug!("check cache file: {}", file.display());
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
