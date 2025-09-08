use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

use crate::config::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct GlobalConfig {
    pub id: Option<String>,
    pub create_time: Option<String>,
    pub config_key: Option<String>,
    pub config_value: Option<String>,
}

pub async fn load_cache(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let list = list_all(&state.db_pool).await?;
    let mut cache_map_write = state.global_config_chache.write().await;
    cache_map_write.clear();
    for config in list {
        cache_map_write.insert(config.config_key.unwrap(), config.config_value.unwrap());
    }
    anyhow::Ok(())
}

pub async fn get_cache(config_key: &str, state: &tauri::State<'_, AppState>) -> Option<String> {
    let cache_map = state.global_config_chache.read().await;
    cache_map.get(config_key).cloned()
}

pub async fn get_by_key(config_key: String, pool: &Pool<Sqlite>) -> anyhow::Result<Option<GlobalConfig>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from global_config where config_key = ");
    query_builder.push_bind(config_key);
    let query = query_builder.build_query_as::<GlobalConfig>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询配置: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn list_all(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<GlobalConfig>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from global_config");
    let query = query_builder.build_query_as::<GlobalConfig>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有配置: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn create(entity: GlobalConfig, state: &tauri::State<'_, AppState>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let entity_clone = entity.clone();
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
    let res = query.execute(&state.db_pool).await;
    tracing::debug!("sqlx: 添加配置: {} {:?}", sql, res);
    if res.is_ok() {
        state.global_config_chache.write().await.insert(entity_clone.config_key.unwrap(), entity_clone.config_value.unwrap());
    }
    anyhow::Ok(res?)
}

pub async fn update_by_key(entity: GlobalConfig, state: &tauri::State<'_, AppState>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let entity_clone = entity.clone();
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update global_config set ");
    let mut separated = query_builder.separated(", ");
    if entity.config_value.is_some() {
        separated.push("config_value = ").push_bind_unseparated(entity.config_value.unwrap());
    }
    query_builder.push(" where config_key = ").push_bind(entity.config_key.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(&state.db_pool).await;
    tracing::debug!("sqlx: 更新配置: {} {:?}", sql, res);
    if res.is_ok() {
        state.global_config_chache.write().await.insert(entity_clone.config_key.unwrap(), entity_clone.config_value.unwrap());
    }
    anyhow::Ok(res?)
}

pub async fn delete_by_key(config_key: String, state: &tauri::State<'_, AppState>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let mut query_builder = QueryBuilder::new("delete from global_config where config_key = ");
    query_builder.push_bind(&config_key);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(&state.db_pool).await;
    tracing::debug!("sqlx: 删除配置: {} {:?}", sql, res);
    if res.is_ok() {
        state.global_config_chache.write().await.remove(&config_key);
    }
    anyhow::Ok(res?)
}
