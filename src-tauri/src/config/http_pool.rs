use super::app_state::AppState;


pub async fn get_http_client(proxy_url: Option<String>, state: tauri::State<'_, AppState>) -> anyhow::Result<reqwest::Client> {
    let config = state.app_config.clone();
    let proxy_key = proxy_url.clone().unwrap_or("no".to_string());
    let reqwest_pool = state.reqwest_pool.read().await.clone();
    let client = reqwest_pool.get(&proxy_key);
    let client = if client.is_some() {
        client.unwrap().to_owned()
    } else {
        let mut client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.danger_accept_invalid_certs)
            .pool_max_idle_per_host(3)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            // 该超时时间为从请求开始到响应结束总耗时，如果超过了总耗时，不管是否正在传输数据，都会结束链接，与其他语言库的逻辑不同，其他语言是如果链接有数据传输，有keep-alive就不会超时，
            // 设置该选项会导致访问流媒体时链接频繁被超时释放，又频繁建立链接，会增加服务器压力，实际上mpv播放时，从头到尾只建立一个链接，由mpv控制缓存速度保持连接分片返回流
            // .timeout(std::time::Duration::from_secs(30))
            // 设置该选项会在播放暂停一段时间时释放链接，如果频繁暂停，会出现上述情况
            // .read_timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(30));
        if let Some(proxy_url) = proxy_url {
            let proxy = reqwest::Proxy::all(&proxy_url);
            if proxy.is_err() {
                return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
            }
            client = client.proxy(proxy.unwrap());
        }
        let client = client.build()?;
        state.reqwest_pool.write().await.insert(proxy_key, client.clone());
        client
    };
    anyhow::Ok(client)
}