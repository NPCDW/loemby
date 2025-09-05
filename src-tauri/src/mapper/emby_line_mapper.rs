use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct EmbyLine {
    pub id: Option<String>,
    pub create_time: Option<String>,
    pub name: Option<String>,
    pub emby_server_id: Option<String>,
    pub emby_server_name: Option<String>,
    pub base_url: Option<String>,
    pub browse_proxy_id: Option<String>,
    pub play_proxy_id: Option<String>,
}

pub async fn get_by_id(id: String, pool: &Pool<Sqlite>) -> Result<EmbyLine, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_line where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build_query_as::<EmbyLine>();
    let sql = query.sql();
    let res = query.fetch_one(pool).await;
    tracing::debug!("sqlx: 查询线路: {} {:?}", sql, res);
    res
}

pub async fn list_emby_server_line(emby_server_id: String, pool: &Pool<Sqlite>) -> Result<Vec<EmbyLine>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_line where emby_server_id = ");
    query_builder.push_bind(emby_server_id);
    let query = query_builder.build_query_as::<EmbyLine>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询单个emby所有线路: {} {:?}", sql, res);
    res
}

pub async fn list_all(pool: &Pool<Sqlite>) -> Result<Vec<EmbyLine>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_line");
    let query = query_builder.build_query_as::<EmbyLine>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有线路: {} {:?}", sql, res);
    res
}

pub async fn create(entity: EmbyLine, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into emby_line(");
    let mut separated = query_builder.separated(", ");
    separated.push("id");
    if entity.name.is_some() {
        separated.push("name");
    }
    if entity.emby_server_id.is_some() {
        separated.push("emby_server_id");
    }
    if entity.emby_server_name.is_some() {
        separated.push("emby_server_name");
    }
    if entity.base_url.is_some() {
        separated.push("base_url");
    }
    if entity.browse_proxy_id.is_some() {
        separated.push("browse_proxy_id");
    }
    if entity.play_proxy_id.is_some() {
        separated.push("play_proxy_id");
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
    if entity.emby_server_id.is_some() {
        separated.push_bind(entity.emby_server_id.unwrap());
    }
    if entity.emby_server_name.is_some() {
        separated.push_bind(entity.emby_server_name.unwrap());
    }
    if entity.base_url.is_some() {
        separated.push_bind(entity.base_url.unwrap());
    }
    if entity.browse_proxy_id.is_some() {
        separated.push_bind(entity.browse_proxy_id.unwrap());
    }
    if entity.play_proxy_id.is_some() {
        separated.push_bind(entity.play_proxy_id.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 添加线路: {} {:?}", sql, res);
    res
}

pub async fn update_by_id(entity: EmbyLine, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update emby_line set ");
    let mut separated = query_builder.separated(", ");
    if entity.name.is_some() {
        separated.push("name = ").push_bind_unseparated(entity.name.unwrap());
    }
    if entity.emby_server_id.is_some() {
        separated.push("emby_server_id = ").push_bind_unseparated(entity.emby_server_id.unwrap());
    }
    if entity.emby_server_name.is_some() {
        separated.push("emby_server_name = ").push_bind_unseparated(entity.emby_server_name.unwrap());
    }
    if entity.base_url.is_some() {
        separated.push("base_url = ").push_bind_unseparated(entity.base_url.unwrap());
    }
    if entity.browse_proxy_id.is_some() {
        separated.push("browse_proxy_id = ").push_bind_unseparated(entity.browse_proxy_id.unwrap());
    }
    if entity.play_proxy_id.is_some() {
        separated.push("play_proxy_id = ").push_bind_unseparated(entity.play_proxy_id.unwrap());
    }
    query_builder.push(" where id = ").push_bind(entity.id.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新线路: {} {:?}", sql, res);
    res
}

pub async fn update_line_emby_server_name(emby_server_id: String, emby_server_name: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update emby_line set emby_server_name = ");
    query_builder.push_bind(emby_server_name);
    query_builder.push(" where emby_server_id = ");
    query_builder.push_bind(emby_server_id);

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新线路emby服务名: {} {:?}", sql, res);
    res
}

pub async fn delete_line_by_emby_server(emby_server_id: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder = QueryBuilder::new("delete from emby_line where emby_server_id = ");
    query_builder.push_bind(emby_server_id);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 删除线路: {} {:?}", sql, res);
    res
}

pub async fn delete_by_id(id: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder = QueryBuilder::new("delete from emby_line where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 删除线路: {} {:?}", sql, res);
    res
}
