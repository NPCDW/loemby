use std::{io::Write, path::PathBuf};

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{config::app_state::AppState, controller::{emby_http_ctl::EmbyPlaybackInfoParam, invoke_ctl::PlayVideoParam}, mapper::{emby_server_mapper, proxy_server_mapper}, service::{axum_svc::{AxumAppState, AxumAppStateRequest, PlayParam}, emby_http_svc}, util::file_util};

pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mpv_path = body.mpv_path.trim().replace("\r", "");
    let mpv_path_vec = mpv_path.split("\n").collect::<Vec<&str>>();
    let mpv_path = mpv_path_vec.iter().find(|&&path| PathBuf::from(path).is_file());
    if mpv_path.is_none() {
        return Err(format!("所有的 mpv 路径都不存在: {}", body.mpv_path));
    }
    let mpv_path = PathBuf::from(mpv_path.unwrap());
    let mpv_startup_dir = if body.mpv_startup_dir.is_none() || body.mpv_startup_dir.as_ref().unwrap().is_empty() {
        mpv_path.parent().unwrap().as_os_str().to_str().unwrap().to_string()
    } else {
        body.mpv_startup_dir.clone().unwrap()
    };

    let mpv_config_dir = app_handle.path().app_config_dir().unwrap().join("mpv_config");
    if !mpv_config_dir.exists() {
        let res = file_util::mkdir(&mpv_config_dir);
        if res.is_err() {
            return Err(format!("创建 mpv 配置目录失败"));
        }
    }
    let mpv_config_path = mpv_config_dir.join("loemby.mpv.conf");
    file_util::write_file(&mpv_config_path, &body.mpv_args.clone().unwrap_or("".to_string()));
    
    let auxm_app_state = state.auxm_app_state.clone();
    let app_state = auxm_app_state.read().await.clone();
    let app_state = app_state.as_ref().unwrap();

    let video_path = if body.proxy.is_some() {
        let uuid = uuid::Uuid::new_v4().to_string();
        app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
            stream_url: body.path.clone(),
            proxy_url: body.proxy.clone(),
            user_agent: body.user_agent.clone(),
        });
        format!("http://127.0.0.1:{}/stream/video/{}", &app_state.port, &uuid)
    } else {
        body.path.clone()
    };

    #[cfg(windows)]
    let pipe_name = r"\\.\pipe\mpvsocket";
    #[cfg(unix)]
    let pipe_name = r"/tmp/mpvsocket";
    let pipe_name = format!("{}-{}", &pipe_name, &uuid::Uuid::new_v4().to_string());

    let mpv_playlist_path = mpv_config_dir.join("mpv_playlist.m3u8");
    let mut mpv_playlist = std::fs::File::create(&mpv_playlist_path).unwrap();
    writeln!(mpv_playlist, "#EXTM3U").unwrap();
    for item in &body.playlist {
        // 写入EXTINF行：持续时间,标题
        writeln!(mpv_playlist, "#EXTINF:-1,{}", item.title).unwrap();
        // 写入URL
        writeln!(mpv_playlist, "http://127.0.0.1:{}/play_media?emby_server_id={}&item_id={}&mpv_ipc={}&direct_link={}&select_policy={}&video_select={}&audio_select={}&subtitle_select={}&version_select={}", &app_state.port, &body.emby_server_id, item.item_id, urlencoding::encode(&pipe_name), &body.direct_link, &body.select_policy, &body.video_select, &body.audio_select, &body.subtitle_select, &body.version_select).unwrap();
    }

    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap());
    command.current_dir(&mpv_startup_dir)
        .arg(&format!("--include={}", mpv_config_path.to_str().unwrap()))
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        // .arg("--force-seekable=yes")  // 某些视频格式在没缓存到的情况下不支持跳转，需要打开此配置，测试后发现强制跳转到没有缓存的位置后，mpv会从头开始缓存，一直缓存到跳转位置，与打开此设置的初衷相违背
        .arg(&format!("--user-agent={}", &body.user_agent))
        .arg(&format!("--title={}", &body.title))
        .arg(&format!("--force-media-title={}", &body.title))
        .arg(&format!("--playlist={}", mpv_playlist_path.to_str().unwrap()));

    if body.mpv_cache_max_bytes.is_some() && body.mpv_cache_max_bytes.unwrap() > 0 {
        command.arg(&format!("--demuxer-max-bytes={}", body.mpv_cache_max_bytes.unwrap()));
    }
    if body.mpv_cache_back_max_bytes.is_some() && body.mpv_cache_back_max_bytes.unwrap() > 0 {
        command.arg(&format!("--demuxer-max-back-bytes={}", body.mpv_cache_back_max_bytes.unwrap()));
    }

    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("播放视频: {} {:?}", &video_path, &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    Ok(())
}

