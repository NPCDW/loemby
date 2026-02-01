use sqlx::{PgPool, SqlitePool, postgres::PgQueryResult, sqlite::SqliteQueryResult};

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
