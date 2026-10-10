use std::sync::RwLock;

use sqlx::{PgPool, SqlitePool, postgres::PgQueryResult, sqlite::SqliteQueryResult};

/// 全局数据库状态，数据库初始化失败后所有数据库操作统一返回可读提示
static DB_STATE: RwLock<DbState> = RwLock::new(DbState::Ready);

/// 数据库是否已初始化成功
///
/// 数据库连不上时应用不再崩溃退出，而是以"无数据库"状态启动并提示用户，
/// 因此所有依赖 db_pool 的入口都要先判断这里。
#[derive(Debug, Clone, PartialEq)]
pub enum DbState {
    Ready,
    Failed(String),
}

/// Database pool wrapper that supports both SQLite and PostgreSQL
#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

impl DbPool {
    /// Close the pool
    pub async fn close(&self) {
        match self {
            DbPool::Sqlite(pool) => pool.close().await,
            DbPool::Postgres(pool) => pool.close().await,
        }
    }
}

/// Query result wrapper
#[derive(Debug)]
pub enum DbQueryResult {
    Sqlite(SqliteQueryResult),
    Postgres(PgQueryResult),
}

impl DbQueryResult {
    pub fn rows_affected(&self) -> u64 {
        match self {
            DbQueryResult::Sqlite(result) => result.rows_affected(),
            DbQueryResult::Postgres(result) => result.rows_affected(),
        }
    }
}

/// Macro to execute queries for both databases
#[macro_export]
macro_rules! db_execute {
    ($pool:expr, |$qb:ident| $body:block) => {{
        $crate::config::db_pool::require_ready()?;
        match $pool {
            $crate::config::db_pool::DbPool::Sqlite(pool) => {
                let mut $qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new("");
                $body
                let query = $qb.build();
                query.execute(pool).await.map(|r| $crate::config::db_pool::DbQueryResult::Sqlite(r))
            }
            $crate::config::db_pool::DbPool::Postgres(pool) => {
                let mut $qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("");
                $body
                let query = $qb.build();
                query.execute(pool).await.map(|r| $crate::config::db_pool::DbQueryResult::Postgres(r))
            }
        }
    }};
}

/// Macro to fetch optional row for both databases
#[macro_export]
macro_rules! db_fetch_optional {
    ($pool:expr, |$qb:ident| $body:block, $row_type:ty) => {{
        $crate::config::db_pool::require_ready()?;
        match $pool {
            $crate::config::db_pool::DbPool::Sqlite(pool) => {
                let mut $qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new("");
                $body
                $qb.build_query_as::<$row_type>().fetch_optional(pool).await
            }
            $crate::config::db_pool::DbPool::Postgres(pool) => {
                let mut $qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("");
                $body
                $qb.build_query_as::<$row_type>().fetch_optional(pool).await
            }
        }
    }};
}

/// Macro to fetch all rows for both databases
#[macro_export]
macro_rules! db_fetch_all {
    ($pool:expr, |$qb:ident| $body:block, $row_type:ty) => {{
        $crate::config::db_pool::require_ready()?;
        match $pool {
            $crate::config::db_pool::DbPool::Sqlite(pool) => {
                let mut $qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new("");
                $body
                $qb.build_query_as::<$row_type>().fetch_all(pool).await
            }
            $crate::config::db_pool::DbPool::Postgres(pool) => {
                let mut $qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("");
                $body
                $qb.build_query_as::<$row_type>().fetch_all(pool).await
            }
        }
    }};
}

/// 校验数据库可用，不可用时返回带用户提示的错误
pub fn require_ready() -> anyhow::Result<()> {
    if let DbState::Failed(reason) = db_state() {
        return Err(anyhow::anyhow!("{}", reason));
    }
    Ok(())
}

/// 读取全局数据库状态
pub fn db_state() -> DbState {
    DB_STATE
        .read()
        .map(|state| state.clone())
        .unwrap_or_else(|err| DbState::Failed(format!("数据库状态读取失败：{}", err)))
}

/// 更新全局数据库状态
pub fn set_db_state(state: DbState) {
    if let Ok(mut guard) = DB_STATE.write() {
        *guard = state;
    }
}
