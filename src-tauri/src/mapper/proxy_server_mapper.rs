use serde::{Deserialize, Serialize};

use crate::{
    config::{app_state::AppState, db_pool::DbPool},
    db_execute, db_fetch_all, db_fetch_optional,
    mapper::global_config_mapper,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct ProxyServer {
    pub id: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
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
    format!(
        "{}://{}{}",
        proxy.proxy_type.unwrap(),
        auth,
        proxy.addr.unwrap()
    )
}

pub async fn get_browse_proxy_url(
    proxy_id: Option<String>,
    state: &tauri::State<'_, AppState>,
) -> Option<String> {
    if proxy_id.is_none() {
        let proxy_id = global_config_mapper::get_cache("global_browse_proxy_id", state).await;
        if proxy_id.is_none() {
            return None;
        }
        return Box::pin(get_browse_proxy_url(proxy_id, state)).await;
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

pub async fn get_play_proxy_url(
    proxy_id: Option<String>,
    state: &tauri::State<'_, AppState>,
) -> Option<String> {
    if proxy_id.is_none() {
        let proxy_id = global_config_mapper::get_cache("global_play_proxy_id", state).await;
        if proxy_id.is_none() {
            return None;
        }
        return Box::pin(get_play_proxy_url(proxy_id, state)).await;
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

pub async fn get_app_proxy_url(
    proxy_id: Option<String>,
    state: &tauri::State<'_, AppState>,
) -> Option<String> {
    if proxy_id.is_none() {
        let proxy_id = global_config_mapper::get_cache("global_browse_proxy_id", state).await;
        return Box::pin(get_browse_proxy_url(proxy_id, state)).await;
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
        }
        None => {
            cache_map_write.remove(id);
        }
    };
    anyhow::Ok(())
}

pub async fn get_cache(id: String, state: &tauri::State<'_, AppState>) -> Option<String> {
    let cache_map = state.proxy_server_cache.read().await;
    cache_map.get(&id).cloned()
}

pub async fn get_by_id(id: String, pool: &DbPool) -> anyhow::Result<Option<ProxyServer>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from proxy_server where id = ");
            qb.push_bind(id);
        },
        ProxyServer
    )?;
    tracing::debug!("sqlx: 查询代理服务器: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_all(pool: &DbPool) -> anyhow::Result<Vec<ProxyServer>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from proxy_server");
        },
        ProxyServer
    )?;
    tracing::debug!("sqlx: 查询所有代理服务器: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create(
    entity: ProxyServer,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let name = entity.name.clone();
    let proxy_type = entity.proxy_type.clone();
    let addr = entity.addr.clone();
    let username = entity.username.clone();
    let password = entity.password.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("insert into proxy_server(");
        let mut separated = qb.separated(", ");
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
        qb.push(")  values(");
        let mut separated = qb.separated(", ");
        separated.push_bind(id.clone());
        if name.is_some() {
            separated.push_bind(name.unwrap());
        }
        if proxy_type.is_some() {
            separated.push_bind(proxy_type.unwrap());
        }
        if addr.is_some() {
            separated.push_bind(addr.unwrap());
        }
        if username.is_some() {
            separated.push_bind(username.unwrap());
        }
        if password.is_some() {
            separated.push_bind(password.unwrap());
        }
        qb.push(")");
    });

    tracing::debug!("sqlx: 添加代理服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn update_by_id(
    entity: ProxyServer,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = entity.id.clone().unwrap();
    let name = entity.name.clone();
    let proxy_type = entity.proxy_type.clone();
    let addr = entity.addr.clone();
    let username = entity.username.clone();
    let password = entity.password.clone();
    let entity_id = entity.id.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("update proxy_server set ");
        let mut separated = qb.separated(", ");
        if name.is_some() {
            separated.push("name = ");
            separated.push_bind_unseparated(name.unwrap());
        }
        if proxy_type.is_some() {
            separated.push("proxy_type = ");
            separated.push_bind_unseparated(proxy_type.unwrap());
        }
        if addr.is_some() {
            separated.push("addr = ");
            separated.push_bind_unseparated(addr.unwrap());
        }
        if username.is_some() {
            separated.push("username = ");
            separated.push_bind_unseparated(username.unwrap());
        }
        if password.is_some() {
            separated.push("password = ");
            separated.push_bind_unseparated(password.unwrap());
        }
        qb.push(" where id = ");
        qb.push_bind(entity_id.unwrap());
    });

    tracing::debug!("sqlx: 更新代理服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn delete_by_id(
    id: String,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("delete from proxy_server where id = ");
        qb.push_bind(&id);
    });

    tracing::debug!("sqlx: 删除代理服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}
