use std::{cmp::{max, min}, path::PathBuf};

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{config::app_state::{AppState, TauriNotify}, controller::invoke_ctl::PlayVideoParam, service::{emby_http_svc::{EmbyPlayingParam, EmbyPlayingProgressParam, EmbyPlayingStoppedParam}}, mapper::{emby_server_mapper::{self, EmbyServer}, global_config_mapper, play_history_mapper::{self, PlayHistory}, proxy_server_mapper}, service::{axum_svc::AxumAppStateRequest, emby_http_svc, trakt_http_svc}, util::file_util};

pub async fn play_video(mut body: PlayVideoParam, state: &tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    body.start_time = chrono::Local::now().timestamp();
    let emby_server = match emby_server_mapper::get_cache(&body.emby_server_id, state).await {
        Some(emby_server) => emby_server,
        None => return Err("emby_server not found".to_string()),
    };
    body.emby_server_name = emby_server.server_name.clone().unwrap();
    body.title = format!("{}{}", body.title.clone(), emby_server.server_name.clone().unwrap());
    let proxy_url = proxy_server_mapper::get_browse_proxy_url(emby_server.browse_proxy_id, state).await;
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

    let mpv_config_path = app_handle.path().app_config_dir().unwrap().join("mpv_config");
    if !mpv_config_path.exists() {
        let res = file_util::mkdir(&mpv_config_path);
        if res.is_err() {
            return Err(format!("创建 mpv 配置目录失败"));
        }
    }
    let mpv_config_path = mpv_config_path.join("loemby.mpv.conf");
    file_util::write_file(&mpv_config_path, &mpv_args);

    let auxm_app_state = state.auxm_app_state.clone();
    let app_state = auxm_app_state.read().await.clone().unwrap();

    let video_path = if proxy_url.is_some() {
        let uuid = uuid::Uuid::new_v4().to_string();
        app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
            stream_url: format!("{}{}", emby_server.base_url.clone().unwrap(), body.path.clone()),
            proxy_url: proxy_url.clone(),
            user_agent: emby_server.user_agent.clone().unwrap(),
        });
        format!("http://127.0.0.1:{}/stream/video/{}", &app_state.port, &uuid)
    } else {
        body.path.clone()
    };

    #[cfg(windows)]
    let pipe_name = r"\\.\pipe\mpvsocket";
    #[cfg(unix)]
    let pipe_name = r"/tmp/mpvsocket";
    let pipe_name = format!("{}-{}", &pipe_name, uuid::Uuid::new_v4().to_string());
    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap());
    command.current_dir(&mpv_startup_dir)
        .arg(&format!("--include={}", mpv_config_path.to_str().unwrap()))
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        // .arg("--force-seekable=yes")  // 某些视频格式在没缓存到的情况下不支持跳转，需要打开此配置，测试后发现强制跳转到没有缓存的位置后，mpv会从头开始缓存，一直缓存到跳转位置，与打开此设置的初衷相违背
        .arg(&format!("--user-agent={}", &emby_server.user_agent.clone().unwrap()))
        .arg(&format!("--title={}", &body.title))
        .arg(&format!("--force-media-title={}", &body.title))
        .arg(&format!("--start=+{}", body.playback_position_ticks / 1000_0000))
        .arg(&video_path);

    let mut cache_max_bytes = 300 * 1024 * 1024;
    let mut cache_back_max_bytes = 150 * 1024 * 1024;
    if body.bitrate.is_some() && body.bitrate.unwrap() > 0 {
        let mpv_cache_seconds = global_config_mapper::get_cache("mpv_cache_seconds", state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(0);
        let mpv_cache_min_bytes = global_config_mapper::get_cache("mpv_cache_min_bytes", state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(0);
        let mpv_cache_max_bytes = global_config_mapper::get_cache("mpv_cache_max_bytes", state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(0);
        let mpv_cache_back_seconds = global_config_mapper::get_cache("mpv_cache_back_seconds", state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(0);
        let mpv_cache_back_min_bytes = global_config_mapper::get_cache("mpv_cache_back_min_bytes", state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(0);
        let mpv_cache_back_max_bytes = global_config_mapper::get_cache("mpv_cache_back_max_bytes", state).await.unwrap_or("0".to_string()).parse::<u64>().unwrap_or(0);
        let mpv_cache_max_bytes = max(min(mpv_cache_seconds * body.bitrate.unwrap() / 8, mpv_cache_max_bytes * 1024 * 1024), mpv_cache_min_bytes * 1024 * 1024);
        let mpv_cache_back_max_bytes = max(min(mpv_cache_back_seconds * body.bitrate.unwrap() / 8, mpv_cache_back_max_bytes * 1024 * 1024), mpv_cache_back_min_bytes * 1024 * 1024);

        if mpv_cache_max_bytes > 0 {
            cache_max_bytes = mpv_cache_max_bytes;
        }
        if mpv_cache_back_max_bytes > 0 {
            cache_back_max_bytes = mpv_cache_back_max_bytes;
        }
    }
    command.arg(&format!("--demuxer-max-bytes={}", cache_max_bytes));
    command.arg(&format!("--demuxer-max-back-bytes={}", cache_back_max_bytes));

    for audio in &body.external_audio {
        let audio_path = if proxy_url.is_some() {
            let uuid = uuid::Uuid::new_v4().to_string();
            app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                stream_url: format!("{}{}", emby_server.base_url.clone().unwrap(), audio.clone()),
                proxy_url: proxy_url.clone(),
                user_agent: emby_server.user_agent.clone().unwrap().clone(),
            });
            format!("http://127.0.0.1:{}/stream/audio/{}", &app_state.port, &uuid)
        } else {
            audio.clone()
        };
        command.arg(&format!("--audio-file={}", audio_path));
    }
    for subtitle in &body.external_subtitle {
        let subtitle_path = if proxy_url.is_some() {
            let uuid = uuid::Uuid::new_v4().to_string();
            app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                stream_url: format!("{}{}", emby_server.base_url.clone().unwrap(), subtitle.clone()),
                proxy_url: proxy_url.clone(),
                user_agent: emby_server.user_agent.clone().unwrap().clone(),
            });
            format!("http://127.0.0.1:{}/stream/subtitle/{}", &app_state.port, &uuid)
        } else {
            subtitle.clone()
        };
        command.arg(&format!("--sub-file={}", subtitle_path));
    }
    if body.vid == -1 {
        command.arg(&format!("--vid=no"));
    } else if body.vid == 0 {
        command.arg(&format!("--vid=auto"));
    } else {
        command.arg(&format!("--vid={}", body.vid));
    }
    if body.aid == -1 {
        command.arg(&format!("--aid=no"));
    } else if body.aid == 0 {
        command.arg(&format!("--aid=auto"));
    } else {
        command.arg(&format!("--aid={}", body.aid));
    }
    if body.sid == -1 {
        command.arg(&format!("--sid=no"));
    } else if body.sid == 0 {
        command.arg(&format!("--sid=auto"));
    } else {
        command.arg(&format!("--sid={}", body.sid));
    }
    tracing::debug!("调用MPV: {:?}", &command);
    
    let player = command.spawn();

    tracing::debug!("播放视频: {} {:?}", &video_path, &player);
    if player.is_err() {
        return Err(player.err().unwrap().to_string());
    }

    let body_clone = body.clone();
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let res = playback_progress(&pipe_name, &mut player.unwrap(), body_clone, app_handle_clone).await;
        if res.is_err() {
            tracing::error!("播放进度失败: {:?}", res.unwrap_err());
        }
    });

    let mut pinned = 0;
    if let Some(series_id) = body.series_id.clone() {
        let pinned_update = play_history_mapper::cancel_pinned(body.emby_server_id.clone(), series_id, &state.db_pool).await.unwrap();
        if pinned_update.rows_affected() > 0 { pinned = 1 }
    }
    match play_history_mapper::get(body.emby_server_id.clone(), body.item_id.clone(), &state.db_pool).await.unwrap() {
        Some(response) => {
            if body.series_id.is_none() {
                pinned = response.pinned.unwrap();
            }
            play_history_mapper::update_by_id(PlayHistory {
                id: response.id,
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_name: Some(body.emby_server_name),
                item_name: Some(body.item_name),
                item_type: Some(body.item_type),
                series_id: body.series_id,
                series_name: body.series_name,
                pinned: Some(pinned),
                ..Default::default()
            }, &state.db_pool).await.unwrap();
        },
        None => {
            play_history_mapper::create(PlayHistory {
                id: Some(uuid::Uuid::new_v4().to_string()),
                create_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_id: Some(body.emby_server_id.clone()),
                emby_server_name: Some(body.emby_server_name),
                item_id: Some(body.item_id.clone()),
                item_name: Some(body.item_name),
                item_type: Some(body.item_type),
                series_id: body.series_id,
                series_name: body.series_name,
                played_duration: Some(0),
                pinned: Some(pinned),
            }, &state.db_pool).await.unwrap();
        },
    }

    let res = emby_http_svc::playing(EmbyPlayingParam {
        emby_server_id: body.emby_server_id.clone(),
        item_id: body.item_id.clone(),
        media_source_id: body.media_source_id.clone(),
        play_session_id: body.play_session_id.clone(),
        position_ticks: body.playback_position_ticks,
    }, state).await;
    if res.is_err() {
        app_handle.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessage".to_string(),
            message_type: "error".to_string(),
            title: None,
            message: format!("调用emby播放进度失败: {}", res.unwrap_err()),
        }).unwrap()
    }

    if let Some(scrobble_trakt_param) = body.scrobble_trakt_param.clone() {
        match trakt_http_svc::start(scrobble_trakt_param, state, 0).await {
            Ok(json) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    alert_type: "TraktStart".to_string(),
                    message_type: "success".to_string(),
                    title: None,
                    message: json,
                }).unwrap(),
            Err(err) => 
                app_handle.emit("tauri_notify", TauriNotify {
                    alert_type: "ElMessage".to_string(),
                    message_type: "error".to_string(),
                    title: None,
                    message: format!("调用trakt开始播放失败: {}", err),
                }).unwrap()
        }
    }

    Ok(())
}

