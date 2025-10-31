use std::{cmp::{max, min}, path::PathBuf};

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{config::app_state::{AppState, TauriNotify}, controller::{emby_http_ctl::{EmbyEpisodesParam, EmbyItemsParam, EmbyPlaybackInfoParam}, invoke_ctl::PlayVideoParam}, mapper::{emby_server_mapper::{self, EmbyServer}, global_config_mapper, play_history_mapper::{self, PlayHistory}, proxy_server_mapper}, service::{axum_svc::{AxumAppState, AxumAppStateRequest, MediaPlaylistParam}, emby_http_svc::{self, EmbyGetAudioStreamUrlParam, EmbyGetDirectStreamUrlParam, EmbyGetSubtitleStreamUrlParam, EmbyGetVideoStreamUrlParam, EmbyPageList, EmbyPlayingParam, EmbyPlayingProgressParam, EmbyPlayingStoppedParam, EpisodeItem, MediaSource, PlaybackInfo, SeriesItem}, trakt_http_svc::{self, ScrobbleParam}}, util::{file_util, media_source_util}};

pub async fn play_video(body: PlayVideoParam, state: &tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let emby_server = match emby_server_mapper::get_cache(&body.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err("emby_server not found".to_string()),
    };
    let external_mpv_switch = global_config_mapper::get_cache("external_mpv_switch", state).await.unwrap_or("off".to_string());
    let mpv_path = if external_mpv_switch == "on" {
        match global_config_mapper::get_cache("mpv_path", state).await {
            None => return Err("未配置 mpv 路径".to_string()),
            Some(mpv_path) => {
                let mpv_path = mpv_path.trim().replace("\r", "");
                PathBuf::from(&mpv_path)
            },
        }
    } else {
        match app_handle.path().resolve("resources/mpv/mpv.exe", tauri::path::BaseDirectory::Resource,) {
            Err(err) => return Err(format!("内置 mpv 路径获取失败: {}", err.to_string())),
            Ok(mpv_path) => mpv_path,
        }
    };
    if !mpv_path.is_file() {
        return Err(format!("mpv 路径不存在: {}", mpv_path.display()));
    }
    let mpv_startup_dir = if external_mpv_switch == "on" {
        match global_config_mapper::get_cache("mpv_startup_dir", state).await {
            Some(mpv_startup_dir) => mpv_startup_dir,
            None => mpv_path.parent().unwrap().as_os_str().to_str().unwrap().to_string(),
        }
    } else {
        mpv_path.parent().unwrap().as_os_str().to_str().unwrap().to_string()
    };
    if !PathBuf::from(&mpv_startup_dir).is_dir() {
        return Err(format!("mpv 启动目录不存在: {}", mpv_startup_dir))
    }
    let mpv_args = global_config_mapper::get_cache("mpv_args", state).await.unwrap_or("".to_string());

    let mpv_config_dir = app_handle.path().app_config_dir().unwrap().join("mpv_config");
    if !mpv_config_dir.exists() {
        let res = file_util::mkdir(&mpv_config_dir);
        if res.is_err() {
            return Err(format!("创建 mpv 配置目录失败"));
        }
    }
    let mpv_config_path = mpv_config_dir.join("loemby.mpv.conf");
    file_util::write_file(&mpv_config_path, &mpv_args);

    let auxm_app_state = state.auxm_app_state.clone();
    let app_state = auxm_app_state.read().await.clone().unwrap();

    let episode = match emby_http_svc::items(EmbyItemsParam {
        emby_server_id: body.emby_server_id.clone(),
        item_id: body.item_id.clone(),
    }, &state, true).await {
        Err(e) => return Err(format!("获取剧集信息失败: {}", e)),
        Ok(episode) => serde_json::from_str::<EpisodeItem>(&episode).unwrap(),
    };

    let mut episode_playlist = vec![];
    if let Some(series_id) = episode.series_id.clone() {
        match emby_http_svc::episodes(EmbyEpisodesParam {
            emby_server_id: body.emby_server_id.clone(),
            series_id: series_id,
            season_id: episode.season_id.unwrap(),
            start_item_id: Some(body.item_id.clone()),
            start_index: 0,
            limit: 10,
            extend_fields: Some(false),
        }, &state).await {
            Err(e) => return Err(e.to_string()),
            Ok(episodes) => {
                let episodes = serde_json::from_str::<EmbyPageList<EpisodeItem>>(&episodes).unwrap();
                for episode in episodes.items {
                    episode_playlist.push(episode);
                }
            },
        }
    } else {
        episode_playlist.push(episode);
    }

    #[cfg(windows)]
    let pipe_name = r"\\.\pipe\mpvsocket";
    #[cfg(unix)]
    let pipe_name = r"/tmp/mpvsocket";
    let pipe_name = format!("{}-{}", &pipe_name, uuid::Uuid::new_v4().to_string());

    let mut mpv_playlist = "#EXTM3U".to_string();
    for episode in &episode_playlist {
        let uuid = uuid::Uuid::new_v4().to_string();
        app_state.playlist.write().await.insert(uuid.clone(), MediaPlaylistParam {
            emby_server_id: body.emby_server_id.clone(),
            item_id: episode.id.clone(),
            playback_position_ticks: if episode.id == episode_playlist[0].id { body.playback_position_ticks } else { 0 },
            use_direct_link: body.use_direct_link.clone(),
            select_policy: body.select_policy.clone(),
            video_select: body.video_select,
            audio_select: body.audio_select,
            subtitle_select: body.subtitle_select,
            version_select: body.version_select,
            mpv_ipc: pipe_name.clone(),
        });
        let series_name = episode.series_name.clone().unwrap_or("🎬电影".to_string());
        mpv_playlist = format!("{}\n#EXTINF:-1,{} | {} | {}\nhttp://127.0.0.1:{}/play_media/{}", mpv_playlist, episode.name, series_name, emby_server.server_name.clone().unwrap(), &app_state.port, &uuid);
    }
    let mpv_playlist_path = mpv_config_dir.join("mpv_playlist.m3u8");

    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap());
    command.current_dir(&mpv_startup_dir)
        .arg(&format!("--include={}", mpv_config_path.to_str().unwrap()))
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        // .arg("--force-seekable=yes")  // 某些视频格式在没缓存到的情况下不支持跳转，需要打开此配置，测试后发现强制跳转到没有缓存的位置后，mpv会从头开始缓存，一直缓存到跳转位置，与打开此设置的初衷相违背
        .arg(&format!("--user-agent={}", &emby_server.user_agent.clone().unwrap()))
        .arg(&format!("--playlist={}", mpv_playlist_path.to_str().unwrap()));

    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("调用MPV结果: {:?}", &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    Ok(())
}

