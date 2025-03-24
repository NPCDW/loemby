use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::{Path, State}, response::IntoResponse, routing::get, Router};
use tauri::Manager;
use tokio::{fs::File, io::AsyncWriteExt, sync::RwLock};
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

pub async fn init_proxy_svc() -> anyhow::Result<AxumAppState> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    println!("axum listening on {:?}", actual_port);
    
    let axum_app_state = AxumAppState {
        app: Arc::new(RwLock::new(None)),
        port: actual_port,
        connect: Arc::new(RwLock::new(HashMap::new())),
    };

    let router = Router::new()
        .route("/stream/{types}/{id}", get(stream))
        .with_state(axum_app_state.clone());

    axum::serve(listener, router).await?;

    anyhow::Ok(axum_app_state)
}

async fn stream(headers: axum::http::HeaderMap, State(app_state): State<AxumAppState>, Path((types, id)): Path<(String, String)>) -> axum::response::Response {
    tracing::debug!("stream: {} {} {:?}", types, id, headers);
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

    if connect.read_from_cache {
        tracing::debug!("stream: {} {} 从缓存读取", types, id);
        let app = app_state.app.read().await.clone();
        let filepath = app.unwrap().path().resolve(&format!("cache/{}/{}", types, id), tauri::path::BaseDirectory::AppLocalData).unwrap();
        if filepath.exists() {
            let file = match tokio::fs::File::open(filepath).await {
                Ok(file) => file,
                Err(err) => return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    axum::http::HeaderMap::new(),
                    axum::body::Body::new(format!("从缓存读取失败 {}", err))
                ).into_response(),
            };
            return (
                axum::http::StatusCode::OK,
                axum::http::HeaderMap::new(),
                axum::body::Body::from_stream(FramedRead::new(file, BytesCodec::new()))
            ).into_response();
        }
    }

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
        Ok(res) => {
            if connect.write_to_cache {
                let mut file = File::create("cache.txt").await.unwrap();
                let status = res.status();
                let header = res.headers().clone();
                let mut stream = res.bytes_stream();

                let (sender, receiver) = tokio::sync::mpsc::channel(32);
                tokio::spawn(async move {
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.unwrap();
                        file.write_all(&chunk).await.unwrap();
                        sender.send(chunk).await.unwrap();
                    }
                });
                let receiver_stream = ReceiverStream::new(receiver);
                let mapped_stream = receiver_stream.map(|chunk| Ok::<_, std::io::Error>(chunk));
            
                return (
                    status,
                    header,
                    axum::body::Body::from_stream(mapped_stream)
                ).into_response()
            }
            return (
                res.status(),
                res.headers().clone(),
                axum::body::Body::from_stream(res.bytes_stream())
            ).into_response()
        },
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
    pub read_from_cache: bool,
    pub write_to_cache: bool,
}

#[derive(Clone)]
pub struct AxumAppState {
    pub app: Arc::<RwLock<Option<tauri::AppHandle>>>,
    pub port: u16,
    pub connect: Arc::<RwLock<HashMap<String, AxumAppStateConnect>>>,
}
