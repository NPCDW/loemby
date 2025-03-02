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
        path: HashMap::new(),
    });

    let router = Router::new()
        .route("/video/{video_id}", get(video_stream))
        .with_state(axum_app_state);

    axum::serve(listener, router).await?;

    anyhow::Ok(())
}

async fn video_stream(headers: axum::http::HeaderMap, State(app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Path(video_id): Path<String>) -> axum::response::Response {
    tracing::debug!("video_stream: {} {:?}", video_id, headers);
    let app_state = app_state.read().await.clone().unwrap();
    let path = app_state.path.get(&video_id).clone();
    if path.is_none() {
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::http::HeaderMap::new(),
            axum::body::Body::empty()
        ).into_response();
    }
    let mut req_headers = headers.clone();
    req_headers.remove( axum::http::header::HOST);
    let client = reqwest::Client::builder()
        // .proxy(reqwest::Proxy::all(proxy_url)?)
        .build();
    if client.is_err() {
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::empty()
        ).into_response();
    }
    let client = client.unwrap();
    let res = client
        .get(path.unwrap())
        .headers(req_headers)
        .send()
        .await;
    let res = res.unwrap();
    (
        res.status(),
        res.headers().clone(),
        axum::body::Body::from_stream(res.bytes_stream())
    ).into_response()
}

#[derive(Clone)]
pub struct AxumAppState {
    pub port: u16,
    pub path: HashMap<String, String>,
}
