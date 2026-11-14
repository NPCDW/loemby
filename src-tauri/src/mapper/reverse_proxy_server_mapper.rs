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

/// 拼接完整 HTTP(S) 目标地址，保留原始编码并避免重复添加同一反代前缀
pub fn apply_reverse_proxy_url(target_url: &str, reverse_proxy_url: Option<&str>) -> String {
    let Some(reverse_proxy_url) = reverse_proxy_url else {
        return target_url.to_string();
    };
    let reverse_proxy_url = normalize_url(reverse_proxy_url);
    if reverse_proxy_url.is_empty() || target_url.starts_with(&reverse_proxy_url) {
        return target_url.to_string();
    }
    match url::Url::parse(target_url) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => format!("{}{}", reverse_proxy_url, target_url),
        _ => target_url.to_string(),
    }
}

pub async fn resolve_reverse_url(
    target_url: &str,
    reverse_proxy_id: Option<String>,
    state: &tauri::State<'_, AppState>,
) -> String {
    let reverse_proxy_url = get_reverse_proxy_url(reverse_proxy_id, state).await;
    apply_reverse_proxy_url(target_url, reverse_proxy_url.as_deref())
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

#[cfg(test)]
mod tests {
    use super::apply_reverse_proxy_url;

    #[test]
    fn reverse_url_preserves_targets_without_a_proxy() {
        let target = "https://example.com/path%2Fitem?q=a%2Bb&redirect_uri=http://127.0.0.1:1234/auth";
        for prefix in [None, Some(""), Some("   ")] {
            assert_eq!(apply_reverse_proxy_url(target, prefix), target);
        }
    }

    #[test]
    fn reverse_url_normalizes_prefix_without_reencoding_the_target() {
        let target = "https://example.com/path%2Fitem?q=a%2Bb&empty=&unicode=%E4%B8%AD#part";
        let expected = format!("https://proxy.example.com/gateway/{}", target);
        for prefix in ["https://proxy.example.com/gateway", " https://proxy.example.com/gateway/ "] {
            assert_eq!(apply_reverse_proxy_url(target, Some(prefix)), expected);
        }
    }

    #[test]
    fn reverse_url_does_not_duplicate_a_prefix_or_match_a_similar_host() {
        let prefix = "https://proxy.example.com";
        let target = "https://example.com/path";
        let routed = apply_reverse_proxy_url(target, Some(prefix));
        assert_eq!(apply_reverse_proxy_url(&routed, Some(prefix)), routed);
        assert_eq!(
            apply_reverse_proxy_url("https://proxy.example.com.evil.test/path", Some(prefix)),
            "https://proxy.example.com/https://proxy.example.com.evil.test/path",
        );
    }

    #[test]
    fn reverse_url_only_wraps_absolute_http_targets() {
        for target in ["file:///tmp/icon.png", "data:image/png;base64,xxx", "/stream/audio/123", "not a URL"] {
            assert_eq!(apply_reverse_proxy_url(target, Some("https://proxy.example.com/")), target);
        }
        assert_eq!(
            apply_reverse_proxy_url("http://yamtrack.lan/webhook/emby/abc", Some("https://proxy.example.com/")),
            "https://proxy.example.com/http://yamtrack.lan/webhook/emby/abc",
        );
    }

    #[tokio::test]
    async fn reverse_requests_preserve_method_headers_and_oauth_body() {
        use axum::{body::Bytes, http::{HeaderMap, Method, Uri}, routing::any, Router};
        use tokio::time::{timeout, Duration};

        let (sender, mut receiver) = tokio::sync::mpsc::channel(2);
        let router = Router::new().fallback(any(move |method: Method, uri: Uri, headers: HeaderMap, body: Bytes| {
            let sender = sender.clone();
            async move {
                sender.send((method, uri, headers, body)).await.unwrap();
                "ok"
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let prefix = format!("http://{}/gateway/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let target = "https://icons.example.com/library%2Ftest.json?q=a%2Bb";
        let response = timeout(Duration::from_secs(5), client
            .get(apply_reverse_proxy_url(target, Some(&prefix)))
            .header(reqwest::header::USER_AGENT, "loemby/test")
            .send()).await.unwrap().unwrap();
        assert!(response.status().is_success());
        let (method, uri, headers, body) = receiver.recv().await.unwrap();
        assert_eq!(method, Method::GET);
        assert_eq!(uri.to_string(), format!("/gateway/{}", target));
        assert_eq!(headers["user-agent"], "loemby/test");
        assert!(body.is_empty());

        let body = serde_json::json!({
            "code": "test-code",
            "refresh_token": "test-refresh",
            "redirect_uri": "http://127.0.0.1:1234/trakt_auth",
        }).to_string();
        let response = timeout(Duration::from_secs(5), client
            .post(apply_reverse_proxy_url("https://token-exchange.i101.workers.dev/trakt", Some(&prefix)))
            .header(reqwest::header::CONTENT_TYPE, "application/json; charset=UTF-8")
            .header(reqwest::header::AUTHORIZATION, "Bearer test-token")
            .header("trakt-api-key", "test-key")
            .body(body.clone())
            .send()).await.unwrap().unwrap();
        assert!(response.status().is_success());
        let (method, uri, headers, received_body) = receiver.recv().await.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(uri.to_string(), "/gateway/https://token-exchange.i101.workers.dev/trakt");
        assert_eq!(headers["content-type"], "application/json; charset=UTF-8");
        assert_eq!(headers["authorization"], "Bearer test-token");
        assert_eq!(headers["trakt-api-key"], "test-key");
        assert_eq!(received_body.as_ref(), body.as_bytes());
        server.abort();
    }
}
