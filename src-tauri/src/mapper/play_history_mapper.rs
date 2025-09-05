use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct PlayHistory {
    pub id: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,

    pub emby_server_id: Option<String>,
    pub emby_server_name: Option<String>,
    pub item_type: Option<String>,
    pub item_id: Option<String>,
    pub item_name: Option<String>,
    pub series_id: Option<String>,
    pub series_name: Option<String>,
    pub played_duration: Option<u32>,
    pub pinned: Option<u32>,
}

pub async fn page(page_number: u32, page_size: u32, pool: &Pool<Sqlite>) -> anyhow::Result<(u32, Vec<PlayHistory>)> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select count(*) as total from play_history");
    let query = query_builder.build_query_as::<(i64,)>();
    let sql = query.sql();
    let res = query.fetch_one(pool).await;
    tracing::debug!("sqlx: 查询播放历史数量: {} {:?}", sql, res);
    let count: (i64,) = res?;
    if count.0 <= 0 {
        return anyhow::Ok((0, vec![]));
    }

    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from play_history order by pinned desc, update_time desc limit ");
    query_builder.push_bind(page_size);
    query_builder.push(" offset ");
    query_builder.push_bind((page_number - 1) * page_size);
    let query = query_builder.build_query_as::<PlayHistory>();
    let sql = query.sql();
    let res = query.fetch_all(pool).await;
    tracing::debug!("sqlx: 查询播放历史列表: {} {:?}", sql, res);
    anyhow::Ok((count.0 as u32, res?))
}

pub async fn get(emby_server_id: String, item_id: String, pool: &Pool<Sqlite>) -> Result<Option<PlayHistory>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from play_history where emby_server_id = ");
    query_builder.push_bind(emby_server_id);
    query_builder.push(" and item_id = ");
    query_builder.push_bind(item_id);
    let query = query_builder.build_query_as::<PlayHistory>();
    let sql = query.sql();
    let res = query.fetch_optional(pool).await;
    tracing::debug!("sqlx: 查询播放历史: {} {:?}", sql, res);
    res
}

pub async fn create(entity: PlayHistory, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into play_history(");
    let mut separated = query_builder.separated(", ");
    separated.push("id");
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
    query_builder.push(")  values(");
    let mut separated = query_builder.separated(", ");
    if entity.id.is_some() {
        separated.push_bind(entity.id.unwrap());
    } else {
        separated.push_bind(uuid::Uuid::new_v4().to_string());
    }
    if entity.emby_server_id.is_some() {
        separated.push_bind(entity.emby_server_id.unwrap());
    }
    if entity.emby_server_name.is_some() {
        separated.push_bind(entity.emby_server_name.unwrap());
    }
    if entity.item_type.is_some() {
        separated.push_bind(entity.item_type.unwrap());
    }
    if entity.item_id.is_some() {
        separated.push_bind(entity.item_id.unwrap());
    }
    if entity.item_name.is_some() {
        separated.push_bind(entity.item_name.unwrap());
    }
    if entity.series_id.is_some() {
        separated.push_bind(entity.series_id.unwrap());
    }
    if entity.series_name.is_some() {
        separated.push_bind(entity.series_name.unwrap());
    }
    if entity.played_duration.is_some() {
        separated.push_bind(entity.played_duration.unwrap());
    }
    if entity.pinned.is_some() {
        separated.push_bind(entity.pinned.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 添加播放历史: {} {:?}", sql, res);
    res
}

pub async fn update_by_id(entity: PlayHistory, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update play_history set ");
    let mut separated = query_builder.separated(", ");
    separated.push("update_time = ").push_bind_unseparated(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    if entity.emby_server_id.is_some() {
        separated.push("emby_server_id = ").push_bind_unseparated(entity.emby_server_id.unwrap());
    }
    if entity.emby_server_name.is_some() {
        separated.push("emby_server_name = ").push_bind_unseparated(entity.emby_server_name.unwrap());
    }
    if entity.item_type.is_some() {
        separated.push("item_type = ").push_bind_unseparated(entity.item_type.unwrap());
    }
    if entity.item_id.is_some() {
        separated.push("item_id = ").push_bind_unseparated(entity.item_id.unwrap());
    }
    if entity.item_name.is_some() {
        separated.push("item_name = ").push_bind_unseparated(entity.item_name.unwrap());
    }
    if entity.series_id.is_some() {
        separated.push("series_id = ").push_bind_unseparated(entity.series_id.unwrap());
    }
    if entity.series_name.is_some() {
        separated.push("series_name = ").push_bind_unseparated(entity.series_name.unwrap());
    }
    if entity.played_duration.is_some() {
        separated.push("played_duration = ").push_bind_unseparated(entity.played_duration.unwrap());
    }
    if entity.pinned.is_some() {
        separated.push("pinned = ").push_bind_unseparated(entity.pinned.unwrap());
    }
    query_builder.push(" where id = ").push_bind(entity.id.unwrap());

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新播放历史: {} {:?}", sql, res);
    res
}

pub async fn cancel_pinned(emby_server_id: String, series_id: String, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("update play_history set pinned = 0 where pinned = 1 and emby_server_id = ");
    query_builder.push_bind(emby_server_id);
    query_builder.push(" and series_id = ");
    query_builder.push_bind(series_id);

    let query = query_builder.build();
    let sql = query.sql();
    let res = query.execute(pool).await;
    tracing::debug!("sqlx: 更新播放历史: {} {:?}", sql, res);
    res
}
