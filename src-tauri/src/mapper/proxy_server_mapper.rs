use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

use crate::{config::app_state::AppState, mapper::global_config_mapper};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct ProxyServer {
    pub id: Option<String>,
    pub create_time: Option<String>,
    pub name: Option<String>,
    pub proxy_type: Option<String>,
    pub addr: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

fn get_proxy_url(proxy: ProxyServer) -> String {
    let username = proxy.username.unwrap_or("".to_string());
    let password = proxy.password.unwrap_or("".to_string());
    let auth = if username == "" && password == "" {
        "".to_string()
    } else {
        format!("{}:{}@", username, password)
    };
    format!("{}://{}{}", proxy.proxy_type.unwrap(), auth, proxy.addr.unwrap())
}

pub async fn get_browse_proxy_url(proxy_id: Option<String>, state: &tauri::State<'_, AppState>) -> Option<String> {
    if proxy_id.is_none() {
        return None;
    }
    let proxy_id = proxy_id.unwrap();
    if proxy_id == "no" {
        return None;
    }
    if proxy_id == "follow" {
        let proxy_id = global_config_mapper::get_cache("global_browse_proxy_id", state).await;
        return Box::pin(get_browse_proxy_url(proxy_id, state)).await;
    }
    return get_cache(proxy_id, state).await;
}

pub async fn get_play_proxy_url(proxy_id: Option<String>, state: &tauri::State<'_, AppState>) -> Option<String> {
    if proxy_id.is_none() {
        return None;
    }
    let proxy_id = proxy_id.unwrap();
    if proxy_id == "no" {
        return None;
    }
    if proxy_id == "follow" {
        let proxy_id = global_config_mapper::get_cache("global_play_proxy_id", state).await;
        return Box::pin(get_play_proxy_url(proxy_id, state)).await;
    }
    return get_cache(proxy_id, state).await;
}

pub async fn get_app_proxy_url(proxy_id: Option<String>, state: &tauri::State<'_, AppState>) -> Option<String> {
    if proxy_id.is_none() {
        return None;
    }
    let proxy_id = proxy_id.unwrap();
    if proxy_id == "no" {
        return None;
    }
    if proxy_id == "followBrowse" {
        let proxy_id = global_config_mapper::get_cache("global_browse_proxy_id", state).await;
        return Box::pin(get_browse_proxy_url(proxy_id, state)).await;
    }
    if proxy_id == "followPlay" {
        let proxy_id = global_config_mapper::get_cache("global_play_proxy_id", state).await;
        return Box::pin(get_play_proxy_url(proxy_id, state)).await;
    }
    return get_cache(proxy_id, state).await;
}

pub async fn load_cache(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let list = list_all(&state.db_pool).await?;
    let mut cache_map_write = state.proxy_server_cache.write().await;
    cache_map_write.clear();
    for proxy in list {
        let id = proxy.id.clone().unwrap();
        let proxy_url = get_proxy_url(proxy);
        cache_map_write.insert(id, proxy_url);
    }
    anyhow::Ok(())
}

pub async fn refresh_cache(id: &str, state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let proxy_server = get_by_id(id.to_string(), &state.db_pool).await?;
    let mut cache_map_write = state.proxy_server_cache.write().await;
    match proxy_server {
        Some(proxy_server) => {
            let proxy_url = get_proxy_url(proxy_server);
            cache_map_write.insert(id.to_string(), proxy_url);
        },
        None => {
            cache_map_write.remove(id);
        },
    };
    anyhow::Ok(())
}

pub async fn get_cache(id: String, state: &tauri::State<'_, AppState>) -> Option<String> {
    let cache_map = state.proxy_server_cache.read().await;
    cache_map.get(&id).cloned()
}

pub async fn get_by_id(id: String, pool: &Pool<Sqlite>) -> anyhow::Result<Option<ProxyServer>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from proxy_server where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build_query_as::<ProxyServer>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询代理服务器: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn list_all(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<ProxyServer>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from proxy_server");
    let query = query_builder.build_query_as::<ProxyServer>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有代理服务器: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn create(entity: ProxyServer, state: &tauri::State<'_, AppState>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into proxy_server(");
    let mut separated = query_builder.separated(", ");
    separated.push("id");
    if entity.name.is_some() {
        separated.push("name");
    }
    if entity.proxy_type.is_some() {
        separated.push("proxy_type");
    }
    if entity.addr.is_some() {
        separated.push("addr");
    }
    if entity.username.is_some() {
        separated.push("username");
    }
    if entity.password.is_some() {
        separated.push("password");
    }
    query_builder.push(")  values(");
    let mut separated = query_builder.separated(", ");
    separated.push_bind(id.clone());
    if entity.name.is_some() {
        separated.push_bind(entity.name.unwrap());
    }
    if entity.proxy_type.is_some() {
        separated.push_bind(entity.proxy_type.unwrap());
    }
    if entity.addr.is_some() {
        separated.push_bind(entity.addr.unwrap());
    }
    if entity.username.is_some() {
        separated.push_bind(entity.username.unwrap());
    }
    if entity.password.is_some() {
        separated.push_bind(entity.password.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(&state.db_pool).await;
    tracing::debug!("sqlx: 添加代理服务器: {} {:?}", sql, res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn update_by_id(entity: ProxyServer, state: &tauri::State<'_, AppState>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let id = entity.id.clone().unwrap();
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update proxy_server set ");
    let mut separated = query_builder.separated(", ");
    if entity.name.is_some() {
        separated.push("name = ").push_bind_unseparated(entity.name.unwrap());
    }
    if entity.proxy_type.is_some() {
        separated.push("proxy_type = ").push_bind_unseparated(entity.proxy_type.unwrap());
    }
    if entity.addr.is_some() {
        separated.push("addr = ").push_bind_unseparated(entity.addr.unwrap());
    }
    if entity.username.is_some() {
        separated.push("username = ").push_bind_unseparated(entity.username.unwrap());
    }
    if entity.password.is_some() {
        separated.push("password = ").push_bind_unseparated(entity.password.unwrap());
    }
    query_builder.push(" where id = ").push_bind(entity.id.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(&state.db_pool).await;
    tracing::debug!("sqlx: 更新代理服务器: {} {:?}", sql, res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn delete_by_id(id: String, state: &tauri::State<'_, AppState>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let mut query_builder = QueryBuilder::new("delete from proxy_server where id = ");
    query_builder.push_bind(&id);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(&state.db_pool).await;
    tracing::debug!("sqlx: 删除代理服务器: {} {:?}", sql, res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}
