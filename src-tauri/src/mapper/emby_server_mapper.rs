use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct EmbyServer {
    pub id: Option<String>,
    pub create_time: Option<String>,

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

    pub order_by: Option<u32>,
    pub icon_url: Option<String>,

    pub browse_proxy_id: Option<String>,
    pub play_proxy_id: Option<String>,
    pub line_id: Option<String>,

    pub last_playback_time: Option<String>,
    pub keep_alive_days: Option<u32>,

    pub disabled: Option<u32>,
}

pub async fn get_by_id(id: String, pool: &Pool<Sqlite>) -> Result<Option<EmbyServer>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_server where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build_query_as::<EmbyServer>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询emby服务器: {} {:?}", sql, res);
    res
}

pub async fn list_all(pool: &Pool<Sqlite>) -> Result<Vec<EmbyServer>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_server");
    let query = query_builder.build_query_as::<EmbyServer>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有emby服务器: {} {:?}", sql, res);
    res
}

pub async fn create(entity: EmbyServer, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into emby_server(");
    let mut separated = query_builder.separated(", ");
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
    query_builder.push(")  values(");
    let mut separated = query_builder.separated(", ");
    if entity.id.is_some() {
        separated.push_bind(entity.id.unwrap());
    } else {
        separated.push_bind(uuid::Uuid::new_v4().to_string());
    }
    if entity.base_url.is_some() {
        separated.push_bind(entity.base_url.unwrap());
    }
    if entity.username.is_some() {
        separated.push_bind(entity.username.unwrap());
    }
    if entity.password.is_some() {
        separated.push_bind(entity.password.unwrap());
    }
    if entity.server_name.is_some() {
        separated.push_bind(entity.server_name.unwrap());
    }
    if entity.server_id.is_some() {
        separated.push_bind(entity.server_id.unwrap());
    }
    if entity.auth_token.is_some() {
        separated.push_bind(entity.auth_token.unwrap());
    }
    if entity.user_id.is_some() {
        separated.push_bind(entity.user_id.unwrap());
    }
    if entity.client.is_some() {
        separated.push_bind(entity.client.unwrap());
    }
    if entity.device.is_some() {
        separated.push_bind(entity.device.unwrap());
    }
    if entity.device_id.is_some() {
        separated.push_bind(entity.device_id.unwrap());
    }
    if entity.client_version.is_some() {
        separated.push_bind(entity.client_version.unwrap());
    }
    if entity.user_agent.is_some() {
        separated.push_bind(entity.user_agent.unwrap());
    }
    if entity.order_by.is_some() {
        separated.push_bind(entity.order_by.unwrap());
    }
    if entity.icon_url.is_some() {
        separated.push_bind(entity.icon_url.unwrap());
    }
    if entity.browse_proxy_id.is_some() {
        separated.push_bind(entity.browse_proxy_id.unwrap());
    }
    if entity.play_proxy_id.is_some() {
        separated.push_bind(entity.play_proxy_id.unwrap());
    }
    if entity.line_id.is_some() {
        separated.push_bind(entity.line_id.unwrap());
    }
    if entity.last_playback_time.is_some() {
        separated.push_bind(entity.last_playback_time.unwrap());
    }
    if entity.keep_alive_days.is_some() {
        separated.push_bind(entity.keep_alive_days.unwrap());
    }
    if entity.disabled.is_some() {
        separated.push_bind(entity.disabled.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 添加emby服务器: {} {:?}", sql, res);
    res
}

pub async fn update_by_id(entity: EmbyServer, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update emby_server set ");
    let mut separated = query_builder.separated(", ");
    if entity.base_url.is_some() {
        separated.push("base_url = ").push_bind_unseparated(entity.base_url.unwrap());
    }
    if entity.username.is_some() {
        separated.push("username = ").push_bind_unseparated(entity.username.unwrap());
    }
    if entity.password.is_some() {
        separated.push("password = ").push_bind_unseparated(entity.password.unwrap());
    }
    if entity.server_name.is_some() {
        separated.push("server_name = ").push_bind_unseparated(entity.server_name.unwrap());
    }
    if entity.server_id.is_some() {
        separated.push("server_id = ").push_bind_unseparated(entity.server_id.unwrap());
    }
    if entity.auth_token.is_some() {
        separated.push("auth_token = ").push_bind_unseparated(entity.auth_token.unwrap());
    }
    if entity.user_id.is_some() {
        separated.push("user_id = ").push_bind_unseparated(entity.user_id.unwrap());
    }
    if entity.client.is_some() {
        separated.push("client = ").push_bind_unseparated(entity.client.unwrap());
    }
    if entity.device.is_some() {
        separated.push("device = ").push_bind_unseparated(entity.device.unwrap());
    }
    if entity.device_id.is_some() {
        separated.push("device_id = ").push_bind_unseparated(entity.device_id.unwrap());
    }
    if entity.client_version.is_some()  {
        separated.push("client_version = ").push_bind_unseparated(entity.client_version.unwrap());
    }
    if entity.user_agent.is_some() {
        separated.push("user_agent = ").push_bind_unseparated(entity.user_agent.unwrap());
    }
    if entity.order_by.is_some() {
        separated.push("order_by = ").push_bind_unseparated(entity.order_by.unwrap());
    }
    if entity.icon_url.is_some() {
        separated.push("icon_url = ").push_bind_unseparated(entity.icon_url.unwrap());
    }
    if entity.browse_proxy_id.is_some() {
        separated.push("browse_proxy_id = ").push_bind_unseparated(entity.browse_proxy_id.unwrap());
    }
    if entity.play_proxy_id.is_some() {
        separated.push("play_proxy_id = ").push_bind_unseparated(entity.play_proxy_id.unwrap());
    }
    if entity.line_id.is_some() {
        separated.push("line_id = ").push_bind_unseparated(entity.line_id.unwrap());
    }
    if entity.last_playback_time.is_some() {
        separated.push("last_playback_time = ").push_bind_unseparated(entity.last_playback_time.unwrap());
    }
    if entity.keep_alive_days.is_some() {
        separated.push("keep_alive_days = ").push_bind_unseparated(entity.keep_alive_days.unwrap());
    }
    if entity.disabled.is_some() {
        separated.push("disabled = ").push_bind_unseparated(entity.disabled.unwrap());
    }
    query_builder.push(" where id = ").push_bind(entity.id.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新emby服务器: {} {:?}", sql, res);
    res
}

pub async fn update_order(removed_id: String, removed_index: u32, added_index: u32, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite>;
    if removed_index > added_index {
        query_builder = QueryBuilder::new("update emby_server set order_by = order_by + 1 where order_by >= ");
        query_builder.push_bind(added_index);
        query_builder.push(" and order_by < ");
        query_builder.push_bind(removed_index);
    } else {
        query_builder = QueryBuilder::new("update emby_server set order_by = order_by - 1 where order_by > ");
        query_builder.push_bind(removed_index);
        query_builder.push(" and order_by <= ");
        query_builder.push_bind(added_index);
    }
    let query = query_builder.build();
    let sql = query.sql();
    let mut res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新emby服务器排序: {} {:?}", sql, res);
    if res.is_ok() {
        res = super::emby_server_mapper::update_by_id(EmbyServer { id: Some(removed_id), order_by: Some(added_index), ..Default::default() }, pool).await;
    }
    res
}

pub async fn defer_order(pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update emby_server set order_by = order_by + 1");
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 推后emby服务器排序: {} {:?}", sql, res);
    res
}

pub async fn delete_by_id(id: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder = QueryBuilder::new("delete from emby_server where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 删除emby服务器: {} {:?}", sql, res);
    res
}
