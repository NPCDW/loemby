use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{response::IntoResponse, routing::get, Router};
use tokio::sync::RwLock;

pub async fn init_proxy_svc() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    tracing::info!("axum listening on {:?}", actual_port);
    
    let app_state = AppState {
        port: actual_port,
        path: Arc::new(RwLock::new(HashMap::new())),
    };

    let router = Router::new()
        .route("/video", get(video_stream))
        .with_state(app_state);

    axum::serve(listener, router).await?;

    anyhow::Ok(())
}

async fn video_stream(headers: axum::http::HeaderMap) -> axum::response::Response {
    println!("进入{:?}", headers);
    let mut req_headers = headers.clone();
    req_headers.remove( axum::http::header::HOST);
    let client = reqwest::Client::new();
    let res = client
        .get("https://www.bilibili.com/video/BV17x411Y77E")
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
pub struct AppState {
    pub port: u16,
    pub path: Arc<RwLock<HashMap<String, String>>>,
}
