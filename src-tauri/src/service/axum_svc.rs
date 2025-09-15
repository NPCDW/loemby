use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::{Path, Query, State}, response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tokio::{fs::File, io::AsyncWriteExt, sync::RwLock};
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_stream::StreamExt;
use crate::{config::{app_state::{AppState, TauriNotify}, http_pool}, mapper::{emby_server_mapper, global_config_mapper, proxy_server_mapper}, service::{emby_http_svc, trakt_http_svc::{self, TraktHttpTokenParam}}};

pub async fn init_axum_svc(axum_app_state: Arc<RwLock<Option<AxumAppState>>>, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
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
        .route("/image", get(image))
        .route("/trakt_auth", get(trakt_auth))
        .with_state(axum_app_state);

    axum::serve(listener, router).await?;

    anyhow::Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayParam {
    pub mpv_ipc: String,
    pub emby_server_id: String,
    pub item_id: String,
    pub direct_link: String,
    pub select_policy: String, // 手动，高码率优先，高分辨率优先
    pub video_select: i32,
    pub audio_select: i32,
    pub subtitle_select: i32,
    pub version_select: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraktAuthParam {
    pub code: String,
    pub state: String,
}

async fn trakt_auth(headers: axum::http::HeaderMap, State(axum_app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Query(params): Query<TraktAuthParam>) -> axum::response::Response {
    tracing::debug!("trakt_auth: {:?} {:?}", params, headers);
    let axum_app_state = axum_app_state.read().await.clone().unwrap();
    if !axum_app_state.trakt_auth_state.read().await.contains(&params.state) {
        tracing::error!("trakt_auth: {} 无效的 state", &params.state);
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(format!("trakt_auth: {} 无效的 state", &params.state))
        ).into_response();
    }
    let app_handle = axum_app_state.app;
    let redirect_uri = format!("http://127.0.0.1:{}/trakt_auth", axum_app_state.port);
    let res = trakt_http_svc::token(TraktHttpTokenParam {
        redirect_uri: redirect_uri.clone(),
        code: Some(params.code),
        refresh_token: None,
    }, &app_handle.state(), &app_handle).await;
    if let Err(err) = res {
        tracing::error!("trakt_auth: 根据code获取token失败 {}", err);
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(format!("trakt_auth: 根据code获取token失败 {}", err))
        ).into_response();
    }
    let res = trakt_http_svc::save_access_token(res.unwrap(), redirect_uri, &app_handle.state()).await;
    if let Err(err) = res {
        tracing::error!("trakt_auth: 保存token失败 {}", err);
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(format!("trakt_auth: 保存token失败 {}", err))
        ).into_response();
    }

    let res = app_handle.emit("trakt_auth", ());
    if let Err(err) = res {
        tracing::error!("trakt_auth: 向前台发送事件失败 {}", err);
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(format!("trakt_auth: 向前台发送事件失败 {}", err))
        ).into_response();
    }
    let window = app_handle.webview_windows();
    let window = window.values().next().expect("Sorry, no window found");
    window.unminimize().expect("Sorry, no window unminimize");
    window.show().expect("Sorry, no window show");
    window.set_focus().expect("Can't Bring Window to Focus");
    axum::response::Html("<html><body style='background-color: #1D1E1F; color: #FFFFFF'>授权成功，您可以关闭网页，并返回应用了</body></html>").into_response()
}

