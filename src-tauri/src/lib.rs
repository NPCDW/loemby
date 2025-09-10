use std::{collections::HashMap, sync::Arc};

use service::axum_svc;
use tauri::{async_runtime::RwLock, Manager};

mod controller;
mod config;
mod mapper;
mod service;
mod util;

use controller::trakt_http_ctl::{trakt_http_token, trakt_http_get_user_info, trakt_http_start, trakt_http_stop};
use controller::app_http_ctl::{app_http_get_emby_icon_library, app_http_get_proxy_location};
use controller::emby_http_ctl::{emby_get_server_info, emby_authenticate_by_name, emby_logout, emby_search, emby_get_continue_play_list, emby_get_favorite_list, emby_next_up, emby_get_media_library_list, emby_get_media_library_child_latest, emby_get_media_library_child, emby_count, emby_items, emby_seasons, emby_episodes, emby_playback_info, emby_playing, emby_playing_progress, emby_playing_stopped, emby_get_direct_stream_url, emby_get_audio_stream_url, emby_get_subtitle_stream_url, emby_get_image_url, emby_star, emby_unstar, emby_played, emby_unplayed, emby_hide_from_resume};
use controller::proxy_server_ctl::{get_proxy_server, list_all_proxy_server, add_proxy_server, update_proxy_server, delete_proxy_server};
use controller::play_history_ctl::{get_play_history, page_play_history, add_play_history, update_play_history, cancel_pinned_play_history};
use controller::global_config_ctl::{get_global_config, list_all_global_config, add_global_config, update_global_config, delete_global_config};
use controller::emby_server_ctl::{get_emby_server, list_all_emby_server, add_emby_server, update_emby_server, defer_emby_server_order, update_emby_server_order, delete_emby_server};
use controller::emby_line_ctl::{get_emby_line, list_emby_server_line, list_all_emby_line, add_emby_line, update_emby_line, update_line_emby_server_name, delete_line_by_emby_server_id, delete_emby_line};
use controller::emby_icon_library_ctl::{get_emby_icon_library, list_all_emby_icon_library, add_emby_icon_library, update_emby_icon_library, delete_emby_icon_library};
use controller::invoke_ctl::{get_sys_info, play_video, http_forward, go_trakt_auth, open_url, updater, restart_app, get_runtime_config, clean_cache};
use config::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            trakt_http_token, trakt_http_get_user_info, trakt_http_start, trakt_http_stop,
            app_http_get_proxy_location, app_http_get_emby_icon_library,
            emby_get_server_info, emby_authenticate_by_name, emby_logout, emby_search, emby_get_continue_play_list, emby_get_favorite_list, emby_next_up, emby_get_media_library_list, emby_get_media_library_child_latest, emby_get_media_library_child, emby_count, emby_items, emby_seasons, emby_episodes, emby_playback_info, emby_playing, emby_playing_progress, emby_playing_stopped, emby_get_direct_stream_url, emby_get_audio_stream_url, emby_get_subtitle_stream_url, emby_get_image_url, emby_star, emby_unstar, emby_played, emby_unplayed, emby_hide_from_resume,
            get_proxy_server, list_all_proxy_server, add_proxy_server, update_proxy_server, delete_proxy_server,
            get_play_history, page_play_history, add_play_history, update_play_history, cancel_pinned_play_history,
            get_global_config, list_all_global_config, add_global_config, update_global_config, delete_global_config,
            get_emby_server, list_all_emby_server, add_emby_server, update_emby_server, defer_emby_server_order, update_emby_server_order, delete_emby_server,
            get_emby_line, list_emby_server_line, list_all_emby_line, add_emby_line, update_emby_line, update_line_emby_server_name, delete_line_by_emby_server_id, delete_emby_line,
            get_emby_icon_library, list_all_emby_icon_library, add_emby_icon_library, update_emby_icon_library, delete_emby_icon_library,
            get_sys_info, play_video, http_forward, go_trakt_auth, open_url, updater, restart_app, get_runtime_config, clean_cache
        ])
        .setup(|app| {
            let config_dir = app.path().resolve("", tauri::path::BaseDirectory::AppConfig)?;
            let config = config::app_config::get_config(app, &config_dir);
            if config.is_err() {
                panic!("Read Config error: {}", config.unwrap_err())
            }
            let config = config.unwrap();
            println!("Read Config: {:?}", &config);

            let local_data_dir = app.path().resolve("", tauri::path::BaseDirectory::AppLocalData)?;
            config::log::init(&local_data_dir, &config.log_level);

            let db_pool = tauri::async_runtime::block_on(config::db::init(config_dir))?;
            
            let axum_app_state = Arc::new(RwLock::new(None));
            let axum_app_state_clone = axum_app_state.clone();
            let app_handle = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let res = axum_svc::init_proxy_svc(axum_app_state_clone, app_handle).await;
                if res.is_err() {
                    tracing::error!("{:#?}", res);
                }
            });

            app.manage(AppState {
                app_config: config,
                auxm_app_state: axum_app_state,
                api_reqwest_pool: Arc::new(RwLock::new(HashMap::new())),
                image_reqwest_pool: Arc::new(RwLock::new(HashMap::new())),
                emby_server_chache: Arc::new(RwLock::new(HashMap::new())),
                global_config_chache: Arc::new(RwLock::new(HashMap::new())),
                proxy_server_chache: Arc::new(RwLock::new(HashMap::new())),
                db_pool,
            });

            tauri::async_runtime::block_on(mapper::emby_server_mapper::load_cache(&app.state()))?;
            tauri::async_runtime::block_on(mapper::global_config_mapper::load_cache(&app.state()))?;
            tauri::async_runtime::block_on(mapper::proxy_server_mapper::load_cache(&app.state()))?;
            
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build()).unwrap_or_else(|err| {
                tracing::error!("Updater plugin error: {}", err)
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
