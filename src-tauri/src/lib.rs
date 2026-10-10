use std::{collections::HashMap, sync::Arc};

use service::axum_svc;
use tauri::{async_runtime::RwLock, Emitter, Manager};

mod controller;
mod config;
mod mapper;
mod service;
mod util;

use controller::app_http_ctl::{app_http_get_emby_icon_library, app_http_get_proxy_location, app_http_get_reverse_proxy_location};
use controller::emby_http_ctl::{emby_get_server_info, emby_authenticate_by_name, emby_logout, emby_search, emby_get_continue_play_list, emby_get_favorite_list, emby_next_up, emby_get_media_library_list, emby_get_media_library_child_latest, emby_get_media_library_child, emby_count, emby_items, emby_seasons, emby_episodes, emby_playback_info, emby_star, emby_unstar, emby_played, emby_unplayed, emby_hide_from_resume};
use controller::proxy_server_ctl::{get_proxy_server, list_all_proxy_server, add_proxy_server, update_proxy_server, delete_proxy_server};
use controller::reverse_proxy_server_ctl::{get_reverse_proxy_server, list_all_reverse_proxy_server, add_reverse_proxy_server, update_reverse_proxy_server, delete_reverse_proxy_server};
use controller::play_history_ctl::{get_play_history, page_play_history, add_play_history, update_play_history, cancel_pinned_play_history};
use controller::global_config_ctl::{get_global_config, list_all_global_config, add_global_config, update_global_config, delete_global_config};
use controller::emby_server_ctl::{get_emby_server, list_all_emby_server, add_emby_server, update_emby_server, defer_emby_server_order, update_emby_server_order, delete_emby_server};
use controller::emby_line_ctl::{get_emby_line, list_emby_server_line, list_all_emby_line, add_emby_line, update_emby_line, update_line_emby_server_name, delete_line_by_emby_server_id, delete_emby_line};
use controller::emby_icon_library_ctl::{get_emby_icon_library, list_all_emby_icon_library, add_emby_icon_library, update_emby_icon_library, delete_emby_icon_library};
use controller::invoke_ctl::{get_sys_info, call_player, go_trakt_auth, go_simkl_auth, open_url, updater, restart_app, get_runtime_config, clean_emby_image_cache, clean_icon_cache, open_folder, open_file};
use config::app_state::{AppState, DbFatalNotify};

