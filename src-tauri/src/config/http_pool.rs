use crate::mapper::global_config_mapper;

use super::app_state::AppState;

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
        let client = reqwest_middleware::ClientBuilder::new(client)
            .with(crate::config::http_traffic_middleware::TrafficCounterMiddleware)
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
        let client = reqwest_middleware::ClientBuilder::new(client)
            .with(crate::config::http_traffic_middleware::TrafficCounterMiddleware)
            .build();
        state.image_reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}

pub async fn get_stream_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest_middleware::ClientWithMiddleware> {
    let danger_accept_invalid_certs = global_config_mapper::get_cache("danger_accept_invalid_certs", state).await;
    let mut client = reqwest::Client::builder()
        .danger_accept_invalid_certs(Some("true".to_string()) == danger_accept_invalid_certs);
    if let Some(proxy_url) = proxy_url {
        let proxy = reqwest::Proxy::all(&proxy_url);
        if proxy.is_err() {
            return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
        }
        client = client.proxy(proxy.unwrap());
    }
    let client = client.build()?;
    let client = reqwest_middleware::ClientBuilder::new(client)
        .with(crate::config::http_traffic_middleware::TrafficCounterMiddleware)
        .build();
    anyhow::Ok(client)
}