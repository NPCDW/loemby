use serde::{Deserialize, Serialize};

use crate::{config::db_pool::DbPool, db_execute, db_fetch_all, db_fetch_optional};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct EmbyLine {
    pub id: Option<String>,
    pub create_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub name: Option<String>,
    pub emby_server_id: Option<String>,
    pub emby_server_name: Option<String>,
    pub base_url: Option<String>,
    pub browse_proxy_id: Option<String>,
    pub play_proxy_id: Option<String>,
}

pub async fn get_by_id(id: String, pool: &DbPool) -> anyhow::Result<Option<EmbyLine>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from emby_line where id = ");
            qb.push_bind(id);
        },
        EmbyLine
    )?;
    tracing::debug!("sqlx: 查询线路: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_emby_server_line(
    emby_server_id: String,
    pool: &DbPool,
) -> anyhow::Result<Vec<EmbyLine>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from emby_line where emby_server_id = ");
            qb.push_bind(emby_server_id);
        },
        EmbyLine
    )?;
    tracing::debug!("sqlx: 查询单个emby所有线路: {:?}", res);
    anyhow::Ok(res)
}

pub async fn list_all(pool: &DbPool) -> anyhow::Result<Vec<EmbyLine>> {
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from emby_line");
        },
        EmbyLine
    )?;
    tracing::debug!("sqlx: 查询所有线路: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create(
    entity: EmbyLine,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let name = entity.name.clone();
    let emby_server_id = entity.emby_server_id.clone();
    let emby_server_name = entity.emby_server_name.clone();
    let base_url = entity.base_url.clone();
    let browse_proxy_id = entity.browse_proxy_id.clone();
    let play_proxy_id = entity.play_proxy_id.clone();

    let res = db_execute!(pool, |qb| {
        qb.push("insert into emby_line(");
        let mut separated = qb.separated(", ");
        separated.push("id");
        separated.push("create_time");
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
        qb.push(")  values(");
        let mut separated = qb.separated(", ");
        separated.push_bind(id);
        separated.push_bind(chrono::Local::now().fixed_offset());
        if name.is_some() {
            separated.push_bind(name.unwrap());
        }
        if emby_server_id.is_some() {
            separated.push_bind(emby_server_id.unwrap());
        }
        if emby_server_name.is_some() {
            separated.push_bind(emby_server_name.unwrap());
        }
        if base_url.is_some() {
            separated.push_bind(base_url.unwrap());
        }
        if browse_proxy_id.is_some() {
            separated.push_bind(browse_proxy_id.unwrap());
        }
        if play_proxy_id.is_some() {
            separated.push_bind(play_proxy_id.unwrap());
        }
        qb.push(")");
    })?;

    tracing::debug!("sqlx: 添加线路: {:?}", res);
    anyhow::Ok(res)
}

pub async fn update_by_id(
    entity: EmbyLine,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let name = entity.name.clone();
    let emby_server_id = entity.emby_server_id.clone();
    let emby_server_name = entity.emby_server_name.clone();
    let base_url = entity.base_url.clone();
    let browse_proxy_id = entity.browse_proxy_id.clone();
    let play_proxy_id = entity.play_proxy_id.clone();
    let entity_id = entity.id.clone();

    let res = db_execute!(pool, |qb| {
        qb.push("update emby_line set ");
        let mut separated = qb.separated(", ");
        if name.is_some() {
            separated.push("name = ");
            separated.push_bind_unseparated(name.unwrap());
        }
        if emby_server_id.is_some() {
            separated.push("emby_server_id = ");
            separated.push_bind_unseparated(emby_server_id.unwrap());
        }
        if emby_server_name.is_some() {
            separated.push("emby_server_name = ");
            separated.push_bind_unseparated(emby_server_name.unwrap());
        }
        if base_url.is_some() {
            separated.push("base_url = ");
            separated.push_bind_unseparated(base_url.unwrap());
        }
        if browse_proxy_id.is_some() {
            separated.push("browse_proxy_id = ");
            separated.push_bind_unseparated(browse_proxy_id.unwrap());
        }
        if play_proxy_id.is_some() {
            separated.push("play_proxy_id = ");
            separated.push_bind_unseparated(play_proxy_id.unwrap());
        }
        qb.push(" where id = ");
        qb.push_bind(entity_id.unwrap());
    })?;

    tracing::debug!("sqlx: 更新线路: {:?}", res);
    anyhow::Ok(res)
}

pub async fn update_line_emby_server_name(
    emby_server_id: String,
    emby_server_name: String,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(pool, |qb| {
        qb.push("update emby_line set emby_server_name = ");
        qb.push_bind(emby_server_name);
        qb.push(" where emby_server_id = ");
        qb.push_bind(emby_server_id);
    })?;

    tracing::debug!("sqlx: 更新线路emby服务名: {:?}", res);
    anyhow::Ok(res)
}

pub async fn delete_line_by_emby_server(
    emby_server_id: String,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(pool, |qb| {
        qb.push("delete from emby_line where emby_server_id = ");
        qb.push_bind(emby_server_id);
    })?;

    tracing::debug!("sqlx: 删除线路: {:?}", res);
    anyhow::Ok(res)
}

pub async fn delete_by_id(
    id: String,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(pool, |qb| {
        qb.push("delete from emby_line where id = ");
        qb.push_bind(id);
    })?;

    tracing::debug!("sqlx: 删除线路: {:?}", res);
    anyhow::Ok(res)
}
