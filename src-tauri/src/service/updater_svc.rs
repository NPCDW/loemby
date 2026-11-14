use serde::Serialize;
use tauri::{Manager, async_runtime::Mutex};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{
    config::app_state::AppState,
    mapper::{global_config_mapper, proxy_server_mapper, reverse_proxy_server_mapper},
};

#[derive(Default)]
pub struct UpdaterState {
    pending: Mutex<PendingUpdate>,
}

#[derive(Default)]
struct PendingUpdate {
    update: Option<Update>,
    installed: bool,
}

#[derive(Serialize)]
pub struct UpdateInfo {
    version: String,
}

pub async fn check(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater_state = app.state::<UpdaterState>();
    let mut pending = updater_state.pending.try_lock().map_err(|_| "正在检查或安装更新，请稍后再试".to_string())?;
    if let Some(update) = &pending.update {
        return Ok(Some(UpdateInfo { version: update.version.clone() }));
    }

    let update = check_update(&app).await.map_err(|err| format!("检查更新失败: {}", err))?;
    let info = update.as_ref().map(|update| UpdateInfo { version: update.version.clone() });
    pending.update = update;
    Ok(info)
}

struct UpdateRoute {
    proxy: Option<url::Url>,
    reverse_proxy_url: Option<String>,
}

impl UpdateRoute {
    fn url(&self, original_url: &url::Url) -> tauri_plugin_updater::Result<url::Url> {
        Ok(reverse_proxy_server_mapper::apply_reverse_proxy_url(
            original_url.as_str(),
            self.reverse_proxy_url.as_deref(),
        )
        .parse()?)
    }

    fn endpoints(&self, config: &tauri_plugin_updater::Config) -> tauri_plugin_updater::Result<Vec<url::Url>> {
        config.endpoints.iter().map(|endpoint| self.url(endpoint)).collect()
    }

    fn apply_download(
        &self, original_url: &url::Url, download_url: &mut url::Url, proxy: &mut Option<url::Url>,
    ) -> tauri_plugin_updater::Result<()> {
        // 始终从清单中的原始地址生成下载路由，避免重试时重复拼接反代
        *download_url = self.url(original_url)?;
        // 未指定代理时清除检查阶段的显式代理，保留原有系统代理行为
        *proxy = self.proxy.clone();
        Ok(())
    }
}

async fn resolve_update_route(state: &tauri::State<'_, AppState>) -> tauri_plugin_updater::Result<UpdateRoute> {
    let app_proxy_id = global_config_mapper::get_cache("app_proxy_id", state).await;
    let proxy_url = proxy_server_mapper::get_app_proxy_url(app_proxy_id, state).await;
    let app_reverse_proxy_id = global_config_mapper::get_cache("app_reverse_proxy_id", state).await;
    let reverse_proxy_url = reverse_proxy_server_mapper::get_reverse_proxy_url(app_reverse_proxy_id, state).await;
    Ok(UpdateRoute { proxy: proxy_url.map(|url| url.parse()).transpose()?, reverse_proxy_url })
}

async fn check_update(app: &tauri::AppHandle) -> tauri_plugin_updater::Result<Option<Update>> {
    let state = app.state::<AppState>();
    let route = resolve_update_route(&state).await?;
    let user_agent = format!("loemby/{}", env!("CARGO_PKG_VERSION"));

    let mut updater = app
        .updater_builder()
        .timeout(std::time::Duration::from_secs(30))
        .header(reqwest::header::USER_AGENT, user_agent)?;
    // 读取合并后的运行时配置，只替换端点，保留插件的签名、版本和安装设置
    if let Some(config) = app.config().plugins.0.get("updater") {
        let config: tauri_plugin_updater::Config = serde_json::from_value(config.clone())?;
        updater = updater.endpoints(route.endpoints(&config)?)?;
    }
    if let Some(proxy_url) = route.proxy {
        updater = updater.proxy(proxy_url);
    }
    // 用户确认安装前，保留清单提供的原始下载地址
    updater.build()?.check().await
}

