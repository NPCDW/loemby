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
        .route("/stream/{types}/{id}", get(stream))
        .with_state(axum_app_state);

    axum::serve(listener, router).await?;

    anyhow::Ok(())
}

async fn stream(headers: axum::http::HeaderMap, State(app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Path((types, id)): Path<(String, String)>) -> axum::response::Response {
    tracing::debug!("stream: {} {} {:?}", types, id, headers);
    let app_state = app_state.read().await.clone().unwrap();
    let connect = app_state.connect.read().await;
    let connect = match connect.get(&id).clone() {
        Some(connect) => connect,
        None => {
            tracing::error!("没有找到 {} 对应的流媒体", &id);
            return (
                axum::http::StatusCode::NOT_FOUND,
                axum::http::HeaderMap::new(),
                axum::body::Body::new("没有找到对应的流媒体".to_string())
            ).into_response();
        }
    };

    let client = connect.client.clone();
    let mut req_headers = headers.clone();
    req_headers.remove(axum::http::header::HOST);
    req_headers.remove(axum::http::header::USER_AGENT);
    req_headers.insert(axum::http::header::USER_AGENT, connect.user_agent.clone().parse().unwrap());
    let res = client
        .get(connect.stream_url.clone())
        .headers(req_headers.clone())
        .send()
        .await;
    tracing::debug!("stream: {} {} 媒体流响应 {:?}", types, &id, res);
    match res {
        Ok(res) => return (
            res.status(),
            res.headers().clone(),
            axum::body::Body::from_stream(res.bytes_stream())
        ).into_response(),
        Err(err) => return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(err.to_string())
        ).into_response(),
    }
}

#[derive(Clone)]
pub struct AxumAppStateConnect {
    pub stream_url: String,
    pub client: reqwest::Client,
    pub user_agent: String,
}

#[derive(Clone)]
pub struct AxumAppState {
    pub port: u16,
    pub connect: Arc::<RwLock<HashMap<String, AxumAppStateConnect>>>,
}