async fn playback_progress(pipe_name: &str, player: &mut tokio::process::Child, body: PlayVideoParam, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    use tokio as tokio_root;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    #[cfg(windows)]
    use interprocess::os::windows::named_pipe::{pipe_mode, tokio::*};
    #[cfg(not(windows))]
    use interprocess::local_socket::{tokio::{prelude::*, Stream}, GenericFilePath};

    let mut retry_count = 0;
    let conn = loop {
        if retry_count >= 10 {
            break None;
        }
        #[cfg(windows)]
        let conn = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_name).await;
        #[cfg(not(windows))]
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
        tokio_root::time::sleep(std::time::Duration::from_millis(500)).await;
        retry_count += 1;
    };
    let conn = match conn {
        Some(conn) => conn,
        None => {
            return Err(anyhow::anyhow!("MPV IPC Failed to connect to mpv IPC"));
        }
    };
    let (recver, mut sender) = conn.split();
    let mut recver = BufReader::new(recver);

    let send_task = tokio_root::spawn(async move {
        let command = if body.run_time_ticks == 0 {
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
            tokio_root::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let mut last_save_time = chrono::Local::now();
    let mut last_record_position = Decimal::from_u64(body.playback_position_ticks).unwrap();
    loop {
        let mut buffer = String::with_capacity(128);
        let read = recver.read_line(&mut buffer).await;
        if read.is_err() {
            tracing::error!("MPV IPC Failed to read pipe {:?}", read);
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        tracing::debug!("MPV IPC Server answered: {}", buffer.trim());
        let json = serde_json::from_str::<MpvIpcResponse>(&buffer);
        if json.is_err() {
            tracing::error!("解析 mpv-ipc 响应失败 {:?}", json);
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        let json = json.unwrap();
        if let Some("end-file") = json.event {
            tracing::debug!("MPV IPC 播放结束");
            send_task.abort();
            let _ = player.kill().await;
            save_playback_progress(&body, &app_handle, last_record_position, 0);
            break;
        }
        if let Some(10022) = json.request_id {
            let progress_percent = json.data;
            if let Some(progress_percent) = progress_percent {
                tracing::debug!("MPV IPC 播放进度百分比 {}", progress_percent);
                last_record_position = Decimal::from_f64(progress_percent).unwrap().round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&body, &app_handle, last_record_position, 1);
                }
            }
        }
        if let Some(10023) = json.request_id {
            let progress = json.data;
            if let Some(progress) = progress {
                tracing::debug!("MPV IPC 播放进度 {}", progress);
                last_record_position = Decimal::from_f64(progress).unwrap() * Decimal::from_i64(1000_0000).unwrap();
                last_record_position = last_record_position.round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&body, &app_handle, last_record_position, 1);
                }
            }
        }
    }
    anyhow::Ok(())
}

fn save_playback_progress(body: &PlayVideoParam, app_handle: &tauri::AppHandle, last_record_position: Decimal, playback_status: u32) {
    if playback_status == 0 {
        let window = app_handle.webview_windows();
        let window = window.values().next().expect("Sorry, no window found");
        window.unminimize().expect("Sorry, no window unminimize");
        window.show().expect("Sorry, no window show");
        window.set_focus().expect("Can't Bring Window to Focus");
    }
    app_handle.emit("playback_progress", PlaybackProgress {
        server_id: &body.server_id,
        item_id: &body.item_id,
        item_type: &body.item_type,
        item_name: &body.item_name,
        series_id: body.series_id.clone(),
        series_name: body.series_name.clone(),
        media_source_id: &body.media_source_id,
        play_session_id: &body.play_session_id,
        progress: last_record_position,
        run_time_ticks: body.run_time_ticks,
        scrobble_trakt_param: body.scrobble_trakt_param.clone(),
        playback_status: playback_status,
        start_time: body.start_time,
    }).unwrap();
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
struct PlaybackProgress<'a> {
    server_id: &'a str,
    item_id: &'a str,
    item_type: &'a str,
    item_name: &'a str,
    series_id: Option<String>,
    series_name: Option<String>,
    media_source_id: &'a str,
    play_session_id: &'a str,
    progress: Decimal,
    pub run_time_ticks: u64,
    pub scrobble_trakt_param: Option<String>,
    // 0 停止  1 播放中
    playback_status: u32,
    start_time: u64,
}

