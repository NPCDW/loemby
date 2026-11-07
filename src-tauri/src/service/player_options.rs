use crate::{
    config::app_state::AppState,
    controller::invoke_ctl::PlayVideoParam,
    mapper::{emby_server_mapper::EmbyServer, global_config_mapper},
};

#[cfg(any(target_os = "android", test))]
pub(super) const CACHE_SPEED_SCRIPT: &str =
    include_str!("../../resources/mpv/portable_config/scripts/cache_speed.lua");

pub(super) struct PlayerOptions {
    pub config: String,
    pub options: Vec<(String, String)>,
}

pub(super) async fn player_options(
    body: &PlayVideoParam,
    state: &tauri::State<'_, AppState>,
    server: &EmbyServer,
) -> PlayerOptions {
    let settings = Settings {
        config: global_config_mapper::get_cache("mpv_args", state).await.unwrap_or_default(),
        volume: global_config_mapper::get_cache("mpv_volume", state).await,
        prefetch_playlist: global_config_mapper::get_cache("prefetch_playlist", state).await,
        cache_speed_enabled: global_config_mapper::get_cache("cache_speed_enabled", state).await,
        cache_speed_ass_style: global_config_mapper::get_cache("cache_speed_ass_style", state).await,
    };
    build_player_options(settings, server.user_agent.as_deref().unwrap_or("Loemby"), body.download == Some(true))
}

#[derive(Default)]
struct Settings {
    config: String,
    volume: Option<String>,
    prefetch_playlist: Option<String>,
    cache_speed_enabled: Option<String>,
    cache_speed_ass_style: Option<String>,
}

// mpv parses key/value-list values itself. Length quoting uses UTF-8 bytes, not
// characters, and keeps commas or a leading quote/bracket/percent sign literal.
fn script_value(value: &str) -> String {
    if value.contains(',') || value.starts_with(['"', '[', '%']) {
        format!("%{}%{}", value.len(), value)
    } else {
        value.to_string()
    }
}

fn build_player_options(settings: Settings, user_agent: &str, download: bool) -> PlayerOptions {
    let enabled = settings.cache_speed_enabled.as_deref().unwrap_or("true");
    let mut script_opts = format!("cache_speed-enabled={}", script_value(enabled));
    if let Some(style) = settings.cache_speed_ass_style.as_deref().filter(|value| !value.is_empty()) {
        script_opts.push_str(",cache_speed-ass_style=");
        script_opts.push_str(&script_value(style));
    }
    let prefetch = if download {
        "no".to_string()
    } else {
        settings.prefetch_playlist.unwrap_or_else(|| "no".to_string())
    };
    // Keep the desktop command-line ordering and precedence: mpv reads the
    // unmodified config first, then these application-owned options override it.
    // Never split mpv_args: comments, quoting, includes and profiles are mpv's job.
    PlayerOptions {
        config: settings.config,
        options: vec![
            ("terminal".to_string(), "no".to_string()),
            ("autoload-files".to_string(), "no".to_string()),
            ("demuxer-lavf-o".to_string(), "protocol_whitelist=[file,http,https,crypto,data,tcp]".to_string()),
            ("user-agent".to_string(), user_agent.to_string()),
            ("volume".to_string(), settings.volume.unwrap_or_else(|| "100".to_string())),
            ("script-opts".to_string(), script_opts),
            ("prefetch-playlist".to_string(), prefetch),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value<'a>(options: &'a PlayerOptions, name: &str) -> &'a str {
        &options.options.iter().find(|(key, _)| key == name).unwrap().1
    }

    #[test]
    fn desktop_defaults_and_option_order_are_preserved() {
        let options = build_player_options(Settings::default(), "server UA", false);
        assert_eq!(options.config, "");
        assert_eq!(options.options, vec![
            ("terminal".into(), "no".into()),
            ("autoload-files".into(), "no".into()),
            ("demuxer-lavf-o".into(), "protocol_whitelist=[file,http,https,crypto,data,tcp]".into()),
            ("user-agent".into(), "server UA".into()),
            ("volume".into(), "100".into()),
            ("script-opts".into(), "cache_speed-enabled=true".into()),
            ("prefetch-playlist".into(), "no".into()),
        ]);
    }

    #[test]
    fn custom_settings_and_raw_config_are_preserved() {
        let config = "# 原始配置\r\nvolume=12\r\ninclude=\"配置 路径.conf\"\r\n[custom]\r\nprofile-cond=path:find('a,b')\r\n";
        let style = "{\\an9\\3c&HA066FD&}";
        let options = build_player_options(Settings {
            config: config.to_string(),
            volume: Some("87.5".to_string()),
            prefetch_playlist: Some("yes".to_string()),
            cache_speed_enabled: Some("false".to_string()),
            cache_speed_ass_style: Some(style.to_string()),
        }, "Custom/1.0 (测试)", false);
        assert_eq!(options.config, config);
        assert_eq!(value(&options, "user-agent"), "Custom/1.0 (测试)");
        assert_eq!(value(&options, "volume"), "87.5");
        assert_eq!(value(&options, "prefetch-playlist"), "yes");
        assert_eq!(value(&options, "script-opts"), format!("cache_speed-enabled=false,cache_speed-ass_style={style}"));
    }

    #[test]
    fn download_disables_prefetch_even_when_enabled() {
        for prefetch_playlist in [None, Some("yes".to_string()), Some("no".to_string())] {
            let options = build_player_options(Settings { prefetch_playlist, ..Default::default() }, "UA", true);
            assert_eq!(value(&options, "prefetch-playlist"), "no");
        }
    }

    #[test]
    fn volume_above_100_is_not_parsed_or_clamped() {
        let options = build_player_options(Settings { volume: Some("150.25".to_string()), ..Default::default() }, "UA", false);
        assert_eq!(value(&options, "volume"), "150.25");
    }

    #[test]
    fn special_characters_are_not_split_or_shell_escaped() {
        let ua = "UA 测试 / a=b, c; \"quoted\" \\path";
        let style = "{\\an9\\pos(10,20)}测试=\"值\"[%]";
        let options = build_player_options(Settings {
            cache_speed_ass_style: Some(style.to_string()), ..Default::default()
        }, ua, false);
        assert_eq!(value(&options, "user-agent"), ua);
        assert_eq!(value(&options, "script-opts"), format!("cache_speed-enabled=true,cache_speed-ass_style=%{}%{style}", style.len()));
        assert!(style.len() > style.chars().count());
        for value in ["\"quoted\"", "[brackets]", "%percent", "a,b"] {
            assert_eq!(script_value(value), format!("%{}%{value}", value.len()));
        }
    }

    #[test]
    fn empty_style_uses_the_bundled_script_default() {
        let options = build_player_options(Settings { cache_speed_ass_style: Some(String::new()), ..Default::default() }, "UA", false);
        assert_eq!(value(&options, "script-opts"), "cache_speed-enabled=true");
        assert!(CACHE_SPEED_SCRIPT.contains("options.read_options(o, \"cache_speed\")"));
    }
}