async fn playback_progress(pipe_name: &str, player: &mut tokio::process::Child, body: PlayVideoParam, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let (recver, mut sender) = get_pipe_rw(pipe_name).await?;
    let mut recver = BufReader::new(recver);

    let track_titles = body.track_titles.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
    let set_track_titles_command = format!(r#"{{ "command": ["script-message-to", "uosc", "set-track-title", "{}"] }}{}"#, track_titles, "\n");
    let write = sender.write_all(set_track_titles_command.as_bytes()).await;
    if write.is_err() {
        tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
    }

    // 观测播放进度，返回太频繁，改为每5秒获取一次，用户跳转时立即获取一次
    // let observe_property_progress_command = r#"{ "command": ["observe_property", 10023, "playback-time"]}"#.to_string() + "\n";
    // let write = sender.write_all(observe_property_progress_command.as_bytes()).await;
    // if write.is_err() {
    //     tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
    // }

    let send_task = tokio::spawn(async move {
        let get_progress_command = r#"{ "command": ["get_property", "playback-time"], "request_id": 10023 }"#.to_string() + "\n";
        loop {
            let write = sender.write_all(get_progress_command.as_bytes()).await;
            if write.is_err() {
                tracing::debug!("MPV IPC Failed to write to pipe {:?}", write);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let mut last_save_time = chrono::Local::now();
    let mut last_record_position = Decimal::from_u64(body.playback_position_ticks).unwrap();
    loop {
        let mut buffer = String::new();
        let read = recver.read_line(&mut buffer).await;
        if read.is_err() {
            tracing::error!("MPV IPC Failed to read pipe {:?}", read);
            send_task.abort();
            let _ = player.kill().await;
            save_playback_progress(&body, &app_handle, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        tracing::debug!("MPV IPC Server answered: {}", buffer.trim());
        if buffer.trim().is_empty() {
            send_task.abort();
            let _ = player.kill().await;
            save_playback_progress(&body, &app_handle, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            tracing::error!("mpv-ipc 响应为空，连接已断开");
            break;
        }
        let json = serde_json::from_str::<MpvIpcResponse>(&buffer);
        if json.is_err() {
            tracing::error!("解析 mpv-ipc 响应失败 {:?}", json);
            send_task.abort();
            let _ = player.kill().await;
            save_playback_progress(&body, &app_handle, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        let json = json.unwrap();
        if let Some("end-file") = json.event {
            tracing::debug!("MPV IPC 播放结束");
            send_task.abort();
            let _ = player.kill().await;
            save_playback_progress(&body, &app_handle, last_record_position, PlayingProgressEnum::Stop).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
            break;
        }
        if let Some("seek") = json.event {
            continue;
        }
        if let Some(10023) = json.request_id {
            let progress = json.data;
            if let Some(progress) = progress {
                tracing::debug!("MPV IPC 播放进度 {}", progress);
                last_record_position = Decimal::from_f64(progress).unwrap().round();
                if chrono::Local::now() - last_save_time >= chrono::Duration::seconds(30) {
                    last_save_time = chrono::Local::now();
                    save_playback_progress(&body, &app_handle, last_record_position, PlayingProgressEnum::Playing).await.unwrap_or_else(|e| tracing::error!("保存播放进度失败: {:?}", e));
                }
            }
        }
    }
    anyhow::Ok(())
}

async fn save_playback_progress(body: &PlayVideoParam, app_handle: &tauri::AppHandle, last_record_position: Decimal, playback_status: PlayingProgressEnum) -> anyhow::Result<()> {
    let position_ticks = (last_record_position * Decimal::from_i64(1000_0000).unwrap()).to_u64().unwrap();
    let state = app_handle.state::<AppState>();
    if playback_status == PlayingProgressEnum::Playing {
        emby_http_svc::playing_progress(EmbyPlayingProgressParam {
            emby_server_id: body.emby_server_id.clone(),
            item_id: body.item_id.clone(),
            media_source_id: body.media_source_id.clone(),
            play_session_id: body.play_session_id.clone(),
            position_ticks: position_ticks,
        }, &state).await?;
        return Ok(());
    }

    let played_duration = chrono::Local::now().timestamp() - body.start_time;
    if played_duration > 300 {
        emby_server_mapper::update_by_id(EmbyServer {
            id: Some(body.emby_server_id.clone()),
            last_playback_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            ..Default::default()
        }, &state).await?;
        app_handle.emit("EmbyServerChange", EmbyServerChangeParam {
            id: &body.emby_server_id,
            event: "update",
        })?;
    } else {
        app_handle.emit("tauri_notify", TauriNotify {
            alert_type: "ElMessage".to_string(),
            message_type: "warning".to_string(),
            title: None,
            message: format!("播放时间不足 5 分钟，不更新最后播放时间"),
        }).unwrap()
    }
    
    match play_history_mapper::get(body.emby_server_id.clone(), body.item_id.clone(), &state.db_pool).await? {
        Some(response) => {
            play_history_mapper::update_by_id(PlayHistory {
                id: response.id,
                update_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                emby_server_name: Some(body.emby_server_name.clone()),
                item_name: Some(body.item_name.clone()),
                item_type: Some(body.item_type.clone()),
                series_id: body.series_id.clone(),
                series_name: body.series_name.clone(),
                played_duration: Some(played_duration + response.played_duration.unwrap()),
                ..Default::default()
            }, &state.db_pool).await?;
        },
        None => tracing::error!("播放记录不存在，无法更新播放记录"),
    }

    let window = app_handle.webview_windows();
    let window = window.values().next().expect("Sorry, no window found");
    window.unminimize().expect("Sorry, no window unminimize");
    window.show().expect("Sorry, no window show");
    window.set_focus().expect("Can't Bring Window to Focus");
    
    let progress_percent = if body.run_time_ticks == 0 {
        last_record_position
    } else {
        (last_record_position * Decimal::from_i64(1000_0000).unwrap() / Decimal::from_u64(body.run_time_ticks).unwrap() * Decimal::from_u64(100).unwrap()).trunc_with_scale(2)
    };
    if let Some(scrobble_trakt_param) = body.scrobble_trakt_param.clone() {
        match serde_json::from_str::<serde_json::Value>(&scrobble_trakt_param) {
            Err(err) => tracing::error!("解析scrobble_trakt_param失败: {}", err),
            Ok(mut scrobble_trakt_param) => {
                scrobble_trakt_param["progress"] = serde_json::to_value(progress_percent).unwrap();
                match trakt_http_svc::stop(scrobble_trakt_param.to_string(), &app_handle.state(), 0).await {
                    Ok(json) => 
                        app_handle.emit("tauri_notify", TauriNotify {
                            alert_type: "TraktStop".to_string(),
                            message_type: "success".to_string(),
                            title: None,
                            message: json,
                        }).unwrap(),
                    Err(err) => 
                        app_handle.emit("tauri_notify", TauriNotify {
                            alert_type: "ElMessage".to_string(),
                            message_type: "error".to_string(),
                            title: None,
                            message: format!("调用trakt停止播放失败: {}", err),
                        }).unwrap()
                }
            },
        }
    }

    emby_http_svc::playing_stopped(EmbyPlayingStoppedParam {
        emby_server_id: body.emby_server_id.clone(),
        item_id: body.item_id.clone(),
        media_source_id: body.media_source_id.clone(),
        play_session_id: body.play_session_id.clone(),
        position_ticks: position_ticks,
    }, &app_handle.state::<AppState>()).await?;

    app_handle.emit("playingStopped", PlaybackStoppedParam {
        emby_server_id: &body.emby_server_id,
        item_id: &body.item_id,
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
