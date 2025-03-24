use std::str::FromStr;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Method, Response};

use crate::{config::{app_state::AppState, http_pool}, controller::invoke_ctl::{HttpForwardParam, LoadImageParam}};

use super::proxy_svc::AxumAppStateConnect;

pub async fn forward(param: HttpForwardParam, state: tauri::State<'_, AppState>) -> anyhow::Result<Response> {
    let mut headers = HeaderMap::new();
    for (key, value) in &param.headers {
        headers.insert(HeaderName::from_str(key).unwrap(), HeaderValue::from_str(value).unwrap());
    }

    let client = http_pool::get_http_client(param.proxy, state).await?;
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
    let app_state = auxm_app_state.read().await.clone();
    let app_state = app_state.as_ref().unwrap();

    let client = http_pool::get_http_client(body.proxy_url.clone(), state).await?;

    let digest = sha256::digest(&body.image_url);
    app_state.connect.write().await.insert(digest.clone(), AxumAppStateConnect {
        stream_url: body.image_url.clone(),
        client: client,
        user_agent: body.user_agent.clone(),
        read_from_cache: true,
        write_to_cache: true,
    });
    let image_path = format!("http://127.0.0.1:{}/stream/image/{}", &app_state.port, &digest);

    anyhow::Ok(image_path)
}