pub async fn play_media(axum_app_state: &AxumAppState, id: &str) -> anyhow::Result<String> {
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
    let media_source = if params.select_policy == "manual" {
        if playback_info.media_sources.len() >= params.version_select as usize {
            &playback_info.media_sources[params.version_select as usize - 1]
        } else {
            &playback_info.media_sources[0]
        }
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
        &playback_info.media_sources[version_select_list[0].version_id as usize - 1]
    };
    // 向mpv添加音频字幕
    let (_recver, mut sender) = get_pipe_rw(&params.mpv_ipc).await?;
    let support_direct_link = media_source.is_remote == Some(true) && media_source.path.is_some() && media_source.path.as_ref().unwrap().contains("://") && !media_source_util::is_internal_url(&media_source.path.as_ref().unwrap());
    let mut video_url = if params.use_direct_link && support_direct_link {
        media_source.path.clone().unwrap()
    } else if media_source.direct_stream_url.is_some() {
        emby_http_svc::get_direct_stream_url(EmbyGetDirectStreamUrlParam {
            emby_server_id: params.emby_server_id.clone(),
            direct_stream_url: media_source.direct_stream_url.clone().unwrap(),
        }, &app_state).await?
    } else {
        emby_http_svc::get_video_stream_url(EmbyGetVideoStreamUrlParam {
            emby_server_id: params.emby_server_id.clone(),
            item_id: params.item_id.clone(),
            container: media_source.container.clone(),
            media_source_id: media_source.id.clone(),
            play_session_id: playback_info.play_session_id.clone(),
        }, &app_state).await?
    };
    if play_proxy_url.is_some() {
        let uuid = uuid::Uuid::new_v4().to_string();
        axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
            stream_url: video_url,
            proxy_url: play_proxy_url.clone(),
            user_agent: emby_server.user_agent.as_ref().unwrap().clone(),
        });
        video_url = format!("http://127.0.0.1:{}/stream/video/{}", &axum_app_state.port, &uuid);
    }

    for media_stream in &media_source.media_streams {
        if media_stream.is_external != Some(true) {
            continue;
        }
        if media_stream.type_ == "Audio" {
            let mut audio_url = emby_http_svc::get_audio_stream_url(EmbyGetAudioStreamUrlParam {
                emby_server_id: params.emby_server_id.clone(),
                item_id: params.item_id.clone(),
                media_source_item_id: media_source.item_id.clone(),
                media_streams_codec: media_stream.codec.clone(),
                media_streams_index: media_stream.index,
                media_streams_is_external: true,
            }, &app_state).await?;
            if play_proxy_url.is_some() {
                let uuid = uuid::Uuid::new_v4().to_string();
                axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                    stream_url: audio_url,
                    proxy_url: play_proxy_url.clone(),
                    user_agent: emby_server.user_agent.as_ref().unwrap().clone(),
                });
                audio_url = format!("http://127.0.0.1:{}/stream/audio/{}", &axum_app_state.port, &uuid);
            }
            let audio_title = format!("{} / {}", media_stream.display_title, media_stream.display_language.clone().unwrap_or_default());
            let command = format!(r#"{{ "command": ["audio-add", "{}", "auto", "{}"] }}{}"#, audio_url, audio_title, "\n");
            sender.write_all(command.as_bytes()).await?;
            tracing::debug!("MPV IPC Command audio-add: {}", command);
        } else if media_stream.type_ == "Subtitle" {
            let mut subtitle_url = emby_http_svc::get_subtitle_stream_url(EmbyGetSubtitleStreamUrlParam {
                emby_server_id: params.emby_server_id.clone(),
                item_id: params.item_id.clone(),
                media_source_id: media_source.id.clone(),
                media_source_item_id: media_source.item_id.clone(),
                media_streams_codec: media_stream.codec.clone(),
                media_streams_index: media_stream.index,
                media_streams_is_external: true,
            }, &app_state).await?;
            if play_proxy_url.is_some() {
                let uuid = uuid::Uuid::new_v4().to_string();
                axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                    stream_url: subtitle_url,
                    proxy_url: play_proxy_url.clone(),
                    user_agent: emby_server.user_agent.as_ref().unwrap().clone(),
                });
                subtitle_url = format!("http://127.0.0.1:{}/stream/subtitle/{}", &axum_app_state.port, &uuid);
            }
            let subtitle_title = format!("{} / {}", media_stream.display_title, media_stream.display_language.clone().unwrap_or_default());
            let command = format!(r#"{{ "command": ["sub-add", "{}", "auto", "{}"] }}{}"#, subtitle_url, subtitle_title, "\n");
            sender.write_all(command.as_bytes()).await?;
            tracing::debug!("MPV IPC Command sub-add: {}", command);
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
            track_titles.video.push(media_source.name.clone() + " / " + &media_stream.display_title);
            video_index += 1;
            if media_stream.is_default == Some(true) {
                vid = video_index;
            }
        } else if media_stream.type_ == "Audio" {
            track_titles.audio.push(media_stream.display_title.clone());
            audio_index += 1;
            if media_stream.is_default == Some(true) && aid == 0 {
                aid = audio_index;
            }
        } else if media_stream.type_ == "Subtitle" {
            track_titles.sub.push(media_stream.display_title.clone());
            subtitle_index += 1;
            let mut score = 0;
            if media_stream.is_default == Some(true) {
                score += 1;
            }
            if media_stream.is_external == Some(true) {
                score += 2;
            }
            if let Some(lang) = &media_stream.display_language {
                if lang.contains("Chinese Simplified") {
                    score += 3;
                }
            }
            if score > subtitle_max_score {
                subtitle_max_score = score;
                sid = subtitle_index;
            }
        }
    }
    let track_titles = serde_json::to_string(&track_titles)?.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
    let set_track_titles_command = format!(r#"{{ "command": ["script-message-to", "uosc", "set-track-title", "{}"] }}{}"#, track_titles, "\n");
    sender.write_all(set_track_titles_command.as_bytes()).await?;
    tracing::debug!("MPV IPC Command set_track_titles: {}", set_track_titles_command);

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
    let command = if vid == -1 {
        format!(r#"{{ "command": ["vid", "no"] }}{}"#, "\n")
    } else if vid == 0 {
        format!(r#"{{ "command": ["vid", "auto"] }}{}"#, "\n")
    } else {
        format!(r#"{{ "command": ["vid", "{}"] }}{}"#, vid, "\n")
    };
    sender.write_all(command.as_bytes()).await?;
    let command = if aid == -1 {
        format!(r#"{{ "command": ["aid", "no"] }}{}"#, "\n")
    } else if aid == 0 {
        format!(r#"{{ "command": ["aid", "auto"] }}{}"#, "\n")
    } else {
        format!(r#"{{ "command": ["aid", "{}"] }}{}"#, aid, "\n")
    };
    sender.write_all(command.as_bytes()).await?;
    let command = if sid == -1 {
        format!(r#"{{ "command": ["sid", "no"] }}{}"#, "\n")
    } else if sid == 0 {
        format!(r#"{{ "command": ["sid", "auto"] }}{}"#, "\n")
    } else {
        format!(r#"{{ "command": ["sid", "{}"] }}{}"#, sid, "\n")
    };
    sender.write_all(command.as_bytes()).await?;

    let mut cache_max_bytes = 300 * 1024 * 1024;
    let mut cache_back_max_bytes = 200 * 1024 * 1024;
    if media_source.bitrate.is_some() && media_source.bitrate.unwrap() > 0 {
        let mpv_cache_seconds = global_config_mapper::get_cache("mpv_cache_seconds", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(300);
        let mpv_cache_min_bytes = global_config_mapper::get_cache("mpv_cache_min_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(cache_max_bytes);
        let mpv_cache_max_bytes = global_config_mapper::get_cache("mpv_cache_max_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(cache_max_bytes);
        let mpv_cache_back_seconds = global_config_mapper::get_cache("mpv_cache_back_seconds", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(200);
        let mpv_cache_back_min_bytes = global_config_mapper::get_cache("mpv_cache_back_min_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(cache_back_max_bytes);
        let mpv_cache_back_max_bytes = global_config_mapper::get_cache("mpv_cache_back_max_bytes", &app_state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(cache_back_max_bytes);
        let mpv_cache_max_bytes = max(min(mpv_cache_seconds * media_source.bitrate.unwrap() / 8, mpv_cache_max_bytes * 1024 * 1024), mpv_cache_min_bytes * 1024 * 1024);
        let mpv_cache_back_max_bytes = max(min(mpv_cache_back_seconds * media_source.bitrate.unwrap() / 8, mpv_cache_back_max_bytes * 1024 * 1024), mpv_cache_back_min_bytes * 1024 * 1024);

        if mpv_cache_max_bytes > 0 {
            cache_max_bytes = mpv_cache_max_bytes;
        }
        if mpv_cache_back_max_bytes > 0 {
            cache_back_max_bytes = mpv_cache_back_max_bytes;
        }
    }
    let command = format!(r#"{{ "command": ["demuxer-max-bytes", "{}"] }}{}"#, cache_max_bytes, "\n");
    sender.write_all(command.as_bytes()).await?;
    let command = format!(r#"{{ "command": ["demuxer-max-back-bytes", "{}"] }}{}"#, cache_back_max_bytes, "\n");
    sender.write_all(command.as_bytes()).await?;

    // 本地播放历史记录
    let episode = match emby_http_svc::items(EmbyItemsParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
    }, &app_state, true).await {
        Err(e) => return Err(anyhow::anyhow!("获取剧集信息失败: {}", e.to_string())),
        Ok(episode) => serde_json::from_str::<EpisodeItem>(&episode).unwrap(),
    };
    let mut pinned = 0;
    if let Some(series_id) = episode.series_id.clone() {
        let pinned_update = play_history_mapper::cancel_pinned(params.emby_server_id.clone(), series_id, &app_state.db_pool).await.unwrap();
        if pinned_update.rows_affected() > 0 { pinned = 1 }
    }
    match play_history_mapper::get(params.emby_server_id.clone(), params.item_id.clone(), &app_state.db_pool).await.unwrap() {
        Some(response) => {
            if episode.series_id.is_none() {
                pinned = response.pinned.unwrap();
            }
            play_history_mapper::update_by_id(PlayHistory {
                id: response.id,
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_name: emby_server.server_name.clone(),
                item_name: Some(episode.name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                pinned: Some(pinned),
                ..Default::default()
            }, &app_state.db_pool).await.unwrap();
        },
        None => {
            play_history_mapper::create(PlayHistory {
                id: Some(uuid::Uuid::new_v4().to_string()),
                create_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_id: emby_server.id.clone(),
                emby_server_name: emby_server.server_name.clone(),
                item_id: Some(episode.id.clone()),
                item_name: Some(episode.name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                played_duration: Some(0),
                pinned: Some(pinned),
            }, &app_state.db_pool).await.unwrap();
        },
    }

    // emby开始播放api
    let res = emby_http_svc::playing(EmbyPlayingParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
        media_source_id: media_source.id.clone(),
        play_session_id: playback_info.play_session_id.clone(),
        position_ticks: params.playback_position_ticks,
    }, &app_state).await;
    if res.is_err() {
        axum_app_state.app.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessage".to_string(),
            message_type: "error".to_string(),
            title: None,
            message: format!("调用emby播放进度失败: {}", res.unwrap_err()),
        }).unwrap()
    }

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
    let scrobble_trakt_param = if trakt_sync_switch != Some("off".to_string()) && trakt_username.is_some() {
        let progress = if let Some(run_time_ticks) = media_source.run_time_ticks {
            (params.playback_position_ticks as f64 / (run_time_ticks as f64 / 100.0)).round() / 100.0
        } else { 0.0 };
        let trakt_scrobble_param = trakt_http_svc::get_scrobble_trakt_param(&episode, &series, progress);
        if let Some(scrobble_trakt_param) = &trakt_scrobble_param {
            match trakt_http_svc::start(scrobble_trakt_param, &app_state, 0).await {
                Ok(json) => 
                    axum_app_state.app.emit("tauri_notify", TauriNotify {
                        alert_type: "TraktStart".to_string(),
                        message_type: "success".to_string(),
                        title: None,
                        message: json,
                    }).unwrap(),
                Err(err) => 
                    axum_app_state.app.emit("tauri_notify", TauriNotify {
                        alert_type: "ElMessage".to_string(),
                        message_type: "error".to_string(),
                        title: None,
                        message: format!("调用trakt开始播放失败: {}", err),
                    }).unwrap()
            }
        }
        trakt_scrobble_param
    } else { None };

    // 播放进度
    let playback_progress_param = PlaybackProgressParam {
        params: params.clone(),
        episode: episode,
        media_source: media_source.clone(),
        playback_info: playback_info,
        app_handle: axum_app_state.app.clone(),
        scrobble_trakt_param,
        start_time: chrono::Local::now().timestamp(),
        emby_server: emby_server,
    };
    tauri::async_runtime::spawn(async move {
        let res = playback_progress(playback_progress_param).await;
        if res.is_err() {
            tracing::error!("播放进度失败: {:?}", res.unwrap_err());
        }
    });

    Ok(video_url)
}

async fn playback_progress(playback_progress_param: PlaybackProgressParam) -> anyhow::Result<()> {
    let (recver, mut sender) = get_pipe_rw(&playback_progress_param.params.mpv_ipc).await?;
    let mut recver = BufReader::new(recver);

    // 观测播放进度，返回太频繁，改为每2秒获取一次，用户跳转时立即获取一次
    // let observe_property_progress_command = r#"{ "command": ["observe_property", 10023, "playback-time"]}"#.to_string() + "\n";
    // let write = sender.write_all(observe_property_progress_command.as_bytes()).await;
    // if write.is_err() {
    //     tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
    // }

    let run_time_ticks = playback_progress_param.media_source.run_time_ticks;
    let send_task = tokio::spawn(async move {
        let command = if run_time_ticks.is_none() || run_time_ticks == Some(0) {
            r#"{ "command": ["get_property", "percent-pos"], "request_id": 10022 }"#.to_string() + "\n"
        } else {
            r#"{ "command": ["get_property", "playback-time"], "request_id": 10023 }"#.to_string() + "\n"
        };
        loop {
            let write = sender.write_all(command.as_bytes()).await;
            if write.is_err() {
                tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let mut last_save_time = chrono::Local::now();
    let mut last_record_position = Decimal::from_u64(playback_progress_param.params.playback_position_ticks / 1000_0000).unwrap();
    let mut start_recording = false;
    loop {
        let mut buffer = String::new();
        let read = recver.read_line(&mut buffer).await;
        if read.is_err() {
            tracing::error!("MPV IPC Failed to read pipe {:?}", read);
            send_task.abort();
            save_playback_progress(&playback_progress_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        tracing::debug!("MPV IPC Server answered: {}", buffer.trim());
        if buffer.trim().is_empty() {
            send_task.abort();
            save_playback_progress(&playback_progress_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            tracing::error!("mpv-ipc 响应为空，连接已断开");
            break;
        }
        let json = serde_json::from_str::<MpvIpcResponse>(&buffer);
        if json.is_err() {
            tracing::error!("解析 mpv-ipc 响应失败 {:?}", json);
            send_task.abort();
            save_playback_progress(&playback_progress_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        let json = json.unwrap();
        if let Some("end-file") = json.event {
            tracing::debug!("MPV IPC 播放结束");
            send_task.abort();
            save_playback_progress(&playback_progress_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        if let Some("seek") = json.event {
            continue;
        }
        if let Some("file-loaded") = json.event {
            start_recording = true;
        }
        if !start_recording {
            continue;
        }
        if let Some(10022) = json.request_id {
            let progress_percent = json.data;
            if let Some(progress_percent) = progress_percent {
                tracing::debug!("MPV IPC 播放进度百分比 {}", progress_percent);
                last_record_position = Decimal::from_f64(progress_percent).unwrap().round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&playback_progress_param, last_record_position, PlayingProgressEnum::Playing).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
                }
            }
        }
        if let Some(10023) = json.request_id {
            let progress = json.data;
            if let Some(progress) = progress {
                tracing::debug!("MPV IPC 播放进度 {}", progress);
                last_record_position = Decimal::from_f64(progress).unwrap().round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&playback_progress_param, last_record_position, PlayingProgressEnum::Playing).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
                }
            }
        }
    }
    anyhow::Ok(())
}

async fn save_playback_progress(playback_progress_param: &PlaybackProgressParam, last_record_position: Decimal, playback_status: PlayingProgressEnum) -> anyhow::Result<()> {
    let position_ticks = (last_record_position * Decimal::from_i64(1000_0000).unwrap()).to_u64().unwrap();
    let state = playback_progress_param.app_handle.state::<AppState>();
    if playback_status == PlayingProgressEnum::Playing {
        emby_http_svc::playing_progress(EmbyPlayingProgressParam {
            emby_server_id: playback_progress_param.params.emby_server_id.clone(),
            item_id: playback_progress_param.params.item_id.clone(),
            media_source_id: playback_progress_param.media_source.id.clone(),
            play_session_id: playback_progress_param.playback_info.play_session_id.clone(),
            position_ticks: position_ticks,
        }, &state).await?;
        return Ok(());
    }

    let played_duration = chrono::Local::now().timestamp() - playback_progress_param.start_time;
    if played_duration > 300 {
        emby_server_mapper::update_by_id(EmbyServer {
            id: Some(playback_progress_param.params.emby_server_id.clone()),
            last_playback_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            ..Default::default()
        }, &state).await?;
        playback_progress_param.app_handle.emit("EmbyServerChange", EmbyServerChangeParam {
            id: &playback_progress_param.params.emby_server_id,
            event: "update",
        })?;
    } else {
        playback_progress_param.app_handle.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessage".to_string(),
            message_type: "warning".to_string(),
            title: None,
            message: format!("播放时间不足 5 分钟，不更新最后播放时间"),
        }).unwrap()
    }
    
    match play_history_mapper::get(playback_progress_param.params.emby_server_id.clone(), playback_progress_param.params.item_id.clone(), &state.db_pool).await? {
        Some(response) => {
            play_history_mapper::update_by_id(PlayHistory {
                id: response.id,
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_name: playback_progress_param.emby_server.server_name.clone(),
                item_name: Some(playback_progress_param.episode.name.clone()),
                item_type: Some(playback_progress_param.episode.type_.clone()),
                series_id: playback_progress_param.episode.series_id.clone(),
                series_name: playback_progress_param.episode.series_name.clone(),
                played_duration: Some(played_duration + response.played_duration.unwrap()),
                ..Default::default()
            }, &state.db_pool).await?;
        },
        None => tracing::error!("播放记录不存在，无法更新播放记录"),
    }

    let window = playback_progress_param.app_handle.webview_windows();
    let window = window.values().next().expect("Sorry, no window found");
    window.unminimize().expect("Sorry, no window unminimize");
    window.show().expect("Sorry, no window show");
    window.set_focus().expect("Can't Bring Window to Focus");
    
    let progress_percent = if let Some(run_time_ticks) = playback_progress_param.media_source.run_time_ticks {
        (last_record_position * Decimal::from_i64(1000_0000).unwrap() / Decimal::from_u64(run_time_ticks).unwrap() * Decimal::from_u64(100).unwrap()).trunc_with_scale(2)
    } else {
        last_record_position
    };
    if let Some(mut scrobble_trakt_param) = playback_progress_param.scrobble_trakt_param.clone() {
        let progress_percent = if progress_percent < Decimal::from_i64(1).unwrap() { Decimal::from_i64(1).unwrap() } else { progress_percent };
        scrobble_trakt_param.progress = progress_percent.to_f64().unwrap();
        match trakt_http_svc::stop(&scrobble_trakt_param, &playback_progress_param.app_handle.state(), 0).await {
            Ok(json) => 
                playback_progress_param.app_handle.emit("tauri_notify", TauriNotify {
                    alert_type: "TraktStop".to_string(),
                    message_type: "success".to_string(),
                    title: None,
                    message: json,
                }).unwrap(),
            Err(err) => 
                playback_progress_param.app_handle.emit("tauri_notify", TauriNotify {
                    alert_type: "ElMessage".to_string(),
                    message_type: "error".to_string(),
                    title: None,
                    message: format!("调用trakt停止播放失败: {}", err),
                }).unwrap()
        }
    }

    emby_http_svc::playing_stopped(EmbyPlayingStoppedParam {
        emby_server_id: playback_progress_param.params.emby_server_id.clone(),
        item_id: playback_progress_param.params.item_id.clone(),
        media_source_id: playback_progress_param.media_source.id.clone(),
        play_session_id: playback_progress_param.playback_info.play_session_id.clone(),
        position_ticks: position_ticks,
    }, &playback_progress_param.app_handle.state::<AppState>()).await?;

    playback_progress_param.app_handle.emit("playingStopped", PlaybackStoppedParam {
        emby_server_id: &playback_progress_param.params.emby_server_id,
        item_id: &playback_progress_param.params.item_id,
        progress_percent: &progress_percent,
    })?;

    Ok(())
}

#[derive(PartialEq)]
enum PlayingProgressEnum {
    Stop,
    Playing,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct MpvIpcResponse<'a> {
    event: Option<&'a str>,    // 事件 end-file | audio-reconfig | video-reconfig | playback-restart | client-message | seek | file-loaded
    data: Option<f64>,    // 获取播放进度时，返回秒
    request_id: Option<u32>,    // 请求ID，可以自定义传入，响应时会返回该ID
    reason: Option<&'a str>,    // quit | eof | error
    error: Option<&'a str>,     // success | property unavailable
    file_error: Option<&'a str>,    // 错误原因 loading failed
}

#[derive(Clone, Serialize)]
struct PlaybackStoppedParam<'a> {
    emby_server_id: &'a str,
    item_id: &'a str,
    progress_percent: &'a Decimal,
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

#[derive(Debug)]
struct PlaybackProgressParam {
    params: MediaPlaylistParam,
    episode: EpisodeItem,
    media_source: MediaSource,
    playback_info: PlaybackInfo,
    app_handle: AppHandle,
    scrobble_trakt_param: Option<ScrobbleParam>,
    start_time: i64,
    emby_server: EmbyServer,
}

#[cfg(not(windows))]
async fn get_pipe_rw(pipe_name: &str) -> anyhow::Result<(interprocess::local_socket::tokio::RecvHalf, interprocess::local_socket::tokio::SendHalf)> {
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
    Ok(conn.split())
}

#[cfg(windows)]
async fn get_pipe_rw(pipe_name: &str) -> anyhow::Result<(interprocess::os::windows::named_pipe::tokio::RecvPipeStream<interprocess::os::windows::named_pipe::pipe_mode::Bytes>, interprocess::os::windows::named_pipe::tokio::SendPipeStream<interprocess::os::windows::named_pipe::pipe_mode::Bytes>)> {
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
