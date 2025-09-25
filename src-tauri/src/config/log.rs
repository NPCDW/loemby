use std::str::FromStr;

use tauri::Manager;
use time::UtcOffset;
use tracing_subscriber::{filter::LevelFilter, fmt::time::OffsetTime, prelude::*, Layer};

pub fn init(app: &tauri::App, log_level: &str) {
    let local_data_dir = app.path().resolve("", tauri::path::BaseDirectory::AppLocalData).unwrap();
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
