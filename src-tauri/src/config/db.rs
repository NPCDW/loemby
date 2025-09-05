use std::path::PathBuf;

use anyhow::Ok;
use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions}, Pool, Sqlite};

use crate::util::file_util;

pub async fn init(db_dir: PathBuf) -> anyhow::Result<Pool<Sqlite>> {
    let db_path = db_dir.join("db/loemby.db");
    file_util::create_file_if_not_exist(&db_path)?;

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .foreign_keys(false);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
