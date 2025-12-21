use std::{cmp::{max, min}, path::PathBuf, sync::Arc};

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, sync::RwLock};
use tokio_stream::StreamExt;

use crate::{config::{app_state::{AppState, TauriNotify}, http_pool}, controller::{emby_http_ctl::{EmbyEpisodesParam, EmbyItemsParam, EmbyPlaybackInfoParam}, invoke_ctl::PlayVideoParam}, mapper::{emby_server_mapper::{self, EmbyServer}, global_config_mapper::{self, GlobalConfig}, play_history_mapper::{self, PlayHistory}, proxy_server_mapper}, service::{axum_svc::{AxumAppState, AxumAppStateRequest, MediaPlaylistParam}, emby_http_svc::{self, EmbyGetAudioStreamUrlParam, EmbyGetDirectStreamUrlParam, EmbyGetSubtitleStreamUrlParam, EmbyGetVideoStreamUrlParam, EmbyPageList, EmbyPlayingParam, EmbyPlayingProgressParam, EmbyPlayingStoppedParam, EpisodeItem, MediaSource, PlaybackInfo, SeriesItem}, trakt_http_svc::{self, ScrobbleParam}}, util::{file_util, media_source_util}};

pub async fn play_video(body: PlayVideoParam, state: &tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let emby_server = match emby_server_mapper::get_cache(&body.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err("emby_server not found".to_string()),
    };
    let external_mpv_switch = global_config_mapper::get_cache("external_mpv_switch", state).await.unwrap_or("off".to_string());
    let (mpv_path, mpv_config_dir, mpv_startup_dir) = if external_mpv_switch == "on" {
        match global_config_mapper::get_cache("mpv_path", state).await {
            None => return Err("未配置 mpv 路径".to_string()),
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
                    return Err(format!("所有的外部 mpv 路径都不存在"));
                }
                (final_mpv_path.unwrap(), final_mpv_config_dir.clone().unwrap(), final_mpv_config_dir.unwrap().parent().unwrap().to_path_buf())
            },
        }
    } else {
        #[cfg(target_os = "windows")]
        match app_handle.path().resolve("resources/mpv/mpv.exe", tauri::path::BaseDirectory::Resource,) {
            Err(err) => return Err(format!("内置 mpv 路径获取失败: {}", err.to_string())),
            Ok(mpv_path) => (mpv_path.clone(), mpv_path.parent().unwrap().join("portable_config"), mpv_path.parent().unwrap().to_path_buf()),
        }
        #[cfg(target_os = "linux")]
        match app_handle.path().resolve("resources/mpv/mpv.AppImage", tauri::path::BaseDirectory::Resource,) {
            Err(err) => return Err(format!("内置 mpv 路径获取失败: {}", err.to_string())),
            Ok(mpv_path) => (mpv_path.clone(), mpv_path.parent().unwrap().join("portable_config"), mpv_path.parent().unwrap().to_path_buf()),
        }
        #[cfg(target_os = "macos")]
        match app_handle.path().resolve("resources/mpv/portable_config", tauri::path::BaseDirectory::Resource,) {
            Err(err) => return Err(format!("内置 mpv 配置目录获取失败: {}", err.to_string())),
            Ok(mpv_config_path) => match PathBuf::from("/usr/bin/mpv").is_file() {
                true => (PathBuf::from("/usr/bin/mpv"), mpv_config_path.clone(), mpv_config_path.parent().unwrap().to_path_buf()),
                false => return Err(format!("mpv 未安装，请先安装 mpv ，检测路径 /usr/bin/mpv")),
            },
        }
    };
    let mpv_args = global_config_mapper::get_cache("mpv_args", state).await.unwrap_or("".to_string());

    if !mpv_config_dir.exists() {
        let res = file_util::mkdir(&mpv_config_dir);
        if res.is_err() {
            return Err(format!("创建 mpv 配置目录失败"));
        }
    }
    let mpv_config_path = mpv_config_dir.join("loemby.mpv.conf");
    file_util::write_file(&mpv_config_path, &mpv_args);

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
            limit: 30,
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
    let mut playlist_start = 0;
    for (i, episode) in episode_playlist.iter().enumerate() {
        if episode.id == body.item_id {
            playlist_start = i;
            break;
        }
    }

    #[cfg(windows)]
    let pipe_name = r"\\.\pipe\mpvsocket";
    #[cfg(unix)]
    let pipe_name = r"/tmp/mpvsocket";
    let pipe_name = format!("{}-{}", &pipe_name, uuid::Uuid::new_v4().to_string());

    let auxm_app_state = state.auxm_app_state.read().await.clone().unwrap();

    let mut mpv_playlist = "#EXTM3U".to_string();
    for (_i, episode) in episode_playlist.iter().enumerate() {
        let uuid = uuid::Uuid::new_v4().to_string();
        let series_name = episode.series_name.clone().unwrap_or("🎬电影".to_string());
        let parent_index_number = episode.parent_index_number.map_or("_".to_string(), |n| n.to_string());
        let index_number = episode.index_number.map_or("_".to_string(), |n| n.to_string());
        let prefix = if episode.series_id.is_some() { format!("S{}E{}. ", parent_index_number, index_number) } else { "".to_string() };
        let episode_name = format!("{}{}", prefix, episode.name);
        let title = format!("{} | {} | {}", episode_name, series_name, emby_server.server_name.as_ref().unwrap());
        auxm_app_state.playlist.write().await.insert(uuid.clone(), MediaPlaylistParam {
            emby_server_id: body.emby_server_id.clone(),
            series_id: body.series_id.clone(),
            series_name: series_name.clone(),
            item_id: episode.id.clone(),
            item_name: episode_name.clone(),
            playback_position_ticks: if episode.id == episode_playlist[0].id { body.playback_position_ticks } else { 0 },
            use_direct_link: body.use_direct_link.clone(),
            select_policy: body.select_policy.clone(),
            video_select: body.video_select,
            audio_select: body.audio_select,
            subtitle_select: body.subtitle_select,
            mpv_ipc: pipe_name.clone(),
            media_title: title.clone(),
        });
        let media_source_select = if body.select_policy == "manual" { body.version_select } else { 0 };
        mpv_playlist = format!("{}\n#EXTINF:-1,{}\nhttp://127.0.0.1:{}/play_media/{}/{}", mpv_playlist, title, &auxm_app_state.port, &uuid, media_source_select);
    }
    let mpv_playlist_path = mpv_config_dir.join("mpv_playlist.m3u8");
    file_util::write_file(&mpv_playlist_path, &mpv_playlist);
    
    let mpv_volume = global_config_mapper::get_cache("mpv_volume", state).await.unwrap_or("100".to_string());

    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap().replace(r"\\?\", ""));
    command.current_dir(&mpv_startup_dir)
        .arg(&format!("--include={}", mpv_config_path.as_os_str().to_str().unwrap().replace(r"\\?\", "")))
        .arg(&format!("--input-ipc-server={}", &pipe_name))
        .arg(&format!("--config-dir={}", mpv_config_dir.as_os_str().to_str().unwrap().replace(r"\\?\", "")))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        .arg("--autoload-files=no")  // 不自动加载外部文件
        // .arg("--force-seekable=yes")  // 某些视频格式在没缓存到的情况下不支持跳转，需要打开此配置，测试后发现强制跳转到没有缓存的位置后，mpv会从头开始缓存，一直缓存到跳转位置，与打开此设置的初衷相违背
        .arg(&format!("--user-agent={}", emby_server.user_agent.as_ref().unwrap()))
        .arg(&format!("--volume={}", &mpv_volume))
        .arg(&format!("--playlist={}", mpv_playlist_path.as_os_str().to_str().unwrap().replace(r"\\?\", "")))
        .arg(&format!("--playlist-start={}", playlist_start));

    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("调用MPV结果: {:?}", &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    Ok(())
}

pub async fn play_media(axum_app_state: &AxumAppState, id: &str, media_source_select: usize) -> anyhow::Result<String> {
    let playlist = axum_app_state.playlist.read().await.clone();
    let params = playlist.get(id).ok_or(anyhow::anyhow!("媒体ID不存在"))?;

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
    };
    let media_source = &playback_info.media_sources[media_source_index];
    // 选中媒体源的视频地址
    let support_direct_link = media_source.is_remote == Some(true) && media_source.path.is_some() && media_source.path.as_ref().ok_or(anyhow::anyhow!("路径解析错误"))?.contains("://") && !media_source_util::is_internal_url(&media_source.path.as_ref().unwrap());
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

    // 播放进程
    let playback_process_param = PlaybackProcessParam {
        params: params.clone(),
        episode: None,
        media_source: media_source.clone(),
        playback_info: playback_info,
        app_handle: axum_app_state.app.clone(),
        axum_app_state: axum_app_state.clone(),
        scrobble_trakt_param: None,
        start_time: chrono::Local::now().timestamp(),
        emby_server: emby_server,
        play_proxy_url,
        id: id.to_string(),
        media_source_select,
        media_source_index,
        sender: None,
    };
    tauri::async_runtime::spawn(async move {
        let res = playback_process(playback_process_param).await;
        if res.is_err() {
            tracing::error!("播放进程失败: {:?}", res.unwrap_err());
        }
    });

    Ok(video_url)
}

async fn playback_process(mut playback_process_param: PlaybackProcessParam) -> anyhow::Result<()> {
    let PlaybackProcessParam {
        ref params,
        episode: _,
        ref media_source,
        playback_info: _,
        ref app_handle,
        axum_app_state: _,
        scrobble_trakt_param: _,
        start_time: _,
        ref emby_server,
        ref play_proxy_url,
        id: _,
        media_source_select: _,
        media_source_index: _,
        sender: _,
    } = playback_process_param;
    let app_state = app_handle.state::<AppState>();

    // 缓存字幕，作用不止于此，播放前和播放后，获取到的媒体元数据信息不一致（主要是字幕的索引位置变化），导致字幕无法正常显示，所以这里缓存字幕
    for media_stream in &media_source.media_streams {
        if media_stream.type_ == "Subtitle" && media_stream.is_external == Some(true) {
            let subtitle_url = emby_http_svc::get_subtitle_stream_url(EmbyGetSubtitleStreamUrlParam {
                emby_server_id: params.emby_server_id.clone(),
                item_id: params.item_id.clone(),
                media_source_id: media_source.id.clone(),
                media_source_item_id: media_source.item_id.clone(),
                media_streams_codec: media_stream.codec.clone(),
                media_streams_index: media_stream.index,
                media_streams_is_external: true,
            }, &app_state).await?;
            let cache_digest = sha256::digest(subtitle_url.clone());
            let cache_file_path = app_handle.path().resolve(&format!("cache/subtitle/{}.ass", cache_digest), tauri::path::BaseDirectory::AppLocalData)?;
            if cache_file_path.exists() {
                tracing::debug!("cache subtitle file exists: {}", cache_file_path.display());
                continue;
            }
            let cache_file_tmp_path = app_handle.path().resolve(&format!("cache/subtitle/{}.ass.tmp", cache_digest), tauri::path::BaseDirectory::AppLocalData)?;
            let (play_proxy_url, app_handle, user_agent) = (play_proxy_url.clone(), app_handle.clone(), emby_server.user_agent.as_ref().unwrap().clone());
            tokio::spawn(async move {
                async fn download_subtitle_file(subtitle_url: String, cache_file_path: PathBuf, play_proxy_url: Option<String>, user_agent: String, app_state: tauri::State<'_, AppState>, cache_file_tmp_path: PathBuf) -> anyhow::Result<()> {
                    let client = http_pool::get_image_http_client(play_proxy_url.clone(), &app_state).await?;
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_str(&user_agent)?);
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
                match download_subtitle_file(subtitle_url, cache_file_path, play_proxy_url, user_agent, app_state, cache_file_tmp_path).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("cache subtitle file failed: {}", e);
                    }
                }
            });
        }
    }

    let (recver, mut sender) = get_pipe_rw(&params.mpv_ipc).await?;
    let mut recver = BufReader::new(recver);

    // 缓存大小
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
    let command = format!(r#"{{ "command": ["set_property", "demuxer-max-bytes", "{}"] }}{}"#, cache_max_bytes, "\n");
    sender.write_all(command.as_bytes()).await?;
    sender.flush().await?;
    let command = format!(r#"{{ "command": ["set_property", "demuxer-max-back-bytes", "{}"] }}{}"#, cache_back_max_bytes, "\n");
    sender.write_all(command.as_bytes()).await?;
    sender.flush().await?;

    // 本地播放历史记录
    let episode = match emby_http_svc::items(EmbyItemsParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
    }, &app_state, true).await {
        Err(e) => return Err(anyhow::anyhow!("获取剧集信息失败: {}", e.to_string())),
        Ok(episode) => serde_json::from_str::<EpisodeItem>(&episode)?,
    };
    playback_process_param.episode = Some(episode);
    let episode = playback_process_param.episode.as_ref().unwrap();
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
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
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
                create_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_id: emby_server.id.clone(),
                emby_server_name: emby_server.server_name.clone(),
                item_id: Some(episode.id.clone()),
                item_name: Some(params.item_name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                played_duration: Some(0),
                pinned: Some(pinned),
            }, &app_state.db_pool).await?;
        },
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
    playback_process_param.scrobble_trakt_param = scrobble_trakt_param;

    // 观测音量
    let observe_property_volume_command = r#"{ "command": ["observe_property", 10001, "volume"]}"#.to_string() + "\n";
    let write = sender.write_all(observe_property_volume_command.as_bytes()).await;
    if write.is_err() {
        tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
    }
    sender.flush().await?;

    playback_process_param.sender = Some(Arc::new(RwLock::new(sender)));
    
    let mut play_info_init_finished = false;
    let mut send_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut last_save_time = chrono::Local::now();
    let mut last_record_position = Decimal::from_u64(params.playback_position_ticks / 1000_0000).unwrap();
    loop {
        let mut buffer = String::new();
        let read = recver.read_line(&mut buffer).await;
        if read.is_err() {
            tracing::error!("MPV IPC Failed to read pipe {:?}", read);
            if let Some(send_task) = send_task { send_task.abort(); }
            save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        tracing::debug!("MPV IPC Received: {}", buffer.trim());
        if buffer.trim().is_empty() {
            if let Some(send_task) = send_task { send_task.abort(); }
            save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            tracing::error!("mpv-ipc 响应为空，连接已断开");
            break;
        }
        let json = serde_json::from_str::<MpvIpcResponse>(&buffer);
        if json.is_err() {
            tracing::error!("解析 mpv-ipc 响应失败 {:?}", json);
            if let Some(send_task) = send_task { send_task.abort(); }
            save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        let json = json?;
        if let Some(10001) = json.id {
            if let Some(volume) = json.data {
                tracing::debug!("MPV IPC 音量观测变更 {}", volume);
                let volume = volume.as_f64().ok_or(anyhow::anyhow!("音量不是数字"))?;
                let mpv_volume = global_config_mapper::get_cache("mpv_volume", &app_handle.state()).await.unwrap_or("100".to_string());
                if mpv_volume.parse::<f64>().unwrap_or(100.0) != volume {
                    let res = global_config_mapper::create_or_update(GlobalConfig {
                        config_key: Some("mpv_volume".to_string()),
                        config_value: Some((volume as usize).to_string()),
                        ..Default::default()}, &app_handle.state()).await;
                    if res.is_err() {
                        tracing::error!("保存音量失败: {:?}", res.err());
                    }
                }
            }
            continue;
        }
        if !play_info_init_finished {
            if Some("file-loaded") == json.event {
                send_task = Some(play_info_init(&playback_process_param).await?);
                play_info_init_finished = true;
            }
            continue;
        }
        if let Some("end-file") = json.event {
            tracing::debug!("MPV IPC 播放结束");
            if let Some(send_task) = send_task { send_task.abort(); }
            if json.reason == Some("eof") || json.reason == Some("stop") {
                save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            } else {
                save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Quit).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            }
            break;
        }
        if let Some(10022) = json.request_id {
            if let Some(progress_percent) = &json.data {
                tracing::debug!("MPV IPC 播放进度百分比 {}", progress_percent);
                let progress_percent = progress_percent.as_f64().ok_or(anyhow::anyhow!("播放进度百分比不是数字"))?;
                last_record_position = Decimal::from_f64(progress_percent).unwrap().round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Playing).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
                }
            }
            continue;
        }
        if let Some(10023) = json.request_id {
            if let Some(progress) = &json.data {
                tracing::debug!("MPV IPC 播放进度 {}", progress);
                let progress = progress.as_f64().ok_or(anyhow::anyhow!("播放进度不是数字"))?;
                last_record_position = Decimal::from_f64(progress).unwrap().round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&playback_process_param, last_record_position, PlayingProgressEnum::Playing).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
                }
            }
            continue;
        }
    }
    anyhow::Ok(())
}

async fn play_info_init(playback_process_param: &PlaybackProcessParam) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    // 向mpv添加音频字幕
    let PlaybackProcessParam {
        params,
        episode: _,
        media_source,
        playback_info,
        app_handle,
        axum_app_state,
        scrobble_trakt_param: _,
        start_time: _,
        emby_server,
        play_proxy_url,
        id,
        media_source_select,
        media_source_index,
        sender,
    } = playback_process_param;
    let app_state = app_handle.state::<AppState>();
    let sender = sender.clone().unwrap();
    
    // 发送多版本命令参数
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
    
    // 发送媒体标题
    let set_force_media_title_command = format!(r#"{{ "command": ["set_property", "force-media-title", "{}"] }}{}"#, params.media_title, "\n");
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
            let command = format!(r#"{{ "command": ["audio-add", "{}", "auto"] }}{}"#, audio_url, "\n");
            sender.write().await.write_all(command.as_bytes()).await?;
            sender.write().await.flush().await?;
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
                subtitle_url = format!("http://127.0.0.1:{}/subtitle/{}", &axum_app_state.port, &uuid);
            }
            let command = format!(r#"{{ "command": ["sub-add", "{}"] }}{}"#, subtitle_url, "\n");
            sender.write().await.write_all(command.as_bytes()).await?;
            sender.write().await.flush().await?;
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
            track_titles.video.push(format!("{} / {}", media_source.name.clone(), media_stream.display_title.clone().unwrap_or("".to_string())));
            video_index += 1;
            if media_stream.is_default == Some(true) {
                vid = video_index;
            }
        } else if media_stream.type_ == "Audio" {
            track_titles.audio.push(media_stream.display_title.clone().unwrap_or("".to_string()));
            audio_index += 1;
            if media_stream.is_default == Some(true) && aid == 0 {
                aid = audio_index;
            }
        } else if media_stream.type_ == "Subtitle" {
            track_titles.sub.push(media_stream.display_title.clone().unwrap_or("".to_string()));
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
    sender.write().await.write_all(set_track_titles_command.as_bytes()).await?;
    sender.write().await.flush().await?;
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
    let property = match vid {
        -1 => "no".to_string(),
        0 => "auto".to_string(),
        _ => vid.to_string()
    };
    let command = format!(r#"{{ "command": ["set_property", "vid", "{}"] }}{}"#, property, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;
    let property = match aid {
        -1 => "no".to_string(),
        0 => "auto".to_string(),
        _ => aid.to_string()
    };
    let command = format!(r#"{{ "command": ["set_property", "aid", "{}"] }}{}"#, property, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;
    let property = match sid {
        -1 => "no".to_string(),
        0 => "auto".to_string(),
        _ => sid.to_string()
    };
    let command = format!(r#"{{ "command": ["set_property", "sid", "{}"] }}{}"#, property, "\n");
    sender.write().await.write_all(command.as_bytes()).await?;
    sender.write().await.flush().await?;

    // 观测播放进度，返回太频繁，改为每2秒获取一次，用户跳转时立即获取一次
    // let observe_property_progress_command = r#"{ "command": ["observe_property", 10023, "playback-time"]}"#.to_string() + "\n";
    // let write = sender.write_all(observe_property_progress_command.as_bytes()).await;
    // let _ = sender.flush().await;
    // if write.is_err() {
    //     tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
    // }

    let run_time_ticks = media_source.run_time_ticks;
    let send_task = tokio::spawn(async move {
        let command = if run_time_ticks.is_none() || run_time_ticks == Some(0) {
            r#"{ "command": ["get_property", "percent-pos"], "request_id": 10022 }"#.to_string() + "\n"
        } else {
            r#"{ "command": ["get_property", "playback-time"], "request_id": 10023 }"#.to_string() + "\n"
        };
        loop {
            let write = sender.write().await.write_all(command.as_bytes()).await;
            if write.is_err() {
                tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
                break;
            }
            let _ = sender.write().await.flush().await;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    tracing::debug!("init mpv play info finished");
    
    // emby开始播放api
    let _ = emby_http_svc::playing(EmbyPlayingParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
        media_source_id: media_source.id.clone(),
        play_session_id: playback_info.play_session_id.clone(),
        position_ticks: params.playback_position_ticks,
    }, &app_state).await;

    Ok(send_task)
}

async fn save_playback_progress(playback_process_param: &PlaybackProcessParam, last_record_position: Decimal, playback_status: PlayingProgressEnum) -> anyhow::Result<()> {
    let PlaybackProcessParam {
        params,
        episode,
        media_source,
        playback_info,
        app_handle,
        axum_app_state: _,
        scrobble_trakt_param,
        start_time,
        emby_server,
        play_proxy_url: _,
        id: _,
        media_source_select: _,
        media_source_index: _,
        sender: _,
    } = playback_process_param;
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
            last_playback_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
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
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_name: emby_server.server_name.clone(),
                item_name: Some(params.item_name.clone()),
                item_type: Some(episode.type_.clone()),
                series_id: episode.series_id.clone(),
                series_name: episode.series_name.clone(),
                played_duration: Some(played_duration + response.played_duration.unwrap()),
                ..Default::default()
            }, &state.db_pool).await?;
        },
        None => tracing::error!("播放记录不存在，无法更新播放记录"),
    }

    // 退出调起APP
    if playback_status == PlayingProgressEnum::Quit {
        let window = app_handle.webview_windows();
        let window = window.values().next().expect("Sorry, no window found");
        window.unminimize().expect("Sorry, no window unminimize");
        window.show().expect("Sorry, no window show");
        window.set_focus().expect("Can't Bring Window to Focus");
    }
    
    let progress_percent = if let Some(run_time_ticks) = media_source.run_time_ticks {
        (last_record_position * Decimal::from_i64(1000_0000).unwrap() / Decimal::from_u64(run_time_ticks).unwrap() * Decimal::from_u64(100).unwrap()).trunc_with_scale(2)
    } else {
        last_record_position
    };
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

    emby_http_svc::playing_stopped(EmbyPlayingStoppedParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
        media_source_id: media_source.id.clone(),
        play_session_id: playback_info.play_session_id.clone(),
        position_ticks: position_ticks,
    }, &app_handle.state::<AppState>()).await?;

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

#[derive(PartialEq)]
enum PlayingProgressEnum {
    Quit,
    Stop,
    Playing,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct MpvIpcResponse<'a> {
    event: Option<&'a str>,    // 事件 end-file | audio-reconfig | video-reconfig | playback-restart | client-message | seek | file-loaded
    data: Option<serde_json::Value>,    // 获取播放进度时，返回秒
    request_id: Option<u32>,    // 请求ID，可以自定义传入，响应时会返回该ID
    id: Option<u32>,    // 观测ID，可以自定义传入，属性发生变化时会返回该ID
    reason: Option<&'a str>,    // quit | eof | error
    playlist_entry_id: Option<u32>,    // 播放列表条目ID，从1开始
    error: Option<&'a str>,     // success | property unavailable
    file_error: Option<&'a str>,    // 错误原因 loading failed
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

struct PlaybackProcessParam {
    params: MediaPlaylistParam,
    episode: Option<EpisodeItem>,
    media_source: MediaSource,
    playback_info: PlaybackInfo,
    app_handle: AppHandle,
    axum_app_state: AxumAppState,
    scrobble_trakt_param: Option<ScrobbleParam>,
    start_time: i64,
    emby_server: EmbyServer,
    play_proxy_url: Option<String>,
    id: String,
    media_source_select: usize,
    media_source_index: usize,
    #[cfg(windows)]
    sender: Option<Arc<RwLock<interprocess::os::windows::named_pipe::tokio::SendPipeStream<interprocess::os::windows::named_pipe::pipe_mode::Bytes>>>>,
    #[cfg(unix)]
    sender: Option<Arc<RwLock<interprocess::local_socket::tokio::SendHalf>>>,
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
