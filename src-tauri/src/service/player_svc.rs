use std::{cmp::{max, min}, path::PathBuf};
#[cfg(desktop)]
use std::sync::Arc;

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use crate::{config::{app_state::{AppState, TauriNotify}, http_pool}, controller::{emby_http_ctl::{EmbyEpisodesParam, EmbyItemsParam, EmbyPlaybackInfoParam}, invoke_ctl::PlayVideoParam}, mapper::{emby_server_mapper::{self, EmbyServer}, global_config_mapper::{self, GlobalConfig}, play_history_mapper::{self, PlayHistory}, proxy_server_mapper}, service::{axum_svc::{AxumAppState, AxumAppStateEmbyStreamRequest, MediaPlaylistParam}, emby_http_svc::{self, EmbyGetAudioStreamUrlParam, EmbyGetSubtitleStreamUrlParam, EmbyGetVideoStreamUrlParam, EmbyPageList, EmbyPlayingParam, EmbyPlayingProgressParam, EmbyPlayingStoppedParam, EpisodeItem, MediaSource, PlaybackInfo, SeriesItem}, simkl_http_svc, trakt_http_svc::{self, TraktScrobbleParam}, yamtrack_http_svc::{self, YamTrackParam}}, util::{file_util, media_source_util}};

#[cfg(target_os = "android")]
#[path = "android_player.rs"]
mod android_player;

#[path = "playback_session.rs"]
mod playback_session;
use playback_session::{Players, Session};
use crate::service::{mpv_ipc, player_options::{player_options, PlayerOptions}};

pub fn init_player(app: &AppHandle) {
    app.manage(Players::default());
}

pub async fn call_player(body: PlayVideoParam, state: &tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    call_player_inner(body, state, app).await.map_err(|error| error.to_string())
}

async fn call_player_inner(body: PlayVideoParam, state: &tauri::State<'_, AppState>, app: AppHandle) -> anyhow::Result<()> {
    anyhow::ensure!(body.select_policy != "manual" || body.version_select > 0, "媒体版本编号必须从 1 开始");
    #[cfg(target_os = "android")]
    anyhow::ensure!(body.download != Some(true), "Android 暂未支持下载");
    let server = emby_server_mapper::get_cache(&body.emby_server_id, state).await
        .ok_or_else(|| anyhow::anyhow!("emby_server not found"))?;
    let options = player_options(&body, state, &server).await;
    #[cfg(target_os = "android")]
    { return android_player::call_player(body, state, app, &server, options).await; }
    #[cfg(desktop)]
    { return call_desktop_player(body, state, app, &server, options).await; }
    #[cfg(all(mobile, not(target_os = "android")))]
    { let _ = (body, app, server, options); anyhow::bail!("当前移动平台尚未支持原生播放"); }
}