pub async fn install(app: tauri::AppHandle, version: String) -> Result<(), String> {
    let updater_state = app.state::<UpdaterState>();
    let mut pending = updater_state.pending.try_lock().map_err(|_| "正在检查或安装更新，请稍后再试".to_string())?;
    let update = pending.update.as_ref().ok_or_else(|| "请先检查更新".to_string())?;
    if update.version != version {
        return Err("更新版本已变化，请重新检查更新".to_string());
    }
    if pending.installed {
        return Ok(());
    }

    // 检查后可能修改代理配置；对副本应用当前配置，保留原始地址供重试
    let state = app.state::<AppState>();
    let route = resolve_update_route(&state).await.map_err(|err| format!("更新失败: {}", err))?;
    let mut routed_update = update.clone();
    route
        .apply_download(&update.download_url, &mut routed_update.download_url, &mut routed_update.proxy)
        .map_err(|err| format!("更新失败: {}", err))?;

    let mut downloaded = 0;
    routed_update
        .download_and_install(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                tracing::debug!("downloaded {downloaded} from {content_length:?}");
            },
            || {
                tracing::debug!("download finished");
            },
        )
        .await
        .map_err(|err| format!("更新失败: {}", err))?;

    pending.installed = true;
    tracing::info!("update installed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn route(proxy: Option<&str>, reverse_proxy_url: Option<&str>) -> UpdateRoute {
        UpdateRoute {
            proxy: proxy.map(|proxy| proxy.parse().unwrap()),
            reverse_proxy_url: reverse_proxy_url.map(str::to_string),
        }
    }

    #[test]
    fn manifest_route_uses_configured_endpoints_in_order() {
        let config: tauri_plugin_updater::Config = serde_json::from_value(serde_json::json!({
            "pubkey": "test-key",
            "endpoints": [
                "https://updates.example.test/custom/latest.json?target={{target}}&arch={{arch}}",
                "https://fallback.example.test/releases/{{current_version}}.json"
            ]
        }))
        .unwrap();
        let original_endpoints = config.endpoints.clone();
        let route = route(Some("http://forward.example.test:8080"), Some("https://reverse.example.test/prefix"));
        let endpoints = route.endpoints(&config).unwrap();

        let expected: Vec<Url> = original_endpoints
            .iter()
            .map(|endpoint| format!("https://reverse.example.test/prefix/{endpoint}").parse().unwrap())
            .collect();
        assert_eq!(endpoints, expected);
        assert_eq!(config.endpoints, original_endpoints);
    }

    #[test]
    fn manifest_route_without_reverse_keeps_configured_endpoints() {
        let config = tauri_plugin_updater::Config {
            endpoints: vec![
                "https://updates.example.test/custom/latest.json".parse().unwrap(),
                "https://fallback.example.test/latest.json".parse().unwrap(),
            ],
            ..Default::default()
        };
        assert_eq!(route(None, None).endpoints(&config).unwrap(), config.endpoints);
    }

    #[test]
    fn pending_download_uses_current_forward_and_reverse() {
        let original_url: Url = "https://artifacts.example.test/releases/update.tar.gz?token=abc".parse().unwrap();
        let mut download_url = original_url.clone();
        let mut proxy = Some("http://check-time-proxy.example.test:8080".parse().unwrap());
        let current_route =
            route(Some("socks5://install-proxy.example.test:1080"), Some("https://reverse.example.test/download"));

        current_route.apply_download(&original_url, &mut download_url, &mut proxy).unwrap();

        assert_eq!(
            download_url.as_str(),
            "https://reverse.example.test/download/https://artifacts.example.test/releases/update.tar.gz?token=abc"
        );
        assert_eq!(proxy, current_route.proxy);
        assert_eq!(original_url.as_str(), "https://artifacts.example.test/releases/update.tar.gz?token=abc");
    }

    #[test]
    fn changed_policy_routes_original_download_without_repeated_prefix() {
        let original_url: Url = "https://artifacts.example.test/update.tar.gz".parse().unwrap();
        let mut download_url = original_url.clone();
        let mut proxy = None;
        route(Some("http://old-proxy.example.test:8080"), Some("https://old-reverse.example.test"))
            .apply_download(&original_url, &mut download_url, &mut proxy)
            .unwrap();

        let current_route = route(Some("http://new-proxy.example.test:8080"), Some("https://new-reverse.example.test"));
        current_route.apply_download(&original_url, &mut download_url, &mut proxy).unwrap();
        current_route.apply_download(&original_url, &mut download_url, &mut proxy).unwrap();

        assert_eq!(
            download_url.as_str(),
            "https://new-reverse.example.test/https://artifacts.example.test/update.tar.gz"
        );
        assert_eq!(proxy, current_route.proxy);
        assert_eq!(original_url.as_str(), "https://artifacts.example.test/update.tar.gz");
    }

    #[test]
    fn disabled_policy_clears_stale_forward_and_reverse_routes() {
        let original_url: Url = "https://artifacts.example.test/update.tar.gz".parse().unwrap();
        let mut download_url = original_url.clone();
        let mut proxy = None;
        route(Some("http://old-proxy.example.test:8080"), Some("https://old-reverse.example.test"))
            .apply_download(&original_url, &mut download_url, &mut proxy)
            .unwrap();

        route(None, None).apply_download(&original_url, &mut download_url, &mut proxy).unwrap();

        assert_eq!(download_url, original_url);
        assert_eq!(proxy, None);
    }

    #[test]
    fn pending_download_already_using_current_reverse_is_not_prefixed_again() {
        let original_url: Url =
            "https://reverse.example.test/prefix/https://artifacts.example.test/update.tar.gz".parse().unwrap();
        let mut download_url = original_url.clone();
        let mut proxy = None;

        route(None, Some("https://reverse.example.test/prefix"))
            .apply_download(&original_url, &mut download_url, &mut proxy)
            .unwrap();

        assert_eq!(download_url, original_url);
        assert_eq!(proxy, None);
    }
}
