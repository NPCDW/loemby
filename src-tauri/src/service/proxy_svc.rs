use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::{Path, State}, response::IntoResponse, routing::get, Router};
use tauri::Manager;
use tokio::{fs::File, io::AsyncWriteExt, sync::RwLock};
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

pub async fn init_proxy_svc(axum_app_state: Arc<RwLock<Option<AxumAppState>>>, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    tracing::info!("axum listening on {:?}", actual_port);
    
    *axum_app_state.write().await = Some(AxumAppState {
        app: app_handle,
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

    let cache_file_path = app_state.app.path().resolve(&format!("cache/{}/{}", types, id), tauri::path::BaseDirectory::AppLocalData).unwrap();
    if connect.read_from_cache {
        tracing::debug!("stream: {} {} 从缓存读取", types, id);
        if cache_file_path.exists() {
            let file = match tokio::fs::File::open(cache_file_path).await {
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
    if let Err(err) = res {
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(err.to_string())
        ).into_response()
    }
    let response = res.unwrap();
    if connect.write_to_cache {
        if !cache_file_path.parent().unwrap().exists() {
            std::fs::create_dir_all(cache_file_path.parent().unwrap()).unwrap();
        }
        let mut file = File::create(cache_file_path).await.unwrap();
        let status = response.status();
        let header = response.headers().clone();
        let mut stream = response.bytes_stream();

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
        response.status(),
        response.headers().clone(),
        axum::body::Body::from_stream(response.bytes_stream())
    ).into_response()
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
    pub app: tauri::AppHandle,
    pub port: u16,
    pub connect: Arc::<RwLock<HashMap<String, AxumAppStateConnect>>>,
}
