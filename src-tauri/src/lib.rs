use std::{collections::HashMap, sync::Arc};

use service::proxy_svc;
use tauri::{async_runtime::RwLock, Manager};

mod controller;
mod config;
mod service;
mod util;

use controller::invoke_ctl::{get_sys_info, play_video, http_forward, go_trakt_auth, open_url, updater, restart_app, get_runtime_config};
use config::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let window = app.webview_windows();
            let window = window.values().next().expect("Sorry, no window found");
            window.unminimize().expect("Sorry, no window unminimize");
            window.show().expect("Sorry, no window show");
            window.set_focus().expect("Can't Bring Window to Focus");
        }))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().add_migrations("sqlite:loemby.db", config::db_migrations::migrations()).build())
        .invoke_handler(tauri::generate_handler![get_sys_info, play_video, http_forward, go_trakt_auth, open_url, updater, restart_app, get_runtime_config])
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

            let axum_app_state = Arc::new(RwLock::new(None));
            let axum_app_state_clone = axum_app_state.clone();
            let app_handle = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let res = proxy_svc::init_proxy_svc(axum_app_state_clone, app_handle).await;
                if res.is_err() {
                    tracing::error!("{:#?}", res);
                }
            });

            app.manage(AppState {
                app_config: config,
                auxm_app_state: axum_app_state,
                api_reqwest_pool: RwLock::new(HashMap::new()),
                image_reqwest_pool: RwLock::new(HashMap::new()),
            });
            
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build()).unwrap_or_else(|err| {
                tracing::error!("Updater plugin error: {}", err)
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
