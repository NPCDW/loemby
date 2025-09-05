use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

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

pub async fn get_by_id(id: String, pool: &Pool<Sqlite>) -> Result<Option<ProxyServer>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from proxy_server where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build_query_as::<ProxyServer>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询代理服务器: {} {:?}", sql, res);
    res
}

pub async fn list_all(pool: &Pool<Sqlite>) -> Result<Vec<ProxyServer>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from proxy_server");
    let query = query_builder.build_query_as::<ProxyServer>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有代理服务器: {} {:?}", sql, res);
    res
}

pub async fn create(entity: ProxyServer, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
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
    if entity.id.is_some() {
        separated.push_bind(entity.id.unwrap());
    } else {
        separated.push_bind(uuid::Uuid::new_v4().to_string());
    }
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
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 添加代理服务器: {} {:?}", sql, res);
    res
}

pub async fn update_by_id(entity: ProxyServer, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
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
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新代理服务器: {} {:?}", sql, res);
    res
}

pub async fn delete_by_id(id: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder = QueryBuilder::new("delete from proxy_server where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 删除代理服务器: {} {:?}", sql, res);
    res
}