#[cfg(desktop)]
async fn call_desktop_player(body: PlayVideoParam, state: &tauri::State<'_, AppState>, app_handle: AppHandle, emby_server: &EmbyServer, options: PlayerOptions) -> anyhow::Result<()> {
    let external_mpv_switch = global_config_mapper::get_cache("external_mpv_switch", state).await.unwrap_or("off".to_string());
    let (mpv_path, mpv_config_dir, mpv_startup_dir) = if external_mpv_switch == "on" {
        match global_config_mapper::get_cache("mpv_path", state).await {
            None => anyhow::bail!("未配置 mpv 路径"),
            Some(mpv_path) => {
                let mpv_path = mpv_path.trim().replace("\r", "");
                let mpv_path_vec = mpv_path.split("\n").collect::<Vec<&str>>();
                let mut final_mpv_path = None;
                let mut final_mpv_config_dir = None;
                for path in mpv_path_vec {
                    let path = path.split(";").collect::<Vec<&str>>();
                    if path.len() == 1 && PathBuf::from(path[0]).is_file() {
                        final_mpv_path = Some(PathBuf::from(path[0]));
                        final_mpv_config_dir = Some(PathBuf::from(path[0]).parent().unwrap().join("portable_config"));
                        break;
                    } else if path.len() == 2 && PathBuf::from(path[0]).is_file() && PathBuf::from(path[1]).is_dir() {
                        final_mpv_path = Some(PathBuf::from(path[0]));
                        final_mpv_config_dir = Some(PathBuf::from(path[1]));
                        break;
                    }
                }
                if final_mpv_path.is_none() {
                    anyhow::bail!("所有的外部 mpv 路径都不存在");
                }
                (final_mpv_path.unwrap(), final_mpv_config_dir.clone().unwrap(), final_mpv_config_dir.unwrap().parent().unwrap().to_path_buf())
            },
        }
    } else {
        #[cfg(target_os = "windows")]
        match app_handle.path().resolve("resources/mpv/mpv.exe", tauri::path::BaseDirectory::Resource,) {
            Err(err) => anyhow::bail!("内置 mpv 路径获取失败: {}", err),
            Ok(mpv_path) => (mpv_path.clone(), mpv_path.parent().unwrap().join("portable_config"), mpv_path.parent().unwrap().to_path_buf()),
        }
        #[cfg(target_os = "linux")]
        match app_handle.path().resolve("resources/mpv/mpv.AppImage", tauri::path::BaseDirectory::Resource,) {
            Err(err) => anyhow::bail!("内置 mpv 路径获取失败: {}", err),
            Ok(mpv_path) => (mpv_path.clone(), mpv_path.parent().unwrap().join("portable_config"), mpv_path.parent().unwrap().to_path_buf()),
        }
        #[cfg(target_os = "macos")]
        match app_handle.path().resolve("resources/mpv/portable_config", tauri::path::BaseDirectory::Resource,) {
            Err(err) => anyhow::bail!("内置 mpv 配置目录获取失败: {}", err),
            Ok(mpv_config_path) => match PathBuf::from("/usr/bin/mpv").is_file() {
                true => (PathBuf::from("/usr/bin/mpv"), mpv_config_path.clone(), mpv_config_path.parent().unwrap().to_path_buf()),
                false => anyhow::bail!("mpv 未安装，请先安装 mpv ，检测路径 /usr/bin/mpv"),
            },
        }
    };
    tokio::fs::create_dir_all(&mpv_config_dir).await?;
    let mpv_config_path = mpv_config_dir.join("loemby.mpv.conf");
    tokio::fs::write(&mpv_config_path, &options.config).await?;
    #[cfg(windows)]
    let pipe_name = r"\\.\pipe\mpvsocket";
    #[cfg(unix)]
    let pipe_name = r"/tmp/mpvsocket";
    let pipe_name = format!("{}-{}", pipe_name, uuid::Uuid::new_v4());
    let plan = prepare_playback_plan(&body, state, emby_server, &pipe_name).await?;
    let session = Arc::new(Session::new(uuid::Uuid::new_v4().to_string(), mpv_config_dir.clone(), plan.ids));
    let players = app_handle.state::<Players>();
    players.sessions.write().await.insert(pipe_name.clone(), session.clone());
    let mut command = tokio::process::Command::new(mpv_path.as_os_str().to_str().unwrap().replace(r"\\?\", ""));
    command.current_dir(&mpv_startup_dir)
        .arg(format!("--include={}", mpv_config_path.to_string_lossy().replace(r"\\?\", "")))
        .arg(format!("--config-dir={}", mpv_config_dir.to_string_lossy().replace(r"\\?\", "")));
    for (name, value) in options.options { command.arg(format!("--{name}={value}")); }
    command.arg(format!("--input-ipc-server={pipe_name}"))
        .arg("--idle=yes")
        .arg("--force-window=immediate");
    tracing::debug!("调用MPV: {:?}", command);
    let setup = async {
        let mut child = command.spawn()?;
        let connected = mpv_ipc::connect(&pipe_name).await;
        let (client, events) = match connected {
            Ok(connected) => connected,
            Err(error) => { let _ = child.kill().await; return Err(error); }
        };
        if let Err(error) = client.request(serde_json::json!(["observe_property", 10001, "volume"])).await {
            let _ = child.kill().await;
            return Err(error);
        }
        *session.child.lock().await = Some(child);
        playback_session::start_process(app_handle.clone(), pipe_name.clone(), session.clone(), client.clone(), events);
        let playlist = mpv_config_dir.join(format!("playlist-{}.m3u8", session.id));
        let load = async {
            tokio::fs::write(&playlist, plan.m3u).await?;
            client.request(serde_json::json!(["loadlist", playlist.to_string_lossy().replace(r"\\?\", ""), "replace", plan.start])).await?;
            Ok::<(), anyhow::Error>(())
        }.await;
        let _ = tokio::fs::remove_file(&playlist).await;
        if let Err(error) = load { let _ = client.request(serde_json::json!(["quit"])).await; return Err(error); }
        Ok(())
    }.await;
    if setup.is_err() { playback_session::cleanup(&app_handle, &pipe_name, &session).await; }
    setup
}

struct PlaybackPlan {
    m3u: String,
    start: usize,
    ids: Vec<String>,
}

async fn prepare_playback_plan(body: &PlayVideoParam, state: &tauri::State<'_, AppState>, emby_server: &EmbyServer, pipe_name: &str) -> anyhow::Result<PlaybackPlan> {
    let episode = emby_http_svc::items(EmbyItemsParam {
        emby_server_id: body.emby_server_id.clone(),
        item_id: body.item_id.clone(),
    }, state, true).await.map_err(anyhow::Error::msg)?;
    let episode = serde_json::from_str::<EpisodeItem>(&episode)?;
    let episodes = if let Some(series_id) = episode.series_id.clone() {
        let episodes = emby_http_svc::episodes(EmbyEpisodesParam {
            emby_server_id: body.emby_server_id.clone(),
            series_id,
            season_id: episode.season_id.clone().ok_or_else(|| anyhow::anyhow!("剧集缺少季编号"))?,
            start_item_id: None,
            start_index: None,
            limit: None,
            extend_fields: Some(false),
        }, state).await.map_err(anyhow::Error::msg)?;
        serde_json::from_str::<EmbyPageList<EpisodeItem>>(&episodes)?.items
    } else {
        vec![episode]
    };
    let start = episodes.iter().position(|episode| episode.id == body.item_id)
        .ok_or_else(|| anyhow::anyhow!("播放列表不包含选中的媒体"))?;
    let axum = state.auxm_app_state.read().await.clone().ok_or_else(|| anyhow::anyhow!("媒体服务尚未就绪"))?;
    let mut plan = PlaybackPlan { m3u: "#EXTM3U".to_string(), start, ids: Vec::new() };
    let mut playlist = axum.playlist.write().await;
    for (index, episode) in episodes.iter().enumerate() {
        let id = uuid::Uuid::new_v4().to_string();
        let series_name = episode.series_name.clone().unwrap_or("🎬电影".to_string());
        let season = episode.parent_index_number.map_or("_".to_string(), |value| value.to_string());
        let number = episode.index_number.map_or("_".to_string(), |value| value.to_string());
        let prefix = if episode.series_id.is_some() { format!("S{}E{}. ", season, number) } else { String::new() };
        let item_name = format!("{}{}", prefix, episode.name);
        let title = format!("{} | {} | {}", item_name, series_name, emby_server.server_name.as_deref().unwrap_or("Emby"));
        playlist.insert(id.clone(), MediaPlaylistParam {
            playlist_index: index + 1,
            playlist_total: episodes.len(),
            emby_server_id: body.emby_server_id.clone(),
            series_id: body.series_id.clone(),
            series_name,
            item_id: episode.id.clone(),
            item_name,
            playback_position_ticks: if episode.id == body.item_id { body.playback_position_ticks } else { 0 },
            use_direct_link: body.use_direct_link,
            select_policy: body.select_policy.clone(),
            version_select: body.version_select,
            video_select: body.video_select,
            audio_select: body.audio_select,
            subtitle_select: body.subtitle_select,
            mpv_ipc: pipe_name.to_string(),
            media_title: title.clone(),
            download: body.download.unwrap_or(false),
        });
        let title = title.replace(['\r', '\n'], " ");
        plan.m3u.push_str(&format!("\n#EXTINF:-1,{}\nhttp://127.0.0.1:{}/play_media/{}/0", title, axum.port, id));
        plan.ids.push(id);
    }
    Ok(plan)
}

pub async fn play_media(axum_app_state: &AxumAppState, id: &str, media_source_select: usize) -> anyhow::Result<String> {
    let playlist = axum_app_state.playlist.read().await.clone();
    let params = playlist.get(id).ok_or(anyhow::anyhow!("媒体ID不存在"))?;

    let app_state = axum_app_state.app.state::<AppState>();
    let emby_server = match emby_server_mapper::get_cache(&params.emby_server_id, &app_state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let play_proxy_url = proxy_server_mapper::get_play_proxy_url(emby_server.play_proxy_id.clone(), &app_state).await;

    // 获取媒体信息
    let playback_info = emby_http_svc::playback_info(EmbyPlaybackInfoParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
    }, &app_state).await?;
    let playback_info = serde_json::from_str::<PlaybackInfo>(&playback_info)?;
    if let Some(error_code) = playback_info.error_code {
        return Err(anyhow::anyhow!("Emby Playback Info Error: {}", error_code));
    }
    if playback_info.media_sources.len() <= 0 {
        return Err(anyhow::anyhow!("Emby Playback Info Error: 没有可用的媒体源"));
    }
    // 自动或手动选择媒体源
    let media_source_index = if media_source_select != 0 {
        if playback_info.media_sources.len() >= media_source_select {
            media_source_select - 1
        } else {
            0
        }
    } else {
        if params.select_policy == "manual" && params.version_select > 0 && playback_info.media_sources.len() >= params.version_select {
            params.version_select - 1
        } else {
            #[derive(Debug, Clone)]
            struct VersionSelect {
                version_id: u32,
                size: u64,
                resolution_level: u32,
            }
            let mut version_select_list: Vec<VersionSelect> = Vec::new();
            for (i, media_source) in playback_info.media_sources.iter().enumerate() {
                version_select_list.push(VersionSelect {
                    version_id: (i + 1) as u32,
                    size: media_source.size.unwrap_or(0),
                    resolution_level: media_source_util::get_resolution_level_from_media_sources(media_source),
                });
            }
            let select_policy = global_config_mapper::get_cache("play_version_auto_select_policy", &app_state).await.unwrap_or("high-resolution".to_string());
            if select_policy == "high-bitrate" {
                version_select_list.sort_by(|a, b| b.size.cmp(&a.size));
            } else if select_policy == "high-resolution" {
                version_select_list.sort_by(|a, b| {
                    if a.resolution_level != b.resolution_level {
                        b.resolution_level.cmp(&a.resolution_level)
                    } else if a.size != b.size {
                        b.size.cmp(&a.size)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            version_select_list[0].version_id as usize - 1
        }
    };
    let media_source = &playback_info.media_sources[media_source_index];
    // 选中媒体源的视频地址
    let support_direct_link = media_source.is_remote == Some(true) && media_source.path.is_some() && media_source.path.as_ref().ok_or(anyhow::anyhow!("路径解析错误"))?.contains("://") && !media_source_util::is_internal_url(&media_source.path.as_ref().unwrap());
    let mut video_url = if params.use_direct_link && support_direct_link {
        media_source.path.clone().unwrap()
    } else if let Some(direct_stream_url) = media_source.direct_stream_url.clone() {
        emby_http_svc::get_half_direct_stream_url(direct_stream_url)
    } else {
        emby_http_svc::get_half_video_stream_url(EmbyGetVideoStreamUrlParam {
            item_id: params.item_id.clone(),
            container: media_source.container.clone(),
            media_source_id: media_source.id.clone(),
            play_session_id: playback_info.play_session_id.clone(),
        })
    };
    let uuid = uuid::Uuid::new_v4().to_string();
    axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateEmbyStreamRequest {
        stream_url: video_url,
        emby_server_id: emby_server.id.clone().unwrap(),
    });
    video_url = format!("http://127.0.0.1:{}/stream/video/{}", &axum_app_state.port, &uuid);

    if params.download {
        let notify_param = DownloadNotifyParamEvent {
            title: format!("视频下载 - {}", params.media_title.clone()),
            url: video_url.clone(),
        };
        axum_app_state.app.emit("tauri_notify", TauriNotify {
            event_type: "DownloadNotify".to_string(),
            message_type: "success".to_string(),
            title: None,
            message: serde_json::to_string(&notify_param).unwrap(),
        }).map_err(|e| anyhow::anyhow!(e))?;
    }

    // 播放进程
    let playback_process_param = PlaybackProcessParam {
        params: params.clone(),
        episode: None,
        media_source: media_source.clone(),
        playback_info: playback_info,
        app_handle: axum_app_state.app.clone(),
        axum_app_state: axum_app_state.clone(),
        scrobble_trakt_param: None,
        scrobble_simkl_param: None,
        scrobble_yamtrack_param: None,
        file_duration: 0.0,
        start_time: chrono::Local::now().timestamp(),
        emby_server: emby_server,
        play_proxy_url,
        id: id.to_string(),
        media_source_select,
        media_source_index,
        sender: None,
        play_info_init_finished: false,
    };
    cache_subtitles(&playback_process_param).await?;
    playback_session::register_media(playback_process_param, uuid, video_url.clone()).await?;
    Ok(video_url)
}

async fn cache_subtitles(playback_process_param: &PlaybackProcessParam) -> anyhow::Result<()> {
    let PlaybackProcessParam { params, media_source, app_handle, emby_server, play_proxy_url, .. } = playback_process_param;
    // 缓存字幕，作用不止于此，播放前和播放后，获取到的媒体元数据信息不一致（主要是字幕的索引位置变化），导致字幕无法正常显示，所以这里缓存字幕
    for media_stream in &media_source.media_streams {
        if media_stream.type_ == "Subtitle" && media_stream.is_external == Some(true) {
            let subtitle_url = emby_http_svc::get_half_subtitle_stream_url(EmbyGetSubtitleStreamUrlParam {
                item_id: params.item_id.clone(),
                media_source_id: media_source.id.clone(),
                media_source_item_id: media_source.item_id.clone(),
                media_streams_codec: media_stream.codec.clone(),
                media_streams_index: media_stream.index,
                media_streams_is_external: true,
            })?;
            let subtitle_url = format!("{}{}", emby_server.reverse_base_url.as_ref().unwrap(), subtitle_url);
            let cache_digest = sha256::digest(subtitle_url.clone());
            let cache_file_path = app_handle.path().resolve(&format!("cache/subtitle/{}.ass", cache_digest), tauri::path::BaseDirectory::AppLocalData)?;
            if cache_file_path.exists() {
                tracing::debug!("cache subtitle file exists: {}", cache_file_path.display());
                continue;
            }
            let cache_file_tmp_path = app_handle.path().resolve(&format!("cache/subtitle/{}.ass.tmp", cache_digest), tauri::path::BaseDirectory::AppLocalData)?;
            let (play_proxy_url, app_handle, emby_server) = (play_proxy_url.clone(), app_handle.clone(), emby_server.clone(),);
            tokio::spawn(async move {
                async fn download_subtitle_file(subtitle_url: String, cache_file_path: PathBuf, play_proxy_url: Option<String>, emby_server: EmbyServer, app_state: tauri::State<'_, AppState>, cache_file_tmp_path: PathBuf) -> anyhow::Result<()> {
                    let client = http_pool::get_image_http_client(play_proxy_url.clone(), &app_state).await?;
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(axum::http::header::USER_AGENT, reqwest::header::HeaderValue::from_str(&emby_server.user_agent.clone().unwrap()).unwrap());
                    headers.insert(axum::http::HeaderName::from_str("X-Emby-Token").unwrap(), reqwest::header::HeaderValue::from_str(&emby_server.auth_token.clone().unwrap()).unwrap());
                    headers.insert("X-Emby-Client", reqwest::header::HeaderValue::from_str(&emby_server.client.clone().unwrap()).unwrap());
                    headers.insert("X-Emby-Device-Name", reqwest::header::HeaderValue::from_str(&emby_server.device.clone().unwrap()).unwrap());
                    headers.insert("X-Emby-Device-Id", reqwest::header::HeaderValue::from_str(&emby_server.device_id.clone().unwrap()).unwrap());
                    headers.insert("X-Emby-Client-Version", reqwest::header::HeaderValue::from_str(&emby_server.client_version.clone().unwrap()).unwrap());
                    let builder = client
                        .get(subtitle_url.clone())
                        .headers(headers);
                    let builder_print = format!("{:?}", &builder);
                    let res = builder.send().await;
                    tracing::debug!("cache subtitle request: {:?} response {:?}", builder_print, res);
                    let response = res?;
                    if response.status() != 200 {
                        return Err(anyhow::anyhow!("cache subtitle request failed {} {}", subtitle_url, response.status()));
                    }
                    let mut stream = response.bytes_stream();
                    if !cache_file_tmp_path.parent().unwrap().exists() {
                        std::fs::create_dir_all(cache_file_tmp_path.parent().unwrap())?;
                    }
                    let mut cache_tmp_file = tokio::fs::File::create(&cache_file_tmp_path).await?;
                    while let Some(Ok(chunk)) = stream.next().await {
                        cache_tmp_file.write_all(&chunk).await?;
                    }
                    cache_tmp_file.flush().await?;
                    drop(cache_tmp_file);
                    let _ = file_util::rename(cache_file_tmp_path, cache_file_path);
                    Ok(())
                }
                let app_state = app_handle.state::<AppState>();
                match download_subtitle_file(subtitle_url, cache_file_path, play_proxy_url, emby_server, app_state, cache_file_tmp_path).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("cache subtitle file failed: {}", e);
                    }
                }
            });
        }
    }

    Ok(())
}

fn media_cache_bytes(bitrate: u64, seconds: u64, minimum: u64, maximum: u64, fallback: u64) -> u64 {
    let calculated = max(min(seconds.saturating_mul(bitrate) / 8, maximum.saturating_mul(1024 * 1024)), minimum.saturating_mul(1024 * 1024));
    if bitrate == 0 || calculated == 0 { fallback } else { calculated }
}

#[cfg(test)]
mod cache_tests {
    use super::media_cache_bytes;

    #[test]
    fn defaults_limits_and_invalid_sizes_do_not_overflow() {
        let mib = 1024 * 1024;
        assert_eq!(media_cache_bytes(0, 60, 10, 100, 300 * mib), 300 * mib);
        assert_eq!(media_cache_bytes(8 * mib, 0, 0, 0, 300 * mib), 300 * mib);
        assert_eq!(media_cache_bytes(8 * mib, 60, 10, 100, 300 * mib), 60 * mib);
        assert_eq!(media_cache_bytes(8 * mib, 600, 10, 100, 300 * mib), 100 * mib);
        assert_eq!(media_cache_bytes(8 * mib, 1, 10, 100, 300 * mib), 10 * mib);
        assert!(media_cache_bytes(u64::MAX, u64::MAX, 0, 100, 300 * mib) <= 100 * mib);
    }
}

async fn apply_media_cache(playback_process_param: &PlaybackProcessParam, client: &mpv_ipc::Client) -> anyhow::Result<()> {
    let media_source = &playback_process_param.media_source;
    let app_state = playback_process_param.app_handle.state::<AppState>();
    // 缓存大小
    let mut cache_max_bytes = 300 * 1024 * 1024;
    let mut cache_back_max_bytes = 200 * 1024 * 1024;
    if media_source.bitrate.is_some() && media_source.bitrate.unwrap() > 0 {
        let mpv_cache_seconds = global_config_mapper::get_cache("mpv_cache_seconds", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(300);
        let mpv_cache_min_bytes = global_config_mapper::get_cache("mpv_cache_min_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(300);
        let mpv_cache_max_bytes = global_config_mapper::get_cache("mpv_cache_max_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(300);
        let mpv_cache_back_seconds = global_config_mapper::get_cache("mpv_cache_back_seconds", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(200);
        let mpv_cache_back_min_bytes = global_config_mapper::get_cache("mpv_cache_back_min_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(200);
        let mpv_cache_back_max_bytes = global_config_mapper::get_cache("mpv_cache_back_max_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(200);
        let mpv_cache_max_bytes = media_cache_bytes(media_source.bitrate.unwrap(), mpv_cache_seconds, mpv_cache_min_bytes, mpv_cache_max_bytes, cache_max_bytes);
        let mpv_cache_back_max_bytes = media_cache_bytes(media_source.bitrate.unwrap(), mpv_cache_back_seconds, mpv_cache_back_min_bytes, mpv_cache_back_max_bytes, cache_back_max_bytes);

        if mpv_cache_max_bytes > 0 {
            cache_max_bytes = mpv_cache_max_bytes;
        }
        if mpv_cache_back_max_bytes > 0 {
            cache_back_max_bytes = mpv_cache_back_max_bytes;
        }
    }
    client.request(serde_json::json!(["set_property", "demuxer-max-bytes", cache_max_bytes])).await?;
    client.request(serde_json::json!(["set_property", "demuxer-max-back-bytes", cache_back_max_bytes])).await?;
    Ok(())
}

async fn play_info_init(playback_process_param: &PlaybackProcessParam) -> anyhow::Result<(EpisodeItem, Option<TraktScrobbleParam>, Option<TraktScrobbleParam>, Option<YamTrackParam>)> {
    let PlaybackProcessParam {
        params,
        media_source,
        playback_info,
        app_handle,
        axum_app_state,
        emby_server,
        sender,
        ..
    } = playback_process_param;
    #[cfg(desktop)]
    let PlaybackProcessParam { id, media_source_select, media_source_index, .. } = playback_process_param;
    let app_state = app_handle.state::<AppState>();
    let sender = sender.clone().unwrap();
    
    // 本地播放历史记录
    let episode = match emby_http_svc::items(EmbyItemsParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
    }, &app_state, true).await {
        Err(e) => return Err(anyhow::anyhow!("获取剧集信息失败: {}", e.to_string())),
        Ok(episode) => serde_json::from_str::<EpisodeItem>(&episode)?,
    };
    let mut pinned = 0;
    if let Some(series_id) = episode.series_id.clone() {
        let pinned_update = play_history_mapper::cancel_pinned(params.emby_server_id.clone(), series_id, &app_state.db_pool).await?;
        if pinned_update.rows_affected() > 0 { pinned = 1 }
    }
    match play_history_mapper::get(params.emby_server_id.clone(), params.item_id.clone(), &app_state.db_pool).await? {
        Some(response) => {
            if episode.series_id.is_none() {
                pinned = response.pinned.unwrap();
            }
            play_history_mapper::update_by_id(PlayHistory {
                id: response.id,
                emby_server_name: emby_server.server_name.clone(),
                item_name: Some(params.item_name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                pinned: Some(pinned),
                ..Default::default()
            }, &app_state.db_pool).await?;
        },
        None => {
            play_history_mapper::create(PlayHistory {
                id: Some(uuid::Uuid::new_v4().to_string()),
                emby_server_id: emby_server.id.clone(),
                emby_server_name: emby_server.server_name.clone(),
                item_id: Some(episode.id.clone()),
                item_name: Some(params.item_name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                played_duration: Some(0),
                pinned: Some(pinned),
                ..Default::default()
            }, &app_state.db_pool).await?;
        },
    }

    // 发送给前端播放通知
    axum_app_state.app.emit("tauri_notify", TauriNotify {
        event_type: "playingNotify".to_string(),
        message_type: "success".to_string(),
        title: None,
        message: serde_json::to_string(&PlaybackNotifyParam {
            emby_server_id: &params.emby_server_id,
            item_id: &params.item_id,
            item_name: &params.item_name,
            series_id: &params.series_id,
            series_name: &params.series_name,
            event: "start",
        })?,
    })?;

    // 桌面 uosc 的多版本菜单；Android 使用原生界面。
    #[cfg(desktop)]
    {
    #[derive(Debug, Serialize, Deserialize)]
    struct MutiVersionCommand {
        path: String,
        title: String,
        hint: String,
    }
    let mut muti_version_list: Vec<MutiVersionCommand> = Vec::new();
    for (i, media_source) in playback_info.media_sources.iter().enumerate() {
        let media_source_select = if media_source_select == &0 && media_source_index == &i { 0 } else { i + 1 };
        muti_version_list.push(MutiVersionCommand {
            path: format!("http://127.0.0.1:{}/play_media/{}/{}", &axum_app_state.port, id, media_source_select),
            title: media_source_util::get_display_title_from_media_sources(media_source),
            hint: format!("{}, {}, {}", media_source_util::get_resolution_from_media_sources(media_source), media_source_util::format_bytes(media_source.size.unwrap_or(0)), media_source_util::format_mbps(media_source.bitrate.unwrap_or(0)))
        });
    }
    let muti_version = serde_json::to_string(&muti_version_list)?.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
    let set_muti_version_command = format!(r#"{{ "command": ["script-message-to", "uosc", "set-muti-version", "{}"] }}{}"#, muti_version, "\n");
    sender.write().await.write_all(set_muti_version_command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command set-muti-version: {}", set_muti_version_command);
    }
    
    // 发送媒体标题
    let set_force_media_title_command = serde_json::to_string(&serde_json::json!({ "command": ["set_property", "force-media-title", params.media_title] }))? + "\n";
    sender.write().await.write_all(set_force_media_title_command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command force-media-title: {}", set_force_media_title_command);

    // 定位播放位置
    if params.playback_position_ticks != 0 {
        let command = format!(r#"{{ "command": ["seek", "{}", "absolute"] }}{}"#, params.playback_position_ticks / 1000_0000, "\n");
        sender.write().await.write_all(command.as_bytes()).await?;
        sender.write().await.flush().await?;
        tracing::debug!("MPV IPC Command seek: {}", command);
    }
    
    // 添加外部音频和外部字幕
    for media_stream in &media_source.media_streams {
        if media_stream.is_external != Some(true) {
            continue;
        }
        if media_stream.type_ == "Audio" {
            let mut audio_url = emby_http_svc::get_half_audio_stream_url(EmbyGetAudioStreamUrlParam {
                item_id: params.item_id.clone(),
                media_source_item_id: media_source.item_id.clone(),
                media_streams_codec: media_stream.codec.clone(),
                media_streams_index: media_stream.index,
                media_streams_is_external: true,
            })?;
            let uuid = uuid::Uuid::new_v4().to_string();
            axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateEmbyStreamRequest {
                stream_url: audio_url,
                emby_server_id: emby_server.id.clone().unwrap(),
            });
            playback_session::register_request(app_handle, &params.mpv_ipc, &uuid).await?;
            audio_url = format!("http://127.0.0.1:{}/stream/audio/{}", &axum_app_state.port, &uuid);
            let command = serde_json::to_string(&serde_json::json!({ "command": ["audio-add", audio_url, "auto", media_stream.display_title.as_deref().unwrap_or("")] }))? + "\n";
            sender.write().await.write_all(command.as_bytes()).await?;
            sender.write().await.flush().await?;
            tracing::debug!("MPV IPC Command audio-add: {}", command);
        } else if media_stream.type_ == "Subtitle" {
            let mut subtitle_url = emby_http_svc::get_half_subtitle_stream_url(EmbyGetSubtitleStreamUrlParam {
                item_id: params.item_id.clone(),
                media_source_id: media_source.id.clone(),
                media_source_item_id: media_source.item_id.clone(),
                media_streams_codec: media_stream.codec.clone(),
                media_streams_index: media_stream.index,
                media_streams_is_external: true,
            })?;
            let uuid = uuid::Uuid::new_v4().to_string();
            axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateEmbyStreamRequest {
                stream_url: subtitle_url,
                emby_server_id: emby_server.id.clone().unwrap(),
            });
            playback_session::register_request(app_handle, &params.mpv_ipc, &uuid).await?;
            subtitle_url = format!("http://127.0.0.1:{}/subtitle/{}", &axum_app_state.port, &uuid);
            let command = serde_json::to_string(&serde_json::json!({ "command": ["sub-add", subtitle_url, "select", media_stream.display_title.as_deref().unwrap_or("")] }))? + "\n";
            sender.write().await.write_all(command.as_bytes()).await?;
            sender.write().await.flush().await?;
            tracing::debug!("MPV IPC Command sub-add: {}", command);
            
            if params.download {
                let notify_param = DownloadNotifyParamEvent {
                    title: format!("字幕下载 - {}", media_stream.display_title.clone().unwrap_or("".to_string())),
                    url: subtitle_url.clone(),
                };
                axum_app_state.app.emit("tauri_notify", TauriNotify {
                    event_type: "DownloadNotify".to_string(),
                    message_type: "success".to_string(),
                    title: None,
                    message: serde_json::to_string(&notify_param).unwrap(),
                }).map_err(|e| anyhow::anyhow!(e))?;
            }
        }
    }
    // 手动或自动选择媒体音频和字幕
    let mut vid = 0;
    let mut aid = 0;
    let mut sid = 0;
    let mut video_index = 0;
    let mut audio_index = 0;
    let mut subtitle_index = 0;
    let mut subtitle_max_score = 0;

    let mut track_titles = TrackTitleParam { video: Vec::new(), audio: Vec::new(), sub: Vec::new(), };
    for media_stream in &media_source.media_streams {
        if media_stream.type_ == "Video" {
            track_titles.video.push(format!("{} / {}", media_source.name.clone(), media_stream.display_title.clone().unwrap_or("".to_string())));
            video_index += 1;
            if media_stream.is_default == Some(true) {
                vid = video_index;
            }
        } else if media_stream.type_ == "Audio" {
            if media_stream.is_external != Some(true) {
                track_titles.audio.push(media_stream.display_title.clone().unwrap_or("".to_string()));
            }
            audio_index += 1;
            if media_stream.is_default == Some(true) && aid == 0 {
                aid = audio_index;
            }
        } else if media_stream.type_ == "Subtitle" {
            if media_stream.is_external != Some(true) {
                track_titles.sub.push(media_stream.display_title.clone().unwrap_or("".to_string()));
            }
            subtitle_index += 1;
            let mut score = 0;
            if media_stream.is_default == Some(true) {
                score += 1;
            }
            if media_stream.is_external == Some(true) {
                score += 2;
            }
            if let Some(lang) = &media_stream.display_language {
                if lang.contains("Chinese") {
                    score += 3;
                    if lang.contains("Simplified") {
                        score += 1;
                    }
                }
                if lang.contains("中") {
                    score += 3;
                    if lang.contains("简") {
                        score += 1;
                    }
                }
            }
            if score > subtitle_max_score {
                subtitle_max_score = score;
                sid = subtitle_index;
            }
        }
    }
    #[cfg(desktop)]
    {
    let track_titles = serde_json::to_string(&track_titles)?.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
    let set_track_titles_command = format!(r#"{{ "command": ["script-message-to", "uosc", "set-track-title", "{}"] }}{}"#, track_titles, "\n");
    sender.write().await.write_all(set_track_titles_command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command set_track_titles: {}", set_track_titles_command);
    }

    if params.select_policy == "manual" {
        vid = params.video_select;
        aid = params.audio_select;
        sid = params.subtitle_select;
    } else {
        if vid == 0 && video_index > 0 {
            vid = 1;
        }
        if aid == 0 && audio_index > 0 {
            aid = 1;
        }
        if sid == 0 && subtitle_index > 0 {
            sid = 1;
        }
    }
    let property = match vid {
        -1 => "no".to_string(),
        0 => "auto".to_string(),
        _ => vid.to_string()
    };
    let command = format!(r#"{{ "command": ["set_property", "vid", "{}"] }}{}"#, property, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command vid: {}", command);
    let property = match aid {
        -1 => "no".to_string(),
        0 => "auto".to_string(),
        _ => aid.to_string()
    };
    let command = format!(r#"{{ "command": ["set_property", "aid", "{}"] }}{}"#, property, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command aid: {}", command);
    let property = match sid {
        -1 => "no".to_string(),
        0 => "auto".to_string(),
        _ => sid.to_string()
    };
    let command = format!(r#"{{ "command": ["set_property", "sid", "{}"] }}{}"#, property, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command sid: {}", command);

    tracing::debug!("init mpv play info finished");
    
    // emby开始播放api
    let res = emby_http_svc::playing(EmbyPlayingParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
        media_source_id: media_source.id.clone(),
        play_session_id: playback_info.play_session_id.clone(),
        position_ticks: params.playback_position_ticks,
    }, &app_state).await;
    if let Err(e) = res {
        tracing::error!("调用Emby开始播放失败: {}", e.to_string());
    }

    let command = format!(r#"{{ "command": ["print-text", "Emby播放初始化完成"] }}{}"#, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;
    tracing::debug!("MPV IPC Command print-text: {}", command);

    // trakt 开始播放
    let series = if let Some(series_id) = episode.series_id.clone() {
        match emby_http_svc::items(EmbyItemsParam {
            emby_server_id: params.emby_server_id.clone(),
            item_id: series_id,
        }, &app_state, true).await {
            Err(e) => return Err(anyhow::anyhow!("获取系列信息失败: {}", e.to_string())),
            Ok(series) => serde_json::from_str::<SeriesItem>(&series).ok(),
        }
    } else { None };
    let trakt_sync_switch = global_config_mapper::get_cache("trakt_sync_switch", &app_state).await;
    let trakt_username = global_config_mapper::get_cache("trakt_username", &app_state).await;
    let scrobble_trakt_param = if !params.download && trakt_sync_switch != Some("off".to_string()) && trakt_username.is_some() {
        let trakt_scrobble_param = trakt_http_svc::get_scrobble_trakt_param(&episode, &series, 0.0);
        if let Some(scrobble_trakt_param) = &trakt_scrobble_param {
            match trakt_http_svc::start(scrobble_trakt_param, &app_state, 0).await {
                Ok(json) => 
                    app_handle.emit("tauri_notify", TauriNotify {
                        event_type: "TraktNotify".to_string(),
                        message_type: "start".to_string(),
                        title: None,
                        message: json,
                    })?,
                Err(err) => 
                    app_handle.emit("tauri_notify", TauriNotify {
                        event_type: "TraktNotify".to_string(),
                        message_type: "error".to_string(),
                        title: None,
                        message: format!("调用trakt开始播放失败: {}", err),
                    })?
            }
        }
        trakt_scrobble_param
    } else { None };

    // simkl 开始播放
    let series = if let Some(series_id) = episode.series_id.clone() {
        match emby_http_svc::items(EmbyItemsParam {
            emby_server_id: params.emby_server_id.clone(),
            item_id: series_id,
        }, &app_state, true).await {
            Err(e) => return Err(anyhow::anyhow!("获取系列信息失败: {}", e.to_string())),
            Ok(series) => serde_json::from_str::<SeriesItem>(&series).ok(),
        }
    } else { None };
    let simkl_sync_switch = global_config_mapper::get_cache("simkl_sync_switch", &app_state).await;
    let simkl_username = global_config_mapper::get_cache("simkl_username", &app_state).await;
    let scrobble_simkl_param = if !params.download && simkl_sync_switch != Some("off".to_string()) && simkl_username.is_some() {
        let simkl_scrobble_param = trakt_http_svc::get_scrobble_trakt_param(&episode, &series, 0.0);
        if let Some(scrobble_simkl_param) = &simkl_scrobble_param {
            match simkl_http_svc::start(scrobble_simkl_param, &app_state, 0).await {
                Ok(json) => 
                    app_handle.emit("tauri_notify", TauriNotify {
                        event_type: "SimklNotify".to_string(),
                        message_type: "start".to_string(),
                        title: None,
                        message: json,
                    })?,
                Err(err) => 
                    app_handle.emit("tauri_notify", TauriNotify {
                        event_type: "SimklNotify".to_string(),
                        message_type: "error".to_string(),
                        title: None,
                        message: format!("调用simkl开始播放失败: {}", err),
                    })?
            }
        }
        simkl_scrobble_param
    } else { None };

    // YamTrack 开始播放
    let yamtrack_sync_switch = global_config_mapper::get_cache("yamtrack_sync_switch", &app_state).await;
    let yamtrack_sync_url = global_config_mapper::get_cache("yamtrack_sync_url", &app_state).await;
    let scrobble_yamtrack_param = if !params.download && yamtrack_sync_switch != Some("off".to_string()) && yamtrack_sync_url.is_some() {
        let yamtrack_scrobble_param = yamtrack_http_svc::get_scrobble_yamtrack_param(&episode, &series, true, false);
        if let Some(scrobble_yamtrack_param) = &yamtrack_scrobble_param {
            match yamtrack_http_svc::track(scrobble_yamtrack_param, &app_state).await {
                Ok(json) => 
                    app_handle.emit("tauri_notify", TauriNotify {
                        event_type: "YamTrackNotify".to_string(),
                        message_type: "start".to_string(),
                        title: None,
                        message: json,
                    })?,
                Err(err) => 
                    app_handle.emit("tauri_notify", TauriNotify {
                        event_type: "YamTrackNotify".to_string(),
                        message_type: "error".to_string(),
                        title: None,
                        message: format!("调用YamTrack开始播放失败: {}", err),
                    })?
            }
        }
        yamtrack_scrobble_param
    } else { None };

    Ok((episode, scrobble_trakt_param, scrobble_simkl_param, scrobble_yamtrack_param))
}

async fn save_playback_progress(playback_process_param: &PlaybackProcessParam, last_record_position: Decimal, playback_status: PlayingProgressEnum) -> anyhow::Result<()> {
    let PlaybackProcessParam {
        params,
        episode,
        media_source,
        playback_info,
        app_handle,
        scrobble_trakt_param,
        scrobble_simkl_param,
        scrobble_yamtrack_param,
        file_duration,
        start_time,
        emby_server,
        play_info_init_finished,
        ..
    } = playback_process_param;
    if !play_info_init_finished {
        return Ok(());
    }
    let episode = episode.as_ref().unwrap();

    let position_ticks = (last_record_position * Decimal::from_i64(1000_0000).unwrap()).to_u64().unwrap();
    let state = app_handle.state::<AppState>();
    if playback_status == PlayingProgressEnum::Playing {
        emby_http_svc::playing_progress(EmbyPlayingProgressParam {
            emby_server_id: params.emby_server_id.clone(),
            item_id: params.item_id.clone(),
            media_source_id: media_source.id.clone(),
            play_session_id: playback_info.play_session_id.clone(),
            position_ticks: position_ticks,
        }, &state).await?;
        return Ok(());
    }

    let played_duration = chrono::Local::now().timestamp() - start_time;
    if played_duration > 300 {
        emby_server_mapper::update_by_id(EmbyServer {
            id: Some(params.emby_server_id.clone()),
            last_playback_time: Some(chrono::Local::now().fixed_offset()),
            ..Default::default()
        }, &state).await?;
        app_handle.emit("EmbyServerChange", EmbyServerChangeParam {
            id: &params.emby_server_id,
            event: "update",
        })?;
    }
    
    match play_history_mapper::get(params.emby_server_id.clone(), params.item_id.clone(), &state.db_pool).await? {
        Some(response) => {
            play_history_mapper::update_by_id(PlayHistory {
                id: response.id,
                emby_server_name: emby_server.server_name.clone(),
                item_name: Some(params.item_name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                played_duration: Some((played_duration as i32) + response.played_duration.unwrap()),
                ..Default::default()
            }, &state.db_pool).await?;
        },
        None => tracing::error!("播放记录不存在，无法更新播放记录"),
    }

    let progress_percent = if file_duration.is_finite() && *file_duration > 0.0 {
        (last_record_position / Decimal::from_f64(*file_duration).unwrap() * Decimal::from_u64(100).unwrap()).trunc_with_scale(2)
    } else { Decimal::ZERO };
    // trakt 播放停止
    if let Some(mut scrobble_trakt_param) = scrobble_trakt_param.clone() {
        let progress_percent = if progress_percent < Decimal::from_i64(1).unwrap() { Decimal::from_i64(1).unwrap() } else { progress_percent };
        scrobble_trakt_param.progress = progress_percent.to_f64().unwrap();
        match trakt_http_svc::stop(&scrobble_trakt_param, &app_handle.state(), 0).await {
            Ok(json) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    event_type: "TraktNotify".to_string(),
                    message_type: "stop".to_string(),
                    title: None,
                    message: json,
                })?,
            Err(err) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    event_type: "TraktNotify".to_string(),
                    message_type: "error".to_string(),
                    title: None,
                    message: format!("调用trakt停止播放失败: {}", err),
                })?
        }
    }
    // simkl 播放停止
    if let Some(mut scrobble_simkl_param) = scrobble_simkl_param.clone() {
        let progress_percent = if progress_percent < Decimal::from_i64(1).unwrap() { Decimal::from_i64(1).unwrap() } else { progress_percent };
        scrobble_simkl_param.progress = progress_percent.to_f64().unwrap();
        match simkl_http_svc::stop(&scrobble_simkl_param, &app_handle.state(), 0).await {
            Ok(json) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    event_type: "SimklNotify".to_string(),
                    message_type: "stop".to_string(),
                    title: None,
                    message: json,
                })?,
            Err(err) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    event_type: "SimklNotify".to_string(),
                    message_type: "error".to_string(),
                    title: None,
                    message: format!("调用simkl停止播放失败: {}", err),
                })?
        }
    }
    // YamTrack 播放停止
    if let Some(mut scrobble_yamtrack_param) = scrobble_yamtrack_param.clone() {
        scrobble_yamtrack_param.played = progress_percent > Decimal::from_i64(80).unwrap();
        match yamtrack_http_svc::track(&scrobble_yamtrack_param, &app_handle.state()).await {
            Ok(json) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    event_type: "YamTrackNotify".to_string(),
                    message_type: "stop".to_string(),
                    title: None,
                    message: json,
                })?,
            Err(err) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    event_type: "YamTrackNotify".to_string(),
                    message_type: "error".to_string(),
                    title: None,
                    message: format!("调用YamTrack停止播放失败: {}", err),
                })?
        }
    }

    // Emby 播放停止
    let res = emby_http_svc::playing_stopped(EmbyPlayingStoppedParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
        media_source_id: media_source.id.clone(),
        play_session_id: playback_info.play_session_id.clone(),
        position_ticks: position_ticks,
    }, &app_handle.state::<AppState>()).await;
    if res.is_err() {
        tracing::error!("调用Emby停止播放失败: {:?}", res.err());
    }

    // 通知前端播放停止
    app_handle.emit("tauri_notify", TauriNotify {
        event_type: "playingNotify".to_string(),
        message_type: "success".to_string(),
        title: None,
        message: serde_json::to_string(&PlaybackNotifyParam {
            emby_server_id: &params.emby_server_id,
            item_id: &params.item_id,
            item_name: &params.item_name,
            series_id: &episode.series_id,
            series_name: &params.series_name,
            event: "stop",
        })?,
    })?;

    Ok(())
}

/// 后台保存播放进度，不阻塞 mpv IPC 读取循环
fn save_playback_progress_async(playback_process_param: PlaybackProcessParam, last_record_position: Decimal, playback_status: PlayingProgressEnum) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = save_playback_progress(&playback_process_param, last_record_position, playback_status).await {
            tracing::error!("保存播放进度失败: {:?}", e);
        }
    });
}

#[derive(PartialEq)]
enum PlayingProgressEnum {
    Quit,
    Error,
    Next,
    EndOfPlaylist,
    Playing,
}

#[derive(Clone, Serialize)]
struct PlaybackNotifyParam<'a> {
    emby_server_id: &'a str,
    item_id: &'a str,
    item_name: &'a str,
    series_id: &'a Option<String>,
    series_name: &'a str,
    event: &'a str,
}

#[derive(Clone, Serialize)]
struct EmbyServerChangeParam<'a> {
    id: &'a str,
    event: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackTitleParam {
    video: Vec<String>,
    audio: Vec<String>,
    sub: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadNotifyParamEvent {
    title: String,
    url: String,
}

#[derive(Clone)]
struct PlaybackProcessParam {
    params: MediaPlaylistParam,
    episode: Option<EpisodeItem>,
    media_source: MediaSource,
    playback_info: PlaybackInfo,
    app_handle: AppHandle,
    axum_app_state: AxumAppState,
    scrobble_trakt_param: Option<TraktScrobbleParam>,
    scrobble_simkl_param: Option<TraktScrobbleParam>,
    scrobble_yamtrack_param: Option<YamTrackParam>,
    file_duration: f64,
    start_time: i64,
    emby_server: EmbyServer,
    play_proxy_url: Option<String>,
    id: String,
    media_source_select: usize,
    media_source_index: usize,
    sender: Option<mpv_ipc::Sender>,
    play_info_init_finished: bool,
}

#[cfg(not(windows))]
pub(super) async fn get_pipe_rw(pipe_name: &str) -> anyhow::Result<(tokio::io::ReadHalf<interprocess::local_socket::tokio::Stream>, tokio::io::WriteHalf<interprocess::local_socket::tokio::Stream>)> {
    use interprocess::local_socket::{tokio::{prelude::*, Stream}, GenericFilePath};

    let mut retry_count = 0;
    let conn = loop {
        if retry_count >= 10 {
            break None;
        }
        let conn = {
            let name = pipe_name.to_fs_name::<GenericFilePath>();
            if name.is_err() {
                return Err(anyhow::anyhow!("MPV IPC Failed to convert pipe name to fs name"));
            }
            let name = name.unwrap();
            Stream::connect(name.clone()).await
        };
        if conn.is_ok() {
            tracing::debug!("MPV IPC connected");
            break Some(conn.unwrap());
        }
        tracing::debug!("MPV IPC Failed to connect to mpv IPC, retrying...");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        retry_count += 1;
    };
    let conn = match conn {
        Some(conn) => conn,
        None => {
            return Err(anyhow::anyhow!("MPV IPC Failed to connect to mpv IPC"));
        }
    };
    Ok(tokio::io::split(conn))
}

#[cfg(windows)]
pub(super) async fn get_pipe_rw(pipe_name: &str) -> anyhow::Result<(interprocess::os::windows::named_pipe::tokio::RecvPipeStream<interprocess::os::windows::named_pipe::pipe_mode::Bytes>, interprocess::os::windows::named_pipe::tokio::SendPipeStream<interprocess::os::windows::named_pipe::pipe_mode::Bytes>)> {
    use interprocess::os::windows::named_pipe::{pipe_mode, tokio::*};

    let mut retry_count = 0;
    let conn = loop {
        if retry_count >= 10 {
            break None;
        }
        let conn = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_name).await;
        if conn.is_ok() {
            tracing::debug!("MPV IPC connected");
            break Some(conn.unwrap());
        }
        tracing::debug!("MPV IPC Failed to connect to mpv IPC, retrying...");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        retry_count += 1;
    };
    let conn = match conn {
        Some(conn) => conn,
        None => {
            return Err(anyhow::anyhow!("MPV IPC Failed to connect to mpv IPC"));
        }
    };
    Ok(conn.split())
}