pub async fn play_media(axum_app_state: &AxumAppState, params: &PlayParam) -> anyhow::Result<()> {
    let (recver, mut sender) = get_pipe_rw(&params.mpv_ipc).await?;

    let app_state = axum_app_state.app.state::<AppState>().clone();
    let emby_server = match emby_server_mapper::get_cache(params.emby_server_id.clone(), &app_state).await {
        Some(emby_server) => emby_server,
        None => return Err(anyhow::anyhow!("emby_server not found")),
    };
    let play_proxy_url = proxy_server_mapper::get_play_proxy_url(emby_server.play_proxy_id, &app_state).await;

    let playback_info_param = EmbyPlaybackInfoParam {
        emby_server_id: params.emby_server_id.clone(),
        item_id: params.item_id.clone(),
    };
    let playback_info = emby_http_svc::playback_info(playback_info_param, &app_state).await?;
    let playback_info = serde_json::from_str::<PlaybackInfo>(&playback_info)?;
    if let Some(error_code) = playback_info.error_code {
        return Err(anyhow::anyhow!("Emby Playback Info Error: {}", error_code));
    }
    // todo!("自动或手动选择媒体源");
    for media_sources in playback_info.media_sources {
        let mut video_url = format!("{}/emby{}", emby_server.base_url.as_ref().unwrap(), media_sources.direct_stream_url);
        if play_proxy_url.is_some() {
            let uuid = uuid::Uuid::new_v4().to_string();
            axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                stream_url: video_url,
                proxy_url: play_proxy_url.clone(),
                user_agent: emby_server.user_agent.as_ref().unwrap().clone(),
            });
            video_url = format!("http://127.0.0.1:{}/stream/video/{}", &axum_app_state.port, &uuid);
        }

        let media_source_item_id = if media_sources.item_id.is_some() {media_sources.item_id.unwrap()} else {params.item_id.clone()};
        for media_stream in media_sources.media_streams {
            if media_stream.is_external.is_none() || media_stream.is_external == Some(false) {
                continue;
            }
            if media_stream.type_ == "Video" {
                let video_title = format!("{} / {}", media_sources.name, media_stream.display_title);
                let command = format!(r#"{{ "command": ["video-add", "{}", "auto", "{}"], "request_id": 10023 }}{}"#, video_url, video_title, "\n");
                sender.write_all(command.as_bytes()).await?;
            } else if media_stream.type_ == "Audio" {
                let mut audio_url = format!("{}/emby/Audio/{}/stream.{}?AudioStreamIndex={}&Static=true", emby_server.base_url.as_ref().unwrap(), media_source_item_id, media_stream.codec, media_stream.index);
                if play_proxy_url.is_some() {
                    let uuid = uuid::Uuid::new_v4().to_string();
                    axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                        stream_url: audio_url,
                        proxy_url: play_proxy_url.clone(),
                        user_agent: emby_server.user_agent.as_ref().unwrap().clone(),
                    });
                    audio_url = format!("http://127.0.0.1:{}/stream/audio/{}", &axum_app_state.port, &uuid);
                }
                let audio_title = format!("{} / {}", media_stream.display_title, media_stream.display_language.unwrap_or_default());
                let command = format!(r#"{{ "command": ["audio-add", "{}", "auto", "{}"], "request_id": 10023 }}{}"#, audio_url, audio_title, "\n");
                sender.write_all(command.as_bytes()).await?;
            } else if media_stream.type_ == "Subtitle" {
                let mut subtitle_url = format!("{}/emby/Videos/{}/{}/Subtitles/{}/Stream.{}", emby_server.base_url.as_ref().unwrap(), media_source_item_id, media_sources.id, media_stream.index, media_stream.codec);
                if play_proxy_url.is_some() {
                    let uuid = uuid::Uuid::new_v4().to_string();
                    axum_app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                        stream_url: subtitle_url,
                        proxy_url: play_proxy_url.clone(),
                        user_agent: emby_server.user_agent.as_ref().unwrap().clone(),
                    });
                    subtitle_url = format!("http://127.0.0.1:{}/stream/subtitle/{}", &axum_app_state.port, &uuid);
                }
                let subtitle_title = format!("{} / {}", media_stream.display_title, media_stream.display_language.unwrap_or_default());
                let command = format!(r#"{{ "command": ["sub-add", "{}", "auto", "{}"], "request_id": 10023 }}{}"#, subtitle_url, subtitle_title, "\n");
                sender.write_all(command.as_bytes()).await?;
            }
        }
    }

    // if body.vid == -1 {
    //     command.arg(&format!("--vid=no"));
    // } else if body.vid == 0 {
    //     command.arg(&format!("--vid=auto"));
    // } else {
    //     command.arg(&format!("--vid={}", body.vid));
    // }
    // if body.aid == -1 {
    //     command.arg(&format!("--aid=no"));
    // } else if body.aid == 0 {
    //     command.arg(&format!("--aid=auto"));
    // } else {
    //     command.arg(&format!("--aid={}", body.aid));
    // }
    // if body.sid == -1 {
    //     command.arg(&format!("--sid=no"));
    // } else if body.sid == 0 {
    //     command.arg(&format!("--sid=auto"));
    // } else {
    //     command.arg(&format!("--sid={}", body.sid));
    // }






    // let body_clone = body.clone();
    // let app_handle_clone = app_handle.clone();
    // tauri::async_runtime::spawn(async move {
    //     let res = playback_progress(&pipe_name, &mut player.unwrap(), body_clone, app_handle_clone).await;
    //     if res.is_err() {
    //         tracing::error!("播放进度失败: {:?}", res.unwrap_err());
    //         save_playback_progress(&body, &app_handle, Decimal::from_u64(body.playback_position_ticks).unwrap(), 0);
    //     }
    // });

    Ok(())
}

