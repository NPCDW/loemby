use serde::{Deserialize, Serialize};

use crate::{config::db_pool::DbPool, db_execute, db_fetch_all, db_fetch_optional};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct EmbyIconLibrary {
    pub id: Option<String>,
    pub create_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub name: Option<String>,
    pub url: Option<String>,
}

pub async fn get_by_id(id: String, pool: &DbPool) -> anyhow::Result<Option<EmbyIconLibrary>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from emby_icon_library where id = ");
            qb.push_bind(id);
        },
        EmbyIconLibrary
    )?;
    tracing::debug!("sqlx: 查询图标库: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_all(pool: &DbPool) -> anyhow::Result<Vec<EmbyIconLibrary>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from emby_icon_library");
        },
        EmbyIconLibrary
    )?;
    tracing::debug!("sqlx: 查询所有图标库: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create(
    entity: EmbyIconLibrary,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let name = entity.name.clone();
    let url = entity.url.clone();

    let res = db_execute!(pool, |qb| {
        qb.push("insert into emby_icon_library(");
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
        separated.push_bind(id);
        separated.push_bind(chrono::Local::now().fixed_offset());
        if name.is_some() {
            separated.push_bind(name.unwrap());
        }
        if url.is_some() {
            separated.push_bind(url.unwrap());
        }
        qb.push(")");
    })?;

    tracing::debug!("sqlx: 添加图标库: {:?}", res);
    anyhow::Ok(res)
}

pub async fn update_by_id(
    entity: EmbyIconLibrary,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let name = entity.name.clone();
    let url = entity.url.clone();
    let entity_id = entity.id.clone();

    let res = db_execute!(pool, |qb| {
        qb.push("update emby_icon_library set ");
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
    })?;

    tracing::debug!("sqlx: 更新图标库: {:?}", res);
    anyhow::Ok(res)
}

pub async fn delete_by_id(
    id: String,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(pool, |qb| {
        qb.push("delete from emby_icon_library where id = ");
        qb.push_bind(id);
    })?;

    tracing::debug!("sqlx: 删除图标库: {:?}", res);
    anyhow::Ok(res)
}
