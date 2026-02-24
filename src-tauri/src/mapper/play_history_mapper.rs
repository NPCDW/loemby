use serde::{Deserialize, Serialize};

use crate::{
    config::db_pool::DbPool, controller::play_history_ctl::PagePlayHistoryParam, db_execute,
    db_fetch_all, db_fetch_optional,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct PlayHistory {
    pub id: Option<String>,
    pub create_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub update_time: Option<chrono::DateTime<chrono::FixedOffset>>,

    pub emby_server_id: Option<String>,
    pub emby_server_name: Option<String>,
    pub item_type: Option<String>,
    pub item_id: Option<String>,
    pub item_name: Option<String>,
    pub series_id: Option<String>,
    pub series_name: Option<String>,
    pub played_duration: Option<i32>,
    pub pinned: Option<i32>,
}

pub async fn page(
    param: PagePlayHistoryParam,
    pool: &DbPool,
) -> anyhow::Result<(u32, Vec<PlayHistory>)> {
    let mut conditions = Vec::new();
    let mut values = Vec::new();
    if let Some(emby_server_id) = param.emby_server_id {
        conditions.push("emby_server_id = ");
        values.push(emby_server_id);
    }
    if let Some(series_name) = param.series_name {
        conditions.push("series_name LIKE ");
        values.push(series_name);
    }
    if let Some(item_name) = param.item_name {
        conditions.push("item_name LIKE ");
        values.push(item_name);
    }

    // Count query
    let count_res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select count(*) as total from play_history");
            if !conditions.is_empty() {
                let mut separated = qb.push(" WHERE ").separated(" and ");
                for (i, key) in conditions.iter().enumerate() {
                    separated.push(key);
                    if key.contains("series_name") || key.contains("item_name") {
                        separated.push_bind_unseparated(format!("%{}%", values[i].clone()));
                    } else {
                        separated.push_bind_unseparated(values[i].clone());
                    }
                }
            }
        },
        (i64,)
    )?;

    tracing::debug!("sqlx: 查询播放历史数量: {:?}", count_res);
    let count: (i64,) = count_res.unwrap_or((0,));
    if count.0 <= 0 {
        return anyhow::Ok((0, vec![]));
    }

    // List query
    let res = db_fetch_all!(
        pool,
        |qb| {
            qb.push("select * from play_history");
            if !conditions.is_empty() {
                let mut separated = qb.push(" WHERE ").separated(" and ");
                for (i, key) in conditions.iter().enumerate() {
                    separated.push(key);
                    if key.contains("series_name") || key.contains("item_name") {
                        separated.push_bind_unseparated(format!("%{}%", values[i].clone()));
                    } else {
                        separated.push_bind_unseparated(values[i].clone());
                    }
                }
            }
            qb.push(" order by pinned desc, update_time desc limit ");
            qb.push_bind(param.page_size);
            qb.push(" offset ");
            qb.push_bind((param.page_number - 1) * param.page_size);
        },
        PlayHistory
    )?;

    tracing::debug!("sqlx: 查询播放历史列表: {:?}", res);
    anyhow::Ok((count.0 as u32, res))
}

pub async fn get(
    emby_server_id: String,
    item_id: String,
    pool: &DbPool,
) -> anyhow::Result<Option<PlayHistory>> {
    let res = db_fetch_optional!(
        pool,
        |qb| {
            qb.push("select * from play_history where emby_server_id = ");
            qb.push_bind(emby_server_id);
            qb.push(" and item_id = ");
            qb.push_bind(item_id);
        },
        PlayHistory
    )?;
    tracing::debug!("sqlx: 查询播放历史: {:?}", res);
    anyhow::Ok(res)
}

