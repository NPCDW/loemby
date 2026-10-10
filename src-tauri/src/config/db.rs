use anyhow::Ok;
use sqlx::{
    postgres::PgPoolOptions,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::time::Duration;
use tauri::Manager;

use crate::{config::app_config::Config, util::file_util};

use super::db_pool::DbPool;

/// 获取连接的最长等待时间，数据库不可达时前端很快就能拿到错误，而不是无限等待
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);
/// 借出连接的生命周期，配合 acquire_timeout 回收僵死连接
const MAX_LIFETIME: Duration = Duration::from_secs(600);
/// 空闲连接回收时间
const IDLE_TIMEOUT: Duration = Duration::from_secs(300);

pub async fn init(app: &tauri::App, config: &Config) -> anyhow::Result<DbPool> {
    tracing::debug!("DB Pool Trying to Init");

    let pool = match config.database_type.as_str() {
        "postgres" => {
            tracing::debug!("Initializing PostgreSQL pool with URL: {}", config.database_url);

            let connect_options: sqlx::postgres::PgConnectOptions =
                config.database_url.parse()?;

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(ACQUIRE_TIMEOUT)
                .max_lifetime(MAX_LIFETIME)
                .idle_timeout(IDLE_TIMEOUT)
                .connect_with(connect_options)
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
                .filename(&db_path)
                .journal_mode(SqliteJournalMode::Wal)
                .create_if_missing(true)
                .foreign_keys(false)
                .busy_timeout(Duration::from_secs(5));

            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .acquire_timeout(ACQUIRE_TIMEOUT)
                .max_lifetime(MAX_LIFETIME)
                .idle_timeout(IDLE_TIMEOUT)
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

/// 把数据库初始化失败翻译成用户能看懂的话
pub fn humanize_init_error(config: &Config, err: &anyhow::Error) -> String {
    let mut tips = Vec::new();
    if config.database_type == "postgres" {
        tips.push(format!("数据库类型：PostgreSQL\n连接地址：{}", config.database_url));
        tips.push("请确认 PostgreSQL 已启动、地址端口可达、账号密码与数据库名正确，且已放行 pg_hba 访问。".to_string());
    } else {
        tips.push("数据库类型：SQLite".to_string());
        tips.push("请确认应用配置目录可写，且磁盘空间充足。".to_string());
    }
    if let Some(sqlx_err) = err.downcast_ref::<sqlx::Error>() {
        match sqlx_err {
            sqlx::Error::PoolTimedOut => {
                tips.push("连接超时：数据库未在超时时间内响应。".to_string());
            }
            sqlx::Error::Database(db_err) => {
                tips.push(format!("数据库返回错误：{}", db_err.message()));
            }
            sqlx::Error::Io(io_err) => {
                tips.push(format!("网络或文件 IO 错误：{}", io_err));
            }
            _ => {
                tips.push(format!("错误详情：{}", sqlx_err));
            }
        }
    } else if err.downcast_ref::<sqlx::migrate::MigrateError>().is_some() {
        tips.push(format!("数据库迁移失败：{}", err));
        tips.push("请检查数据库账号是否有建表、改表权限，或库中是否残留了不一致的表结构。".to_string());
    } else {
        tips.push(format!("错误详情：{}", err));
    }
    tips.join("\n")
}
