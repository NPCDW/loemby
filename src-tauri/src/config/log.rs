use std::str::FromStr;

use tauri::Manager;
use time::UtcOffset;
use tracing_subscriber::{filter::LevelFilter, fmt::time::OffsetTime, prelude::*, Layer};

pub fn init(app: &tauri::App, log_level: &str) {
    let dir = app
        .path()
        .resolve("loemby/logs/", tauri::path::BaseDirectory::AppLocalData)
        .unwrap();
    let file_appender = tracing_appender::rolling::never(dir, "loemby.log");
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
                .with_filter(level),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_timer(local_time.clone())
                .with_filter(level),
        )
        .init();
}
