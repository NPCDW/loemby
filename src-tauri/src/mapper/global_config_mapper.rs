use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct GlobalConfig {
    pub id: Option<String>,
    pub create_time: Option<String>,
    pub config_key: Option<String>,
    pub config_value: Option<String>,
}

pub async fn get_by_key(config_key: String, pool: &Pool<Sqlite>) -> Result<Option<GlobalConfig>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from global_config where config_key = ");
    query_builder.push_bind(config_key);
    let query = query_builder.build_query_as::<GlobalConfig>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询配置: {} {:?}", sql, res);
    res
}

pub async fn list_all(pool: &Pool<Sqlite>) -> Result<Vec<GlobalConfig>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from global_config");
    let query = query_builder.build_query_as::<GlobalConfig>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有配置: {} {:?}", sql, res);
    res
}

pub async fn create(entity: GlobalConfig, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into global_config(");
    let mut separated = query_builder.separated(", ");
    separated.push("id");
    if entity.config_key.is_some() {
        separated.push("config_key");
    }
    if entity.config_value.is_some() {
        separated.push("config_value");
    }
    query_builder.push(")  values(");
    let mut separated = query_builder.separated(", ");
    if entity.id.is_some() {
        separated.push_bind(entity.id.unwrap());
    } else {
        separated.push_bind(uuid::Uuid::new_v4().to_string());
    }
    if entity.config_key.is_some() {
        separated.push_bind(entity.config_key.unwrap());
    }
    if entity.config_value.is_some() {
        separated.push_bind(entity.config_value.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 添加配置: {} {:?}", sql, res);
    res
}

pub async fn update_by_key(entity: GlobalConfig, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update global_config set ");
    let mut separated = query_builder.separated(", ");
    if entity.config_value.is_some() {
        separated.push("config_value = ").push_bind_unseparated(entity.config_value.unwrap());
    }
    query_builder.push(" where config_key = ").push_bind(entity.config_key.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新配置: {} {:?}", sql, res);
    res
}

pub async fn delete_by_key(config_key: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder = QueryBuilder::new("delete from global_config where config_key = ");
    query_builder.push_bind(config_key);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 删除配置: {} {:?}", sql, res);
    res
}
