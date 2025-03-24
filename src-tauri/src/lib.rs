use std::collections::HashMap;

use service::proxy_svc;
use tauri::{async_runtime::RwLock, Manager};

mod controller;
mod config;
mod service;
mod util;

use controller::invoke_ctl::{get_sys_info, play_video, http_forward, load_image};
use config::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let axum_app_state = match proxy_svc::init_proxy_svc().await {
        Ok(state) => state,
        Err(err) => panic!("axum init fail: {:#?}", err),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let window = app.webview_windows();
            let window = window.values().next().expect("Sorry, no window found");
            window.unminimize().expect("Sorry, no window unminimize");
            window.show().expect("Sorry, no window show");
            window.set_focus().expect("Can't Bring Window to Focus");
        }))
        .plugin(tauri_plugin_sql::Builder::default().add_migrations("sqlite:loemby.db", config::db_migrations::migrations()).build())
        .invoke_handler(tauri::generate_handler![get_sys_info, play_video, http_forward, load_image])
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

            *axum_app_state.app.blocking_write() = Some(app.app_handle().clone());

            app.manage(AppState {
                app_config: config,
                auxm_app_state: axum_app_state,
                reqwest_pool: RwLock::new(HashMap::new()),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
