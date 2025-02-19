use std::{path::PathBuf, str::FromStr};

use time::UtcOffset;
use tracing_subscriber::{filter::LevelFilter, fmt::time::OffsetTime, prelude::*, Layer};

pub fn init(root_dir: &PathBuf, log_level: &str) {
    let logs_dir = root_dir.join("logs/");
    let file_appender = tracing_appender::rolling::never(logs_dir, "loemby.log");
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
