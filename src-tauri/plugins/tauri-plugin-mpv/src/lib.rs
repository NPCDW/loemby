use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime, plugin::{Builder, PluginHandle, TauriPlugin, mobile::PluginInvokeError}};

pub struct Mpv<R: Runtime>(PluginHandle<R>);

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareOptions {
    pub title: Option<String>,
    pub start_paused: bool,
    pub config_path: Option<PathBuf>,
    pub mpv_options: Vec<(String, String)>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResult {
    pub session_id: String,
    pub socket_path: PathBuf,
}

impl<R: Runtime> Mpv<R> {
    pub async fn prepare(&self, session_id: &str, socket_path: &Path, options: PrepareOptions) -> Result<PrepareResult, PluginInvokeError> {
        self.0.run_mobile_plugin_async("prepare", serde_json::json!({
            "sessionId": session_id, "socketPath": socket_path, "options": options
        })).await
    }

    pub async fn close(&self, session_id: &str) -> Result<(), PluginInvokeError> {
        self.0.run_mobile_plugin_async("close", serde_json::json!({ "sessionId": session_id })).await
    }
}

pub trait MpvExt<R: Runtime> {
    fn mpv(&self) -> &Mpv<R>;
}

impl<R: Runtime, T: Manager<R>> MpvExt<R> for T {
    fn mpv(&self) -> &Mpv<R> { self.state::<Mpv<R>>().inner() }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mpv").setup(|app, api| {
        #[cfg(target_os = "android")]
        app.manage(Mpv(api.register_android_plugin("com.loemby.mpv", "MpvPlugin")?));
        #[cfg(not(target_os = "android"))]
        let _ = (app, api);
        Ok(())
    }).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_options_preserve_paths_and_ordered_string_pairs() {
        let options = PrepareOptions {
            title: Some("测试播放".into()),
            start_paused: true,
            config_path: Some(PathBuf::from("/data/配置 目录/player.conf")),
            mpv_options: vec![
                ("volume".into(), "150.25".into()),
                ("user-agent".into(), "UA 测试=a,b \"quoted\" \\path".into()),
                ("script".into(), "/data/配置 目录/cache_speed.lua".into()),
            ],
        };
        assert_eq!(serde_json::to_value(options).unwrap(), serde_json::json!({
            "title": "测试播放",
            "startPaused": true,
            "configPath": "/data/配置 目录/player.conf",
            "mpvOptions": [
                ["volume", "150.25"],
                ["user-agent", "UA 测试=a,b \"quoted\" \\path"],
                ["script", "/data/配置 目录/cache_speed.lua"]
            ]
        }));
    }

    #[test]
    fn prepare_options_default_has_no_config_or_options() {
        assert_eq!(serde_json::to_value(PrepareOptions::default()).unwrap(), serde_json::json!({
            "title": null, "startPaused": false, "configPath": null, "mpvOptions": []
        }));
    }
}
