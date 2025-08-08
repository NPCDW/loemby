use std::{path::PathBuf, str::FromStr};
// use std::{fs, time::{Duration, SystemTime}};

use time::UtcOffset;
use tracing_subscriber::{filter::LevelFilter, fmt::time::OffsetTime, prelude::*, Layer};

pub fn init(local_data_dir: &PathBuf, log_level: &str) {
    let logs_dir = local_data_dir.join("logs/");
    let file_appender = tracing_appender::rolling::daily(logs_dir, "loemby.log");
    let local_time = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        time::format_description::parse(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
        )
        .unwrap(),
    );
    let level = LevelFilter::from_str(log_level).unwrap_or(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_writer(std::io::stderr)
                .with_timer(local_time.clone())
                .with_line_number(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .log_internal_errors(true)
                .with_filter(level),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file_appender)
                .with_timer(local_time.clone())
                .with_line_number(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .log_internal_errors(true)
                .with_filter(level),
        )
        .init();
}

// pub fn clean(local_data_dir: &PathBuf) -> anyhow::Result<()> {
//     let now = SystemTime::now();
//     let thirty_days = Duration::from_secs(30 * 24 * 60 * 60);
    
//     for entry in fs::read_dir(local_data_dir)? {
//         let entry = entry?;
//         let path = entry.path();
        
//         if path.is_dir() {
//             clean(&path)?;
//             // 可选：删除空目录
//             if fs::read_dir(&path)?.count() == 0 {
//                 fs::remove_dir(&path)?;
//             }
//         } else {
//             let metadata = fs::metadata(&path)?;
//             let modified = metadata.modified()?;
//             let accessed = metadata.accessed()?;
            
//             if now.duration_since(modified)? > thirty_days 
//                 && now.duration_since(accessed)? > thirty_days {
//                 fs::remove_file(&path)?;
//             }
//         }
//     }
//     Ok(())
// }