use std::str::FromStr;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Method, Response};

use crate::{config::{app_state::AppState, http_pool}, controller::invoke_ctl::{HttpForwardParam, LoadImageParam}};

use super::proxy_svc::AxumAppStateRequest;

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
