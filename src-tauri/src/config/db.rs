use anyhow::Ok;
use sqlx::{
    postgres::PgPoolOptions,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tauri::Manager;

use crate::{config::app_config::Config, util::file_util};

use super::db_pool::DbPool;

pub async fn init(app: &tauri::App, config: &Config) -> anyhow::Result<DbPool> {
    tracing::debug!("DB Pool Trying to Init");

    let pool = match config.database_type.as_str() {
        "postgres" => {
            let database_url = config
                .database_url
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("database_url is required for postgres"))?;

            tracing::debug!("Initializing PostgreSQL pool with URL: {}", database_url);

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?;

            tracing::debug!("PostgreSQL Pool Inited");

            sqlx::migrate!("./migrations/postgres")
                .run(&pool)
                .await?;

            tracing::debug!("PostgreSQL migrations completed");

            DbPool::Postgres(pool)
        }
        "sqlite" | _ => {
            let db_dir = app
                .path()
                .resolve("", tauri::path::BaseDirectory::AppConfig)?;
            let db_path = db_dir.join("db/loemby.db");
            file_util::create_file_if_not_exist(&db_path)?;

            let options = SqliteConnectOptions::new()
                .filename(db_path)
                .journal_mode(SqliteJournalMode::Wal)
                .create_if_missing(true)
                .foreign_keys(false);

            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await?;

            tracing::debug!("SQLite Pool Inited");

            sqlx::migrate!("./migrations/sqlite")
                .run(&pool)
                .await?;

            tracing::debug!("SQLite migrations completed");

            DbPool::Sqlite(pool)
        }
    };

    Ok(pool)
}
