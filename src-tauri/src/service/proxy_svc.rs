use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::{Path, Query, State}, response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
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
        request: Arc::new(RwLock::new(HashMap::new())),
        trakt_auth_state: Arc::new(RwLock::new(vec![])),
    });

    let router = Router::new()
        .route("/stream/{types}/{id}", get(stream))
        .route("/trakt_auth", get(trakt_auth))
        .with_state(axum_app_state);

    axum::serve(listener, router).await?;

    anyhow::Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraktAuthParam {
    pub code: String,
    pub state: String,
}

async fn trakt_auth(headers: axum::http::HeaderMap, State(app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Query(params): Query<TraktAuthParam>) {
    tracing::debug!("trakt_auth: {:?} {:?}", params, headers);
    let app_state = app_state.read().await.clone().unwrap();
    if !app_state.trakt_auth_state.read().await.contains(&params.state) {
        tracing::error!("trakt_auth: {} 无效的 state", &params.state);
        return;
    }
    let res = app_state.app.emit("trakt_auth", params.code);
    if let Err(err) = res {
        tracing::error!("trakt_auth: 向前台发送事件失败 {}", err);
    }
}

async fn stream(headers: axum::http::HeaderMap, State(app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Path((types, id)): Path<(String, String)>) -> axum::response::Response {
    tracing::debug!("stream: {} {} {:?}", types, id, headers);
    let app_state = app_state.read().await.clone().unwrap();
    let request = app_state.request.read().await;
    let request = match request.get(&id).clone() {
        Some(request) => request,
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
    if request.read_from_cache {
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

    let client = request.client.clone();
    let mut req_headers = headers.clone();
    req_headers.remove(axum::http::header::HOST);
    req_headers.remove(axum::http::header::USER_AGENT);
    req_headers.insert(axum::http::header::USER_AGENT, request.user_agent.clone().parse().unwrap());
    let res = client
        .get(request.stream_url.clone())
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
    if request.write_to_cache {
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
pub struct AxumAppStateRequest {
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
    pub request: Arc::<RwLock<HashMap<String, AxumAppStateRequest>>>,
    pub trakt_auth_state: Arc::<RwLock<Vec<String>>>,
}