async fn stream(headers: axum::http::HeaderMap, State(axum_app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Path((types, id)): Path<(String, String)>) -> axum::response::Response {
    tracing::debug!("stream: {} {} {:?}", types, id, headers);
    let axum_app_state = axum_app_state.read().await.clone().unwrap();
    let request = axum_app_state.request.read().await;
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

    let app_state = axum_app_state.app.state::<AppState>().clone();
    let client = http_pool::get_stream_http_client(request.proxy_url.clone(), &app_state).await.unwrap();
    let mut req_headers = headers.clone();
    req_headers.remove(axum::http::header::HOST);
    req_headers.remove(axum::http::header::REFERER);
    req_headers.remove(axum::http::header::USER_AGENT);
    req_headers.insert(axum::http::header::USER_AGENT, request.user_agent.clone().parse().unwrap());
    let mut res = client
        .get(request.stream_url.clone())
        .headers(req_headers.clone())
        .send()
        .await;
    tracing::debug!("stream: {} {} {:?} {:?} 媒体流响应 {:?}", types, &id, request, req_headers, res);
    // 手动重定向，reqwest 默认会自动重定向，但某些情况下会失败，所以这里手动重定向，猜测可能是 https 重定向到 http 会失败
    for _ in 1..10 {
        if let Ok(response) = &res {
            if !response.status().is_redirection() {
                break;
            }
            tracing::debug!("stream: {} {} 手动重定向", types, &id);
            match response.headers().get(axum::http::header::LOCATION) {
                None => {
                    tracing::error!("stream: {} {} {:?} 手动重定向失败，媒体流响应没有 Location 头", types, &id, request);
                    axum_app_state.app.emit("tauri_notify", TauriNotify {
                        alert_type: "ElMessage".to_string(),
                        message_type: "error".to_string(),
                        title: None,
                        message: format!("手动重定向失败，媒体流响应没有 Location 头"),
                    }).unwrap();
                    return (
                        axum::http::StatusCode::SERVICE_UNAVAILABLE,
                        axum::http::HeaderMap::new(),
                        axum::body::Body::new("手动重定向失败，媒体流响应没有 Location 头".to_string())
                    ).into_response();
                },
                Some(location) => {
                    tracing::debug!("stream: {} {} {:?} 手动重定向到 {:?}", types, &id, request, location);
                    // /cdn?path=/mnt/files/nfs-files/resources\xe7\x94\xb5\xe8\xa7\x86\xe5\x89\xa7/\xe6\xac\xa7\xe7\xbe\x8e\xe5\x89\xa7/\xe6\x98\xaf\xef\xbc\x8c\xe9\xa6\x96\xe7\x9b\xb8 (1986)/Season 1/\xe6\x98\xaf\xef\xbc\x8c\xe9\xa6\x96\xe7\x9b\xb8 - S01E04 - \xe7\xac\xac 4 \xe9\x9b\x86.mp4
                    // 如果使用 location.to_str() 这个方法会报错，所以可能是因为这个原因所以没有自动重定向
                    let location = String::from_utf8_lossy(location.as_bytes()).to_string();
                    res = client
                        .get(location)
                        .headers(req_headers.clone())
                        .send()
                        .await;
                    tracing::debug!("stream: {} {} {:?} {:?} 手动重定向后媒体流响应 {:?}", types, &id, request, req_headers, res);
                },
            }
        }
    }
    match res {
        Err(err) => {
            tracing::error!("stream: {} {} {:?} 媒体流响应 {:?}", types, &id, request.user_agent, err);
            axum_app_state.app.emit("tauri_notify", TauriNotify {
                alert_type: "ElMessage".to_string(),
                message_type: "error".to_string(),
                title: None,
                message: format!("媒体流响应错误: {}", err),
            }).unwrap();
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::http::HeaderMap::new(),
                axum::body::Body::new(err.to_string())
            ).into_response()
        },
        Ok(response) => {
            if !response.status().is_success() {
                let status = response.status();
                let headers = response.headers().clone();
                tracing::error!("stream: {} {} {:?} 媒体流响应 {:?} {:?}", types, &id, req_headers, status, headers);
                let mut stream = response.bytes_stream();
                let mut bytes = Vec::new();
                while let Some(Ok(chunk)) = stream.next().await {
                    bytes.extend_from_slice(&chunk);
                }
                let text = String::from_utf8_lossy(&bytes);
                tracing::error!("stream: {} {} {:?} 错误响应内容: {}", types, &id, req_headers, text);
                axum_app_state.app.emit("tauri_notify", TauriNotify {
                    alert_type: "ElMessage".to_string(),
                    message_type: "error".to_string(),
                    title: None,
                    message: format!("媒体流响应错误: {} {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"), text),
                }).unwrap();
                return (
                    status,
                    headers,
                ).into_response();
            }
            (
                response.status(),
                response.headers().clone(),
                axum::body::Body::from_stream(response.bytes_stream())
            ).into_response()
        },
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbyImageParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub image_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IconImageParam {
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ImageParamType {
    Emby(EmbyImageParam),
    Icon(IconImageParam),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageParam {
    pub param: ImageParamType,
}

async fn image(headers: axum::http::HeaderMap, State(axum_app_state): State<Arc<RwLock<Option<AxumAppState>>>>, Query(param): Query<ImageParam>) -> axum::response::Response {
    tracing::debug!("image: {:?}", param);
    let axum_app_state = axum_app_state.read().await.clone().unwrap();
    let state = axum_app_state.app.state::<AppState>().clone();
    let disabled_image_cache = global_config_mapper::get_cache("disabled_image_cache", &state).await.unwrap_or("off".to_string()) == "on";
    let user_agent;
    let image_url;
    let cache_prefix;
    let proxy_url;
    match &param.param {
        ImageParamType::Emby(param) => {
            let emby_server = match emby_server_mapper::get_cache(param.emby_server_id.clone(), &state).await {
                Some(emby_server) => emby_server,
                None => return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    axum::http::HeaderMap::new(),
                    axum::body::Body::new(format!("emby_server not found"))
                ).into_response(),
            };
            proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, &state).await;
            user_agent = emby_server.user_agent.unwrap();
            image_url = emby_http_svc::get_image_url(&emby_server.base_url.unwrap(), &param.item_id, &param.image_type);
            cache_prefix = format!("image/{}/{}", param.emby_server_id, param.image_type);
        },
        ImageParamType::Icon(param) => {
            let app_proxy_id = global_config_mapper::get_cache("app_proxy_id", &state).await;
            proxy_url = proxy_server_mapper::get_app_proxy_url(app_proxy_id, &state).await;
            user_agent = format!("loemby/{}", env!("CARGO_PKG_VERSION"));
            image_url = param.image_url.clone();
            cache_prefix = "icon".to_string();
        },
    }

    let cache_digest = sha256::digest(&image_url);
    let cache_file_path = axum_app_state.app.path().resolve(&format!("cache/{}/{}.png", cache_prefix, cache_digest), tauri::path::BaseDirectory::AppLocalData).unwrap();
    let metadata_file_path = axum_app_state.app.path().resolve(&format!("cache/{}/{}.metadata", cache_prefix, cache_digest), tauri::path::BaseDirectory::AppLocalData).unwrap();
    if cache_file_path.exists() && !disabled_image_cache {
        tracing::debug!("image: {:?} {:?} 从缓存读取", cache_file_path, param);
        let file = match tokio::fs::File::open(&cache_file_path).await {
            Ok(file) => file,
            Err(err) => return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::http::HeaderMap::new(),
                axum::body::Body::new(format!("从缓存读取失败 {}", err))
            ).into_response(),
        };

        // 从 .metadata 文件中读取元数据
        let mut headers = axum::http::HeaderMap::new();
        if let Ok(metadata_content) = tokio::fs::read_to_string(&metadata_file_path).await {
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_content) {
                if let Some(content_type) = metadata["content_type"].as_str() {
                    headers.insert("content-type", content_type.parse().unwrap());
                }
                if let Some(content_length) = metadata["content_length"].as_str() {
                    headers.insert("content-length", content_length.parse().unwrap());
                }
                if let Some(content_encoding) = metadata["content_encoding"].as_str() {
                    headers.insert("content-encoding", content_encoding.parse().unwrap());
                }
            }
        }

        return (
            axum::http::StatusCode::OK,
            headers,
            axum::body::Body::from_stream(FramedRead::new(file, BytesCodec::new()))
        ).into_response();
    }

    let app_state = axum_app_state.app.state::<AppState>().clone();
    let client = match http_pool::get_image_http_client(proxy_url.clone(), &app_state).await {
        Ok(client) => client,
        Err(err) => return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(format!("http连接池获取失败 {}", err))
        ).into_response(),
    };
    let mut req_headers = headers.clone();
    req_headers.remove(axum::http::header::HOST);
    req_headers.remove(axum::http::header::REFERER);
    req_headers.remove(axum::http::header::USER_AGENT);
    req_headers.insert(axum::http::header::USER_AGENT, user_agent.clone().parse().unwrap());
    req_headers.insert(axum::http::header::REFERER, image_url.clone().parse().unwrap());
    let res = client
        .get(image_url.clone())
        .headers(req_headers.clone())
        .send()
        .await;
    tracing::debug!("image: {:?} 媒体流响应 {:?}", param, res);
    if let Err(err) = res {
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(err.to_string())
        ).into_response()
    }
    let response = res.unwrap();
    let status = response.status();
    let header = response.headers().clone();
    let mut stream = response.bytes_stream();
    if disabled_image_cache {
        return (
            status,
            header,
            axum::body::Body::from_stream(stream)
        ).into_response();
    }
    if !cache_file_path.parent().unwrap().exists() {
        std::fs::create_dir_all(cache_file_path.parent().unwrap()).unwrap();
    }
    let mut cache_file = File::create(&cache_file_path).await.unwrap();
    while let Some(Ok(chunk)) = stream.next().await {
        cache_file.write_all(&chunk).await.unwrap();
    }
    cache_file.flush().await.unwrap();
    drop(cache_file);

    // 保存元数据到 .metadata 文件
    let mut metadata_file = File::create(&metadata_file_path).await.unwrap();
    let metadata = serde_json::json!({
        "content_type": header.get("content-type").map(|v| v.to_str().unwrap().to_string()),
        "content_length": header.get("content-length").map(|v| v.to_str().unwrap().to_string()),
        "content_encoding": header.get("content-encoding").map(|v| v.to_str().unwrap().to_string()),
    });
    metadata_file.write_all(metadata.to_string().as_bytes()).await.unwrap();
    metadata_file.flush().await.unwrap();
    drop(metadata_file);

    let file = match tokio::fs::File::open(&cache_file_path).await {
        Ok(file) => file,
        Err(err) => return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::http::HeaderMap::new(),
            axum::body::Body::new(format!("从缓存读取失败 {}", err))
        ).into_response(),
    };
    (
        status,
        header,
        axum::body::Body::from_stream(FramedRead::new(file, BytesCodec::new()))
    ).into_response()
}

#[derive(Clone, Debug)]
pub struct AxumAppStateRequest {
    pub stream_url: String,
    pub proxy_url: Option<String>,
    pub user_agent: String,
}

#[derive(Clone)]
pub struct AxumAppState {
    pub app: tauri::AppHandle,
    pub port: u16,
    pub request: Arc::<RwLock<HashMap<String, AxumAppStateRequest>>>,
    pub trakt_auth_state: Arc::<RwLock<Vec<String>>>,
}