use crate::service::{cache_svc, updater_svc};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            app_http_get_proxy_location, app_http_get_reverse_proxy_location, app_http_get_emby_icon_library,
            emby_get_server_info, emby_authenticate_by_name, emby_logout, emby_search, emby_get_continue_play_list, emby_get_favorite_list, emby_next_up, emby_get_media_library_list, emby_get_media_library_child_latest, emby_get_media_library_child, emby_count, emby_items, emby_seasons, emby_episodes, emby_playback_info, emby_star, emby_unstar, emby_played, emby_unplayed, emby_hide_from_resume,
            get_proxy_server, list_all_proxy_server, add_proxy_server, update_proxy_server, delete_proxy_server,
            get_reverse_proxy_server, list_all_reverse_proxy_server, add_reverse_proxy_server, update_reverse_proxy_server, delete_reverse_proxy_server,
            get_play_history, page_play_history, add_play_history, update_play_history, cancel_pinned_play_history,
            get_global_config, list_all_global_config, add_global_config, update_global_config, delete_global_config,
            get_emby_server, list_all_emby_server, add_emby_server, update_emby_server, defer_emby_server_order, update_emby_server_order, delete_emby_server,
            get_emby_line, list_emby_server_line, list_all_emby_line, add_emby_line, update_emby_line, update_line_emby_server_name, delete_line_by_emby_server_id, delete_emby_line,
            get_emby_icon_library, list_all_emby_icon_library, add_emby_icon_library, update_emby_icon_library, delete_emby_icon_library,
            get_sys_info, call_player, go_trakt_auth, go_simkl_auth, open_url, updater, restart_app, get_runtime_config, clean_emby_image_cache, clean_icon_cache, open_folder, open_file
        ])
        .setup(|app| {
            // 配置读取失败不再 panic（直接闪退），而是用默认值兜底继续启动，保证用户能看到提示
            let config = match config::app_config::get_config(app) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("Read Config error: {:#}", err);
                    config::app_config::Config::default()
                }
            };
            println!("Read Config: {:?}", &config);

            config::log::init(app, &config.log_level);

            // 数据库连不上时不退出程序：记录原因并继续启动，前端会弹出提示
            let db_pool = match tauri::async_runtime::block_on(config::db::init(app, &config)) {
                Ok(pool) => {
                    config::db_pool::set_db_state(config::db_pool::DbState::Ready);
                    Some(pool)
                }
                Err(err) => {
                    let reason = config::db::humanize_init_error(&config, &err);
                    tracing::error!("数据库初始化失败，应用将以无数据库模式启动: {:#}", err);
                    config::db_pool::set_db_state(config::db_pool::DbState::Failed(format!(
                        "数据库连接失败，相关数据功能不可用。\n\n{}",
                        reason
                    )));
                    None
                }
            };

            let axum_app_state = Arc::new(RwLock::new(None));
            let axum_app_state_clone = axum_app_state.clone();
            let app_handle = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let res = axum_svc::init_axum_svc(axum_app_state_clone, app_handle).await;
                if res.is_err() {
                    tracing::error!("{:#?}", res);
                }
            });

            app.manage(AppState {
                app_handle: app.app_handle().clone(),
                app_config: config,
                auxm_app_state: axum_app_state,
                api_reqwest_pool: Arc::new(RwLock::new(HashMap::new())),
                image_reqwest_pool: Arc::new(RwLock::new(HashMap::new())),
                stream_reqwest_pool: Arc::new(RwLock::new(HashMap::new())),
                emby_server_cache: Arc::new(RwLock::new(HashMap::new())),
                global_config_cache: Arc::new(RwLock::new(HashMap::new())),
                proxy_server_cache: Arc::new(RwLock::new(HashMap::new())),
                reverse_proxy_server_cache: Arc::new(RwLock::new(HashMap::new())),
                emby_http_cache: Arc::new(RwLock::new(HashMap::new())),
                db_pool,
            });

            // 缓存加载依赖数据库，失败时同样只提示不退出
            let cache_load = (|| -> anyhow::Result<()> {
                tauri::async_runtime::block_on(mapper::global_config_mapper::load_cache(&app.state()))?;
                tauri::async_runtime::block_on(mapper::proxy_server_mapper::load_cache(&app.state()))?;
                // 反代服务器缓存需在 emby_server 缓存之前加载，因为 emby_server 的 base_url 需要拼接反代地址
                tauri::async_runtime::block_on(mapper::reverse_proxy_server_mapper::load_cache(&app.state()))?;
                tauri::async_runtime::block_on(mapper::emby_server_mapper::load_cache(&app.state()))?;
                anyhow::Ok(())
            })();
            if let Err(err) = cache_load {
                tracing::error!("缓存加载失败: {:#}", err);
            }

            // 数据库不可用时把原因推给前端，让前端弹出明确的错误提示
            if let Some(reason) = app
                .state::<AppState>()
                .db_pool
                .is_none()
                .then(|| app.state::<AppState>().db_unavailable_reason())
            {
                let app_handle = app.app_handle().clone();
                // 前端监听器是在 app.mount 之后才注册的，延迟一会儿再发，避免事件丢失
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    let _ = app_handle.emit("db_fatal_error", DbFatalNotify { reason });
                    let _ = app_handle.emit("tauri_notify", config::app_state::TauriNotify {
                        event_type: "ElMessage".to_string(),
                        message_type: "error".to_string(),
                        title: Some("数据库连接失败".to_string()),
                        message: "数据库连接失败，服务器、播放历史、设置等数据功能不可用，请检查数据库配置后重启应用".to_string(),
                    });
                });
            }
            
            let app_handle = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let res = cache_svc::init(&app_handle).await;
                if res.is_err() {
                    tracing::error!("缓存初始化失败: {:#?}", res);
                }
                let res = cache_svc::clean_plan(&app_handle).await;
                if res.is_err() {
                    tracing::error!("清理缓存计划失败: {:#?}", res);
                }
            });

            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_updater::Builder::new().build()).unwrap_or_else(|err| {
                    tracing::error!("Updater plugin error: {}", err)
                });
                let app_handle = app.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let res = updater_svc::update(app_handle).await;
                    if res.is_err() {
                        tracing::error!("自动升级失败: {:#?}", res);
                    }
                });
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::Exit => {
                    tracing::debug!("Application exiting, closing database connections...");
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        if let Some(db_pool) = state.db_pool.as_ref() {
                            tauri::async_runtime::block_on(db_pool.close());
                            tracing::debug!("Database connection closed successfully");
                        }
                    }
                }
                _ => {}
            }
        });
}