async fn get_pipe_rw(pipe_name: &str) -> anyhow::Result<(interprocess::local_socket::tokio::RecvHalf, interprocess::local_socket::tokio::SendHalf)> {
    #[cfg(windows)]
    use interprocess::os::windows::named_pipe::{pipe_mode, tokio::*};
    #[cfg(not(windows))]
    use interprocess::local_socket::{tokio::{prelude::*, Stream}, GenericFilePath};

    let mut retry_count = 0;
    let conn = loop {
        if retry_count >= 10 {
            break None;
        }
        #[cfg(windows)]
        let conn = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(pipe_name).await;
        #[cfg(not(windows))]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackInfo {
    #[serde(rename = "PlaySessionId")]
    pub play_session_id: String,

    #[serde(rename = "MediaSources")]
    pub media_sources: Vec<MediaSource>,

    #[serde(rename = "ErrorCode")]
    pub error_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaSource {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "ItemId")]
    pub item_id: Option<String>,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "RunTimeTicks")]
    pub run_time_ticks: Option<i64>,

    #[serde(rename = "Size")]
    pub size: Option<u64>,

    #[serde(rename = "Bitrate")]
    pub bitrate: Option<u64>,

    #[serde(rename = "DirectStreamUrl")]
    pub direct_stream_url: String,

    #[serde(rename = "MediaStreams")]
    pub media_streams: Vec<MediaStream>,

    #[serde(rename = "IsRemote")]
    pub is_remote: Option<bool>,

    #[serde(rename = "Path")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaStream {
    #[serde(rename = "Codec")]
    pub codec: String,

    #[serde(rename = "DisplayTitle")]
    pub display_title: String,

    #[serde(rename = "DisplayLanguage")]
    pub display_language: Option<String>,

    #[serde(rename = "Title")]
    pub title: Option<String>,

    #[serde(rename = "BitRate")]
    pub bit_rate: Option<u32>,

    #[serde(rename = "Height")]
    pub height: Option<u32>, // 有些媒体流（如音频）可能没有高度

    #[serde(rename = "Width")]
    pub width: Option<u32>, // 同上

    #[serde(rename = "Type")]
    pub type_: String,

    #[serde(rename = "Language")]
    pub language: Option<String>,

    #[serde(rename = "Index")]
    pub index: u32,

    #[serde(rename = "IsDefault")]
    pub is_default: Option<bool>,

    #[serde(rename = "IsExternal")]
    pub is_external: Option<bool>,
}