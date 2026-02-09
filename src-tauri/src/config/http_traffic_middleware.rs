use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

use std::sync::Arc;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct TrafficScope {
    pub emby_server_id: Option<String>,
    pub proxy_id: Option<String>,
}

#[derive(Debug)]
pub struct TrafficStats {
    upload: AtomicU64,
    download: AtomicU64,
}

impl TrafficStats {
    fn new() -> Self {
        Self {
            upload: AtomicU64::new(0),
            download: AtomicU64::new(0),
        }
    }
}

/// Estimate size of headers (keys + values)
fn estimate_headers_size(headers: &axum::http::HeaderMap) -> u64 {
    headers.iter().fold(0, |acc, (k, v)| acc + k.as_str().len() as u64 + v.len() as u64)
}

/// Middleware for counting traffic
pub struct TrafficCounterMiddleware;

#[async_trait::async_trait]
impl Middleware for TrafficCounterMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut axum::http::Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        // Check if there is a traffic scope in extensions
        let scope_stats = if let Some((scope, traffic_stat)) = extensions.get::<(TrafficScope, Arc::<RwLock<HashMap<TrafficScope, Arc<TrafficStats>>>>)>() {
            // Get or create stats for this scope
            // Optimistic read first
            if let Some(stat) = traffic_stat.read().await.get(scope) {
                Some(stat.clone())
            } else {
                let mut map = traffic_stat.write().await;
                // Check again
                if let Some(stat) = map.get(scope) {
                    Some(stat.clone())
                } else {
                    // Insert new stats
                    let v = Arc::new(TrafficStats::new());
                    map.insert(scope.clone(), v.clone());
                    Some(v)
                }
            }
        } else {
            None
        };

        // Count upload traffic (best effort based on content-length or known body size)
        let mut upload_size = 0;
        
        // Add URL length
        upload_size += req.url().as_str().len() as u64;
        
        // Add headers size
        upload_size += estimate_headers_size(req.headers());

        // Add body size
        if let Some(len) = req.body().and_then(|b| b.as_bytes().map(|b| b.len() as u64)) {
             upload_size += len;
        }
        
        if let Some(s) = &scope_stats {
            s.upload.fetch_add(upload_size, Ordering::Relaxed);
        }

        // Execute request
        let res = next.run(req, extensions).await?;

        // Count response headers
        let res_headers_size = estimate_headers_size(res.headers());
        if let Some(s) = &scope_stats {
            s.download.fetch_add(res_headers_size, Ordering::Relaxed);
        }

        // Wrap response stream to count download traffic
        let status = res.status();
        let headers = res.headers().clone();
        let version = res.version();
        
        let stream = res.bytes_stream().map(move |res| {
            if let Ok(bytes) = &res {
                let len = bytes.len() as u64;
                if let Some(s) = &scope_stats {
                     s.download.fetch_add(len, Ordering::Relaxed);
                }
            }
            res
        });

        // Reconstruct response with wrapped stream using http::Response builder
        let mut builder = axum::http::Response::builder()
            .status(status)
            .version(version);
            
        for (k, v) in headers.iter() {
            builder = builder.header(k, v);
        }
        
        // Wrap the stream into a reqwest::Body
        let body = reqwest::Body::wrap_stream(stream);
        
        // Build http::Response
        let http_response = builder.body(body)
            .map_err(|e: axum::http::Error| reqwest_middleware::Error::Middleware(anyhow::anyhow!(e)))?;
        
        // Convert http::Response to reqwest::Response
        let final_res = Response::from(http_response);

        Ok(final_res)
    }
}
