use std::str::FromStr;

use tokio::io::AsyncReadExt;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Method, Response};

use crate::controller::invoke_ctl::{HttpForwardParam, LoadImageParam};

pub async fn forward(param: HttpForwardParam) -> anyhow::Result<Response> {
    let mut headers = HeaderMap::new();
    for (key, value) in &param.headers {
        headers.insert(HeaderName::from_str(key).unwrap(), HeaderValue::from_str(value).unwrap());
    }

    let mut client = reqwest::Client::builder();
    if let Some(proxy_url) = param.proxy {
        let proxy = reqwest::Proxy::all(&proxy_url);
        if proxy.is_err() {
            return Err(anyhow::anyhow!("{} 代理不正确 {:?}", proxy_url, proxy));
        }
        client = client.proxy(proxy.unwrap());
    }
    let client = client.build()?;
    let mut builder = client
        .request(Method::from_str(&param.method)?, param.url)
        .headers(headers);
    if let Some(body) = param.body {
        builder = builder.body(body);
    }
    tracing::debug!("reqwest request {:?}", &builder);
    let response = builder.send().await?;
    tracing::debug!("reqwest response {:?}", &response);
    Ok(response)
}

pub async fn load_image(body: LoadImageParam, reader: tauri::ipc::Channel<&[u8]>) {
    tracing::debug!("load_image: {}", body.image_url);
    
    let mut client = reqwest::Client::builder();
    if let Some(proxy_url) = body.proxy_url {
        let proxy = reqwest::Proxy::all(&proxy_url);
        if proxy.is_err() {
            tracing::error!("{} 代理不正确 {:?}", proxy_url, proxy);
            return;
        }
        client = client.proxy(proxy.unwrap());
    }
    let client = client.build();
    if client.is_err() {
        tracing::error!("{} 创建图片流请求失败 {:?}", body.image_url, client);
        return;
    }
    let client = client.unwrap();
    let mut req_headers = HeaderMap::new();
    req_headers.insert(HeaderName::from_str("User-Agent").unwrap(), HeaderValue::from_str(&body.user_agent).unwrap());
    let res = client
        .get(body.image_url)
        .headers(req_headers)
        .send()
        .await;
    if res.is_err() {
        tracing::error!("{} 请求图片流失败 {:?}", body.image_url, res);
        return;
    }
    let res = res.unwrap();

    use futures_util::StreamExt;
    while let Some(chunk) = res.bytes_stream().next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let slice: &[u8] = &chunk;
        channel.send(slice).await.map_err(|e| e.to_string())?;
    }
}