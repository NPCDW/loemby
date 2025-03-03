use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::{Path, State}, response::IntoResponse, routing::get, Router};
use tokio::sync::RwLock;

pub async fn init_proxy_svc(axum_app_state: Arc<RwLock<Option<AxumAppState>>>) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    tracing::info!("axum listening on {:?}", actual_port);
    
    *axum_app_state.write().await = Some(AxumAppState {
        port: actual_port,
        connect: Arc::new(RwLock::new(HashMap::new())),
    });

    let router = Router::new()
        .route("/stream/{id}", get(stream))
        .with_state(axum_app_state);

    axum::serve(listener, router).await?;

    anyhow::Ok(())
}

async fn stream(headers: axum::http::HeaderMap, State(app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Path(id): Path<String>) -> axum::response::Response {
    tracing::debug!("stream: {} {:?}", id, headers);
    let app_state = app_state.read().await.clone().unwrap();
    let connect = app_state.connect.read().await;
    let connect = connect.get(&id).clone();
    if connect.is_none() {
        tracing::error!("没有找到 {} 对应的流媒体", &id);
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::http::HeaderMap::new(),
            axum::body::Body::empty()
        ).into_response();
    }
    let connect = connect.unwrap();

    let mut client = reqwest::Client::builder();
    if let Some(proxy_url) = connect.proxy_url.clone() {
        let proxy = reqwest::Proxy::all(&proxy_url);
        if proxy.is_err() {
            tracing::error!("{} 代理不正确 {:?}", proxy_url, proxy);
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::http::HeaderMap::new(),
                axum::body::Body::empty()
            ).into_response();
        }
        client = client.proxy(proxy.unwrap());
    }
    let client = client.build();
    if client.is_err() {
        tracing::error!("{} 创建媒体流请求失败 {:?}", &id, client);
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::empty()
        ).into_response();
    }
    let client = client.unwrap();
    let mut req_headers = headers.clone();
    req_headers.remove( axum::http::header::HOST);
    req_headers.insert( axum::http::header::USER_AGENT, connect.user_agent.clone().parse().unwrap() );
    let res = client
        .get(connect.stream_url.clone())
        .headers(req_headers)
        .send()
        .await;
    if res.is_err() {
        tracing::error!("{} 请求媒体流失败 {:?}", &id, res);
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::empty()
        ).into_response();
    }
    let res = res.unwrap();
    (
        res.status(),
        res.headers().clone(),
        axum::body::Body::from_stream(res.bytes_stream())
    ).into_response()
}

#[derive(Clone)]
pub struct AxumAppStateConnect {
    pub stream_url: String,
    pub proxy_url: Option<String>,
    pub user_agent: String,
}

#[derive(Clone)]
pub struct AxumAppState {
    pub port: u16,
    pub connect: Arc::<RwLock<HashMap<String, AxumAppStateConnect>>>,
}
