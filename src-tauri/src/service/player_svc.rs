use std::path::PathBuf;

use rust_decimal::prelude::*;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

use crate::{config::{app_state::AppState, http_pool}, controller::invoke_ctl::PlayVideoParam, service::proxy_svc::AxumAppStateRequest, util::file_util};

pub async fn play_video(body: PlayVideoParam, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mpv_path = body.mpv_path.clone();
    let mpv_path = PathBuf::from(mpv_path);
    if !mpv_path.exists() {
        return Err(format!("mpv 路径不存在: {}", mpv_path.to_str().unwrap()));
    }
    let mpv_parent_path = mpv_path.parent().unwrap();

    let mpv_config_path = app_handle.path().app_config_dir().unwrap().join("mpv_config");
    if !mpv_config_path.exists() {
        let res = file_util::mkdir(&mpv_config_path);
        if res.is_err() {
            return Err(format!("创建 mpv 配置目录失败"));
        }
    }
    let mpv_config_path = mpv_config_path.join("loemby.conf");
    file_util::write_file(&mpv_config_path, &body.mpv_args);

    let auxm_app_state = state.auxm_app_state.clone();
    let app_state = auxm_app_state.read().await.clone();
    let app_state = app_state.as_ref().unwrap();

    let client = match http_pool::get_http_client(body.proxy.clone(), state).await {
        Ok(client) => client,
        Err(err) => return Err(err.to_string())
    };
    let video_path = if body.proxy.is_some() {
        let uuid = uuid::Uuid::new_v4().to_string();
        app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
            stream_url: body.path.clone(),
            client: client.clone(),
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
    let pipe_name = format!("{}-{}-{}", &pipe_name, &body.server_id, &body.media_source_id);
    let mut command = tokio::process::Command::new(&mpv_path.as_os_str().to_str().unwrap());
    command.current_dir(&mpv_parent_path.as_os_str().to_str().unwrap())
        .arg(&format!("--include={}", mpv_config_path.to_str().unwrap()))
        .arg(&format!("--input-ipc-server={}", pipe_name))
        .arg("--terminal=no")  // 不显示控制台输出
        .arg("--force-window=immediate")  // 先打开窗口再加载视频
        // .arg("--force-seekable=yes")  // 某些视频格式在没缓存到的情况下不支持跳转，需要打开此配置，测试后发现强制跳转到没有缓存的位置后，mpv会从头开始缓存，一直缓存到跳转位置，与打开此设置的初衷相违背
        // .arg(&format!("--user-agent={}", &body.user_agent))
        .arg(&format!("--title={}", &body.title))
        .arg(&format!("--force-media-title={}", &body.title))
        .arg(&format!("--start=+{}", body.playback_position_ticks / 1000_0000))
        .arg(&video_path);

    for audio in &body.external_audio {
        let audio_path = if body.proxy.is_some() {
            let uuid = uuid::Uuid::new_v4().to_string();
            app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                stream_url: audio.clone(),
                client: client.clone(),
                user_agent: body.user_agent.clone(),
            });
            format!("http://127.0.0.1:{}/stream/audio/{}", &app_state.port, &uuid)
        } else {
            audio.clone()
        };
        command.arg(&format!("--audio-file={}", audio_path));
    }
    for subtitle in &body.external_subtitle {
        let subtitle_path = if body.proxy.is_some() {
            let uuid = uuid::Uuid::new_v4().to_string();
            app_state.request.write().await.insert(uuid.clone(), AxumAppStateRequest {
                stream_url: subtitle.clone(),
                client: client.clone(),
                user_agent: body.user_agent.clone(),
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
        command.arg(&format!("--vid={}", body.aid));
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
            save_playback_progress(&body, &app_handle, Decimal::from_u64(body.playback_position_ticks).unwrap(), 0);
        }
    });

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
        let command = r#"{ "command": ["get_property", "playback-time"], "request_id": 10023 }"#.to_string() + "\n";
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
        media_source_id: &body.media_source_id,
        play_session_id: &body.play_session_id,
        progress: last_record_position,
        run_time_ticks: body.run_time_ticks,
        scrobble_trakt_param: body.scrobble_trakt_param.clone(),
        playback_status: playback_status,
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
    media_source_id: &'a str,
    play_session_id: &'a str,
    progress: Decimal,
    pub run_time_ticks: u64,
    pub scrobble_trakt_param: Option<String>,
    // 0 停止  1 播放中
    playback_status: u32,
}