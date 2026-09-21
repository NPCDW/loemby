use crate::mapper::global_config_mapper;

use super::app_state::AppState;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

pub async fn get_api_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest_middleware::ClientWithMiddleware> {

    let proxy_key = proxy_url.clone().unwrap_or("no".to_string());
    let danger_accept_invalid_certs = global_config_mapper::get_cache("danger_accept_invalid_certs", state).await;
    // 必须 clone 否则在 read 未完成时 write 会锁住
    let reqwest_pool = state.api_reqwest_pool.read().await.clone();
    let client = reqwest_pool.get(&proxy_key);
    let client = if client.is_some() {
        client.unwrap().to_owned()
    } else {
        let mut client = reqwest::Client::builder()
            .danger_accept_invalid_certs(Some("true".to_string()) == danger_accept_invalid_certs)
            .pool_max_idle_per_host(6)
            .pool_idle_timeout(tokio::time::Duration::from_secs(90))
            .read_timeout(tokio::time::Duration::from_secs(30))
            .connect_timeout(tokio::time::Duration::from_secs(30));
        if let Some(proxy_url) = proxy_url {
            let proxy = reqwest::Proxy::all(&proxy_url);
            if proxy.is_err() {
                return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
            }
            client = client.proxy(proxy.unwrap());
        }
        let client = client.build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        state.api_reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}

pub async fn get_image_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest_middleware::ClientWithMiddleware> {
    let proxy_key = proxy_url.clone().unwrap_or("no".to_string());
    let danger_accept_invalid_certs = global_config_mapper::get_cache("danger_accept_invalid_certs", state).await;
    // 必须 clone 否则在 read 未完成时 write 会锁住
    let reqwest_pool = state.image_reqwest_pool.read().await.clone();
    let client = reqwest_pool.get(&proxy_key);
    let client = if client.is_some() {
        client.unwrap().to_owned()
    } else {
        let mut client = reqwest::Client::builder()
            .danger_accept_invalid_certs(Some("true".to_string()) == danger_accept_invalid_certs)
            .pool_max_idle_per_host(6)
            .pool_idle_timeout(tokio::time::Duration::from_secs(90))
            .read_timeout(tokio::time::Duration::from_secs(30))
            .connect_timeout(tokio::time::Duration::from_secs(30));
        if let Some(proxy_url) = proxy_url {
            let proxy = reqwest::Proxy::all(&proxy_url);
            if proxy.is_err() {
                return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
            }
            client = client.proxy(proxy.unwrap());
        }
        let client = client.build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        state.image_reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}

pub async fn get_stream_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest_middleware::ClientWithMiddleware> {
    let proxy_key = proxy_url.clone().unwrap_or("no".to_string());
    let danger_accept_invalid_certs = global_config_mapper::get_cache("danger_accept_invalid_certs", state).await;
    // 必须 clone 否则在 read 未完成时 write 会锁住
    let reqwest_pool = state.stream_reqwest_pool.read().await.clone();
    let client = reqwest_pool.get(&proxy_key);
    let client = if client.is_some() {
        client.unwrap().to_owned()
    } else {
        let mut client = reqwest::Client::builder()
            .danger_accept_invalid_certs(Some("true".to_string()) == danger_accept_invalid_certs)
            // 禁用自动重定向，因为重定向到的网址可能有不规范的编码，或者重定向后 reqwest 默认只会携带 ua 和 auth ,不会携带 x-emby-token 导致无法访问
            .redirect(reqwest::redirect::Policy::none())
            // 流媒体必须复用连接：m3u8 里的每个 ts 都是一次独立的请求，如果每次请求都新建一个 Client
            // （连接池也随 Client 一起被销毁），那么每个 ts 都会重新握手建连，表现为每个切片一条连接
            .pool_max_idle_per_host(6)
            .pool_idle_timeout(tokio::time::Duration::from_secs(90))
            // 注意：流媒体不能设置 read_timeout，播放器可能长时间挂着连接不读取数据
            .connect_timeout(tokio::time::Duration::from_secs(30));
        if let Some(proxy_url) = proxy_url {
            let proxy = reqwest::Proxy::all(&proxy_url);
            if proxy.is_err() {
                return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
            }
            client = client.proxy(proxy.unwrap());
        }
        let client = client.build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        state.stream_reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}