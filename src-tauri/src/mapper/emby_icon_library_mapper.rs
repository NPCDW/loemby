use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct EmbyIconLibrary {
    pub id: Option<String>,
    pub create_time: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

pub async fn get_by_id(id: String, pool: &Pool<Sqlite>) -> anyhow::Result<Option<EmbyIconLibrary>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_icon_library where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build_query_as::<EmbyIconLibrary>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询图标库: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn list_all(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<EmbyIconLibrary>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from emby_icon_library");
    let query = query_builder.build_query_as::<EmbyIconLibrary>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询所有图标库: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn create(entity: EmbyIconLibrary, pool: &Pool<Sqlite>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into emby_icon_library(");
    let mut separated = query_builder.separated(", ");
    separated.push("id");
    if entity.name.is_some() {
        separated.push("name");
    }
    if entity.url.is_some() {
        separated.push("url");
    }
    query_builder.push(")  values(");
    let mut separated = query_builder.separated(", ");
    separated.push_bind(id);
    if entity.name.is_some() {
        separated.push_bind(entity.name.unwrap());
    }
    if entity.url.is_some() {
        separated.push_bind(entity.url.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 添加图标库: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn update_by_id(entity: EmbyIconLibrary, pool: &Pool<Sqlite>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update emby_icon_library set ");
    let mut separated = query_builder.separated(", ");
    if entity.name.is_some() {
        separated.push("name = ").push_bind_unseparated(entity.name.unwrap());
    }
    if entity.url.is_some() {
        separated.push("url = ").push_bind_unseparated(entity.url.unwrap());
    }
    query_builder.push(" where id = ").push_bind(entity.id.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新图标库: {} {:?}", sql, res);
    anyhow::Ok(res?)
}

pub async fn delete_by_id(id: String, pool: &Pool<Sqlite>) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult> {
    let mut query_builder = QueryBuilder::new("delete from emby_icon_library where id = ");
    query_builder.push_bind(id);
    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 删除图标库: {} {:?}", sql, res);
    anyhow::Ok(res?)
}