pub async fn create(
    entity: PlayHistory,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let id = if entity.id.is_some() {
        entity.id.clone().unwrap()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let emby_server_id = entity.emby_server_id.clone();
    let emby_server_name = entity.emby_server_name.clone();
    let item_type = entity.item_type.clone();
    let item_id = entity.item_id.clone();
    let item_name = entity.item_name.clone();
    let series_id = entity.series_id.clone();
    let series_name = entity.series_name.clone();
    let played_duration = entity.played_duration.clone();
    let pinned = entity.pinned.clone();

    let res = db_execute!(pool, |qb| {
        qb.push("insert into play_history(");
        let mut separated = qb.separated(", ");
        separated.push("id");
        separated.push("create_time");
        separated.push("update_time");
        if entity.emby_server_id.is_some() {
            separated.push("emby_server_id");
        }
        if entity.emby_server_name.is_some() {
            separated.push("emby_server_name");
        }
        if entity.item_type.is_some() {
            separated.push("item_type");
        }
        if entity.item_id.is_some() {
            separated.push("item_id");
        }
        if entity.item_name.is_some() {
            separated.push("item_name");
        }
        if entity.series_id.is_some() {
            separated.push("series_id");
        }
        if entity.series_name.is_some() {
            separated.push("series_name");
        }
        if entity.played_duration.is_some() {
            separated.push("played_duration");
        }
        if entity.pinned.is_some() {
            separated.push("pinned");
        }
        qb.push(")  values(");
        let mut separated = qb.separated(", ");
        separated.push_bind(id);
        separated.push_bind(chrono::Local::now().fixed_offset());
        separated.push_bind(chrono::Local::now().fixed_offset());
        if emby_server_id.is_some() {
            separated.push_bind(emby_server_id.unwrap());
        }
        if emby_server_name.is_some() {
            separated.push_bind(emby_server_name.unwrap());
        }
        if item_type.is_some() {
            separated.push_bind(item_type.unwrap());
        }
        if item_id.is_some() {
            separated.push_bind(item_id.unwrap());
        }
        if item_name.is_some() {
            separated.push_bind(item_name.unwrap());
        }
        if series_id.is_some() {
            separated.push_bind(series_id.unwrap());
        }
        if series_name.is_some() {
            separated.push_bind(series_name.unwrap());
        }
        if played_duration.is_some() {
            separated.push_bind(played_duration.unwrap());
        }
        if pinned.is_some() {
            separated.push_bind(pinned.unwrap());
        }
        qb.push(")");
    })?;

    tracing::debug!("sqlx: 添加播放历史: {:?}", res);
    anyhow::Ok(res)
}

pub async fn update_by_id(
    entity: PlayHistory,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let emby_server_id = entity.emby_server_id.clone();
    let emby_server_name = entity.emby_server_name.clone();
    let item_type = entity.item_type.clone();
    let item_id = entity.item_id.clone();
    let item_name = entity.item_name.clone();
    let series_id = entity.series_id.clone();
    let series_name = entity.series_name.clone();
    let played_duration = entity.played_duration.clone();
    let pinned = entity.pinned.clone();
    let entity_id = entity.id.clone();

    let res = db_execute!(pool, |qb| {
        qb.push("update play_history set ");
        let mut separated = qb.separated(", ");
        separated.push("update_time = ");
        separated.push_bind_unseparated(chrono::Local::now().fixed_offset());
        if emby_server_id.is_some() {
            separated.push("emby_server_id = ");
            separated.push_bind_unseparated(emby_server_id.unwrap());
        }
        if emby_server_name.is_some() {
            separated.push("emby_server_name = ");
            separated.push_bind_unseparated(emby_server_name.unwrap());
        }
        if item_type.is_some() {
            separated.push("item_type = ");
            separated.push_bind_unseparated(item_type.unwrap());
        }
        if item_id.is_some() {
            separated.push("item_id = ");
            separated.push_bind_unseparated(item_id.unwrap());
        }
        if item_name.is_some() {
            separated.push("item_name = ");
            separated.push_bind_unseparated(item_name.unwrap());
        }
        if series_id.is_some() {
            separated.push("series_id = ");
            separated.push_bind_unseparated(series_id.unwrap());
        }
        if series_name.is_some() {
            separated.push("series_name = ");
            separated.push_bind_unseparated(series_name.unwrap());
        }
        if played_duration.is_some() {
            separated.push("played_duration = ");
            separated.push_bind_unseparated(played_duration.unwrap());
        }
        if pinned.is_some() {
            separated.push("pinned = ");
            separated.push_bind_unseparated(pinned.unwrap());
        }
        qb.push(" where id = ");
        qb.push_bind(entity_id.unwrap());
    })?;

    tracing::debug!("sqlx: 更新播放历史: {:?}", res);
    anyhow::Ok(res)
}

pub async fn cancel_pinned(
    emby_server_id: String,
    series_id: String,
    pool: &DbPool,
) -> anyhow::Result<crate::config::db_pool::DbQueryResult> {
    let res = db_execute!(pool, |qb| {
        qb.push("update play_history set pinned = 0 where pinned = 1 and emby_server_id = ");
        qb.push_bind(emby_server_id);
        qb.push(" and series_id = ");
        qb.push_bind(series_id);
    })?;

    tracing::debug!("sqlx: 更新播放历史: {:?}", res);
    anyhow::Ok(res)
}
