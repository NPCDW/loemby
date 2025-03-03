use std::str::FromStr;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Method, Response};

use crate::{config::app_state::AppState, controller::invoke_ctl::{HttpForwardParam, LoadImageParam}};

use super::proxy_svc::AxumAppStateConnect;

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

pub async fn load_image(body: LoadImageParam, state: tauri::State<'_, AppState>) -> anyhow::Result<String> {
    let auxm_app_state = state.auxm_app_state.clone();
    let mut app_state = auxm_app_state.read().await.clone();
    let app_state = app_state.as_mut().unwrap();

    let uuid = uuid::Uuid::new_v4().to_string();
    app_state.connect.write().await.insert(uuid.clone(), AxumAppStateConnect {stream_url: body.image_url.clone(), proxy_url: body.proxy_url.clone(), user_agent: body.user_agent.clone()});
    let image_path = format!("http://127.0.0.1:{}/stream/{}", &app_state.port, &uuid);

    anyhow::Ok(image_path)
}
