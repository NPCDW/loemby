use super::app_state::AppState;

pub async fn get_api_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest::Client> {
    let config = state.app_config.clone();
    let proxy_key = proxy_url.clone().unwrap_or("no".to_string());
    // 必须 clone 否则在 read 未完成时 write 会锁住
    let reqwest_pool = state.api_reqwest_pool.read().await.clone();
    let client = reqwest_pool.get(&proxy_key);
    let client = if client.is_some() {
        client.unwrap().to_owned()
    } else {
        let mut client = reqwest::Client::builder()
            // 結束バンド 不支持 rustls
            // .use_rustls_tls()
            .danger_accept_invalid_certs(config.danger_accept_invalid_certs)
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
        state.api_reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}

pub async fn get_image_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest::Client> {
    let config = state.app_config.clone();
    let proxy_key = proxy_url.clone().unwrap_or("no".to_string());
    // 必须 clone 否则在 read 未完成时 write 会锁住
    let reqwest_pool = state.image_reqwest_pool.read().await.clone();
    let client = reqwest_pool.get(&proxy_key);
    let client = if client.is_some() {
        client.unwrap().to_owned()
    } else {
        let mut client = reqwest::Client::builder()
            // 結束バンド 不支持 rustls
            // .use_rustls_tls()
            .danger_accept_invalid_certs(config.danger_accept_invalid_certs)
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
        state.image_reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}

pub async fn get_stream_http_client(proxy_url: Option<String>, state: &tauri::State<'_, AppState>) -> anyhow::Result<reqwest::Client> {
    let config = state.app_config.clone();
    let mut client = reqwest::Client::builder()
        // 結束バンド 不支持 rustls
        // .use_rustls_tls()
        .danger_accept_invalid_certs(config.danger_accept_invalid_certs);
    if let Some(proxy_url) = proxy_url {
        let proxy = reqwest::Proxy::all(&proxy_url);
        if proxy.is_err() {
            return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
        }
        client = client.proxy(proxy.unwrap());
    }
    let client = client.build()?;
    anyhow::Ok(client)
}