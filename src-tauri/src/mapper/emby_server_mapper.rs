use serde::{Deserialize, Serialize};

use crate::{
    config::{app_state::AppState, db_pool::DbPool},
    db_execute, db_fetch_all, db_fetch_optional,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct EmbyServer {
    pub id: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,

    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,

    pub server_name: Option<String>,
    pub server_id: Option<String>,
    pub auth_token: Option<String>,
    pub user_id: Option<String>,

    pub client: Option<String>,
    pub device: Option<String>,
    pub device_id: Option<String>,
    pub client_version: Option<String>,
    pub user_agent: Option<String>,

    pub order_by: Option<i32>,
    pub icon_url: Option<String>,

    pub browse_proxy_id: Option<String>,
    pub play_proxy_id: Option<String>,
    pub line_id: Option<String>,

    pub last_playback_time: Option<chrono::NaiveDateTime>,
    pub keep_alive_days: Option<i32>,

    pub disabled: Option<i32>,
}

pub async fn load_cache(state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let list = list_all(&state.db_pool).await?;
    let mut cache_map_write = state.emby_server_cache.write().await;
    cache_map_write.clear();
    for server in list {
        cache_map_write.insert(server.id.clone().unwrap(), server);
    }
    anyhow::Ok(())
}

pub async fn refresh_cache(id: &str, state: &tauri::State<'_, AppState>) -> anyhow::Result<()> {
    let emby_server = get_by_id(id.to_string(), &state.db_pool).await?;
    let mut cache_map_write = state.emby_server_cache.write().await;
    match emby_server {
        Some(emby_server) => {
            cache_map_write.insert(id.to_string(), emby_server);
        }
        None => {
            cache_map_write.remove(id);
        }
    };
    anyhow::Ok(())
}

pub async fn get_cache(id: &str, state: &tauri::State<'_, AppState>) -> Option<EmbyServer> {
    let cache_map = state.emby_server_cache.read().await;
    cache_map.get(id).cloned()
}

pub async fn get_by_id(id: String, pool: &DbPool) -> anyhow::Result<Option<EmbyServer>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from emby_server where id = ");
            qb.push_bind(id);
        },
        EmbyServer
    )?;
    tracing::debug!("sqlx: 查询emby服务器: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_all(pool: &DbPool) -> anyhow::Result<Vec<EmbyServer>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from emby_server");
        },
        EmbyServer
    )?;
    tracing::debug!("sqlx: 查询所有emby服务器: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create(
    entity: EmbyServer,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let base_url = entity.base_url.clone();
    let username = entity.username.clone();
    let password = entity.password.clone();
    let server_name = entity.server_name.clone();
    let server_id = entity.server_id.clone();
    let auth_token = entity.auth_token.clone();
    let user_id = entity.user_id.clone();
    let client = entity.client.clone();
    let device = entity.device.clone();
    let device_id = entity.device_id.clone();
    let client_version = entity.client_version.clone();
    let user_agent = entity.user_agent.clone();
    let order_by = entity.order_by.clone();
    let icon_url = entity.icon_url.clone();
    let browse_proxy_id = entity.browse_proxy_id.clone();
    let play_proxy_id = entity.play_proxy_id.clone();
    let line_id = entity.line_id.clone();
    let last_playback_time = entity.last_playback_time.clone();
    let keep_alive_days = entity.keep_alive_days.clone();
    let disabled = entity.disabled.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("insert into emby_server(");
        let mut separated = qb.separated(", ");
        separated.push("id");
        if entity.base_url.is_some() {
            separated.push("base_url");
        }
        if entity.username.is_some() {
            separated.push("username");
        }
        if entity.password.is_some() {
            separated.push("password");
        }
        if entity.server_name.is_some() {
            separated.push("server_name");
        }
        if entity.server_id.is_some() {
            separated.push("server_id");
        }
        if entity.auth_token.is_some() {
            separated.push("auth_token");
        }
        if entity.user_id.is_some() {
            separated.push("user_id");
        }
        if entity.client.is_some() {
            separated.push("client");
        }
        if entity.device.is_some() {
            separated.push("device");
        }
        if entity.device_id.is_some() {
            separated.push("device_id");
        }
        if entity.client_version.is_some() {
            separated.push("client_version");
        }
        if entity.user_agent.is_some() {
            separated.push("user_agent");
        }
        if entity.order_by.is_some() {
            separated.push("order_by");
        }
        if entity.icon_url.is_some() {
            separated.push("icon_url");
        }
        if entity.browse_proxy_id.is_some() {
            separated.push("browse_proxy_id");
        }
        if entity.play_proxy_id.is_some() {
            separated.push("play_proxy_id");
        }
        if entity.line_id.is_some() {
            separated.push("line_id");
        }
        if entity.last_playback_time.is_some() {
            separated.push("last_playback_time");
        }
        if entity.keep_alive_days.is_some() {
            separated.push("keep_alive_days");
        }
        if entity.disabled.is_some() {
            separated.push("disabled");
        }
        qb.push(")  values(");
        let mut separated = qb.separated(", ");
        separated.push_bind(id.clone());
        if base_url.is_some() {
            separated.push_bind(base_url.unwrap());
        }
        if username.is_some() {
            separated.push_bind(username.unwrap());
        }
        if password.is_some() {
            separated.push_bind(password.unwrap());
        }
        if server_name.is_some() {
            separated.push_bind(server_name.unwrap());
        }
        if server_id.is_some() {
            separated.push_bind(server_id.unwrap());
        }
        if auth_token.is_some() {
            separated.push_bind(auth_token.unwrap());
        }
        if user_id.is_some() {
            separated.push_bind(user_id.unwrap());
        }
        if client.is_some() {
            separated.push_bind(client.unwrap());
        }
        if device.is_some() {
            separated.push_bind(device.unwrap());
        }
        if device_id.is_some() {
            separated.push_bind(device_id.unwrap());
        }
        if client_version.is_some() {
            separated.push_bind(client_version.unwrap());
        }
        if user_agent.is_some() {
            separated.push_bind(user_agent.unwrap());
        }
        if order_by.is_some() {
            separated.push_bind(order_by.unwrap());
        }
        if icon_url.is_some() {
            separated.push_bind(icon_url.unwrap());
        }
        if browse_proxy_id.is_some() {
            separated.push_bind(browse_proxy_id.unwrap());
        }
        if play_proxy_id.is_some() {
            separated.push_bind(play_proxy_id.unwrap());
        }
        if line_id.is_some() {
            separated.push_bind(line_id.unwrap());
        }
        if last_playback_time.is_some() {
            separated.push_bind(last_playback_time.unwrap());
        }
        if keep_alive_days.is_some() {
            separated.push_bind(keep_alive_days.unwrap());
        }
        if disabled.is_some() {
            separated.push_bind(disabled.unwrap());
        }
        qb.push(")");
    });

    tracing::debug!("sqlx: 添加emby服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn update_by_id(
    entity: EmbyServer,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = entity.id.clone().unwrap();
    let base_url = entity.base_url.clone();
    let username = entity.username.clone();
    let password = entity.password.clone();
    let server_name = entity.server_name.clone();
    let server_id = entity.server_id.clone();
    let auth_token = entity.auth_token.clone();
    let user_id = entity.user_id.clone();
    let client = entity.client.clone();
    let device = entity.device.clone();
    let device_id = entity.device_id.clone();
    let client_version = entity.client_version.clone();
    let user_agent = entity.user_agent.clone();
    let order_by = entity.order_by.clone();
    let icon_url = entity.icon_url.clone();
    let browse_proxy_id = entity.browse_proxy_id.clone();
    let play_proxy_id = entity.play_proxy_id.clone();
    let line_id = entity.line_id.clone();
    let last_playback_time = entity.last_playback_time.clone();
    let keep_alive_days = entity.keep_alive_days.clone();
    let disabled = entity.disabled.clone();
    let entity_id = entity.id.clone();

    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("update emby_server set ");
        let mut separated = qb.separated(", ");
        if base_url.is_some() {
            separated.push("base_url = ");
            separated.push_bind_unseparated(base_url.unwrap());
        }
        if username.is_some() {
            separated.push("username = ");
            separated.push_bind_unseparated(username.unwrap());
        }
        if password.is_some() {
            separated.push("password = ");
            separated.push_bind_unseparated(password.unwrap());
        }
        if server_name.is_some() {
            separated.push("server_name = ");
            separated.push_bind_unseparated(server_name.unwrap());
        }
        if server_id.is_some() {
            separated.push("server_id = ");
            separated.push_bind_unseparated(server_id.unwrap());
        }
        if auth_token.is_some() {
            separated.push("auth_token = ");
            separated.push_bind_unseparated(auth_token.unwrap());
        }
        if user_id.is_some() {
            separated.push("user_id = ");
            separated.push_bind_unseparated(user_id.unwrap());
        }
        if client.is_some() {
            separated.push("client = ");
            separated.push_bind_unseparated(client.unwrap());
        }
        if device.is_some() {
            separated.push("device = ");
            separated.push_bind_unseparated(device.unwrap());
        }
        if device_id.is_some() {
            separated.push("device_id = ");
            separated.push_bind_unseparated(device_id.unwrap());
        }
        if client_version.is_some() {
            separated.push("client_version = ");
            separated.push_bind_unseparated(client_version.unwrap());
        }
        if user_agent.is_some() {
            separated.push("user_agent = ");
            separated.push_bind_unseparated(user_agent.unwrap());
        }
        if order_by.is_some() {
            separated.push("order_by = ");
            separated.push_bind_unseparated(order_by.unwrap());
        }
        if icon_url.is_some() {
            separated.push("icon_url = ");
            separated.push_bind_unseparated(icon_url.unwrap());
        }
        if browse_proxy_id.is_some() {
            separated.push("browse_proxy_id = ");
            separated.push_bind_unseparated(browse_proxy_id.unwrap());
        }
        if play_proxy_id.is_some() {
            separated.push("play_proxy_id = ");
            separated.push_bind_unseparated(play_proxy_id.unwrap());
        }
        if line_id.is_some() {
            separated.push("line_id = ");
            separated.push_bind_unseparated(line_id.unwrap());
        }
        if last_playback_time.is_some() {
            separated.push("last_playback_time = ");
            separated.push_bind_unseparated(last_playback_time.unwrap());
        }
        if keep_alive_days.is_some() {
            separated.push("keep_alive_days = ");
            separated.push_bind_unseparated(keep_alive_days.unwrap());
        }
        if disabled.is_some() {
            separated.push("disabled = ");
            separated.push_bind_unseparated(disabled.unwrap());
        }
        qb.push(" where id = ");
        qb.push_bind(entity_id.unwrap());
    });

    tracing::debug!("sqlx: 更新emby服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}

pub async fn update_order(
    removed_id: String,
    removed_index: i32,
    added_index: i32,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = if removed_index > added_index {
        db_execute!(&state.db_pool, |qb| {
            qb.push("update emby_server set order_by = order_by + 1 where order_by >= ");
            qb.push_bind(added_index);
            qb.push(" and order_by < ");
            qb.push_bind(removed_index);
        })
    } else {
        db_execute!(&state.db_pool, |qb| {
            qb.push("update emby_server set order_by = order_by - 1 where order_by > ");
            qb.push_bind(removed_index);
            qb.push(" and order_by <= ");
            qb.push_bind(added_index);
        })
    };

    tracing::debug!("sqlx: 更新emby服务器排序: {:?}", res);
    if res.is_ok() {
        super::emby_server_mapper::update_by_id(
            EmbyServer {
                id: Some(removed_id),
                order_by: Some(added_index),
                ..Default::default()
            },
            &state,
        )
        .await?;
    }
    anyhow::Ok(res?)
}

pub async fn defer_order(
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("update emby_server set order_by = order_by + 1");
    });

    tracing::debug!("sqlx: 推后emby服务器排序: {:?}", res);
    anyhow::Ok(res?)
}

pub async fn delete_by_id(
    id: String,
    state: &tauri::State<'_, AppState>,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(&state.db_pool, |qb| {
        qb.push("delete from emby_server where id = ");
        qb.push_bind(&id);
    });

    tracing::debug!("sqlx: 删除emby服务器: {:?}", res);
    if res.is_ok() {
        refresh_cache(&id, state).await?;
    }
    anyhow::Ok(res?)
}
