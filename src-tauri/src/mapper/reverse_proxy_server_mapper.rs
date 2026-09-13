use serde::{Deserialize, Serialize};

use crate::{
    config::{app_state::AppState, db_pool::DbPool},
    db_execute, db_fetch_all, db_fetch_optional,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct ReverseProxyServer {
    pub id: Option<String>,
    pub create_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub name: Option<String>,
    pub url: Option<String>,
}

/// 规范化反代地址：确保以 / 结尾
pub fn normalize_url(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }
    if url.ends_with('/') {
        url.to_string()
    } else {
        format!("{}/", url)
    }
}

pub async fn load_cache(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let list = list_all(&state.db_pool).await?;
    let mut cache_map_write = state.reverse_proxy_server_cache.write().await;
    cache_map_write.clear();
    for reverse_proxy in list {
        let id = reverse_proxy.id.clone().unwrap();
        let url = normalize_url(&reverse_proxy.url.clone().unwrap_or_default());
        if !url.is_empty() {
            cache_map_write.insert(id, url);
        }
    }
    anyhow::Ok(())
}

pub async fn refresh_cache(id: &str, state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let reverse_proxy = get_by_id(id.to_string(), &state.db_pool).await?;
    let mut cache_map_write = state.reverse_proxy_server_cache.write().await;
    match reverse_proxy {
        Some(reverse_proxy) => {
            let url = normalize_url(&reverse_proxy.url.clone().unwrap_or_default());
            if url.is_empty() {
                cache_map_write.remove(id);
            } else {
                cache_map_write.insert(id.to_string(), url);
            }
        }
        None => {
            cache_map_write.remove(id);
        }
    };
    anyhow::Ok(())
}

pub async fn get_cache(id: String, state: &tauri::State<'_, AppState>) -> Option<String> {
    let cache_map = state.reverse_proxy_server_cache.read().await;
    cache_map.get(&id).cloned()
}

/// 根据 reverse_proxy_id 获取反代地址前缀
/// 为 "no" 或不存在时返回 None
pub async fn get_reverse_proxy_url(
    reverse_proxy_id: Option<String>,
    state: &tauri::State<'_, AppState>,
) -> Option<String> {
    let reverse_proxy_id = reverse_proxy_id?;
    if reverse_proxy_id == "no" || reverse_proxy_id.is_empty() {
        return None;
    }
    get_cache(reverse_proxy_id, state).await
}

pub async fn get_by_id(id: String, pool: &DbPool) -> anyhow::Result<Option<ReverseProxyServer>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from reverse_proxy_server where id = ");
            qb.push_bind(id);
        },
        ReverseProxyServer
    )?;
    tracing::debug!("sqlx: 查询反代服务器: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_all(pool: &DbPool) -> anyhow::Result<Vec<ReverseProxyServer>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from reverse_proxy_server");
        },
        ReverseProxyServer
    )?;
    tracing::debug!("sqlx: 查询所有反代服务器: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create(
    entity: ReverseProxyServer,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let name = entity.name.clone();
    let url = entity.url.as_ref().map(|url| normalize_url(url));

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("insert into reverse_proxy_server(");
        let mut separated = qb.separated(", ");
        separated.push("id");
        separated.push("create_time");
        if entity.name.is_some() {
            separated.push("name");
        }
        if entity.url.is_some() {
            separated.push("url");
        }
        qb.push(")  values(");
        let mut separated = qb.separated(", ");
        separated.push_bind(id.clone());
        separated.push_bind(chrono::Local::now().fixed_offset());
        if name.is_some() {
            separated.push_bind(name.unwrap());
        }
        if url.is_some() {
            separated.push_bind(url.unwrap());
        }
        qb.push(")");
    });

    tracing::debug!("sqlx: 添加反代服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn update_by_id(
    entity: ReverseProxyServer,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = entity.id.clone().unwrap();
    let name = entity.name.clone();
    let url = entity.url.as_ref().map(|url| normalize_url(url));
    let entity_id = entity.id.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("update reverse_proxy_server set ");
        let mut separated = qb.separated(", ");
        if name.is_some() {
            separated.push("name = ");
            separated.push_bind_unseparated(name.unwrap());
        }
        if url.is_some() {
            separated.push("url = ");
            separated.push_bind_unseparated(url.unwrap());
        }
        qb.push(" where id = ");
        qb.push_bind(entity_id.unwrap());
    });

    tracing::debug!("sqlx: 更新反代服务器: {:?}", res);
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
        qb.push("delete from reverse_proxy_server where id = ");
        qb.push_bind(&id);
    });

    tracing::debug!("sqlx: 删除反代服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}
