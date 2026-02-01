use serde::{Deserialize, Serialize};

use crate::{
    config::{app_state::AppState, db_pool::DbPool},
    db_execute, db_fetch_all, db_fetch_optional,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct GlobalConfig {
    pub id: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub config_key: Option<String>,
    pub config_value: Option<String>,
}

pub async fn load_cache(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let list = list_all(&state.db_pool).await?;
    let mut cache_map_write = state.global_config_cache.write().await;
    cache_map_write.clear();
    for config in list {
        cache_map_write.insert(config.config_key.unwrap(), config.config_value.unwrap());
    }
    anyhow::Ok(())
}

pub async fn refresh_cache(
    config_key: &str,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<()> {
    let global_config = get_by_key(config_key.to_string(), &state.db_pool).await?;
    let mut cache_map_write = state.global_config_cache.write().await;
    match global_config {
        Some(global_config) => {
            cache_map_write.insert(config_key.to_string(), global_config.config_value.unwrap());
        }
        None => {
            cache_map_write.remove(config_key);
        }
    };
    anyhow::Ok(())
}

pub async fn get_cache(config_key: &str, state: &tauri::State<'_, AppState>) -> Option<String> {
    let cache_map = state.global_config_cache.read().await;
    cache_map.get(config_key).cloned()
}

pub async fn get_by_key(config_key: String, pool: &DbPool) -> anyhow::Result<Option<GlobalConfig>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from global_config where config_key = ");
            qb.push_bind(config_key);
        },
        GlobalConfig
    )?;
    tracing::debug!("sqlx: 查询配置: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_all(pool: &DbPool) -> anyhow::Result<Vec<GlobalConfig>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from global_config");
        },
        GlobalConfig
    )?;
    tracing::debug!("sqlx: 查询所有配置: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create_or_update(
    entity: GlobalConfig,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    anyhow::Ok(
        match get_cache(entity.config_key.as_ref().unwrap(), state).await {
            Some(_) => update_by_key(entity, state).await,
            None => create(entity, state).await,
        }?,
    )
}

pub async fn create(
    entity: GlobalConfig,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let config_key = entity.config_key.clone();
    let config_value = entity.config_value.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("insert into global_config(");
        let mut separated = qb.separated(", ");
        separated.push("id");
        if entity.config_key.is_some() {
            separated.push("config_key");
        }
        if entity.config_value.is_some() {
            separated.push("config_value");
        }
        qb.push(")  values(");
        let mut separated = qb.separated(", ");
        separated.push_bind(id);
        if config_key.is_some() {
            separated.push_bind(config_key.clone().unwrap());
        }
        if config_value.is_some() {
            separated.push_bind(config_value.unwrap());
        }
        qb.push(")");
    });

    tracing::debug!("sqlx: 添加配置: {:?}", res);
    if res.is_ok() {
        refresh_cache(config_key.as_ref().unwrap(), state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn update_by_key(
    entity: GlobalConfig,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let config_key = entity.config_key.clone();
    let config_value = entity.config_value.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("update global_config set ");
        let mut separated = qb.separated(", ");
        if config_value.is_some() {
            separated.push("config_value = ");
            separated.push_bind_unseparated(config_value.unwrap());
        }
        qb.push(" where config_key = ");
        qb.push_bind(config_key.clone().unwrap());
    });

    tracing::debug!("sqlx: 更新配置: {:?}", res);
    if res.is_ok() {
        refresh_cache(config_key.as_ref().unwrap(), state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn delete_by_key(
    config_key: String,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("delete from global_config where config_key = ");
        qb.push_bind(&config_key);
    });

    tracing::debug!("sqlx: 删除配置: {:?}", res);
    if res.is_ok() {
        refresh_cache(&config_key, state).await?;
    }
    anyhow::Ok(res?)
}
