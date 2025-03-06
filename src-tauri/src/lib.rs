use std::{collections::HashMap, sync::Arc};

use service::proxy_svc;
use tauri::{async_runtime::RwLock, Manager};

mod controller;
mod config;
mod service;
mod util;

use controller::invoke_ctl::{get_config, save_config, play_video, http_forward, load_image};
use config::app_state::AppState;

#[cfg(debug_assertions)]
fn is_development() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn is_development() -> bool {
    false
}

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
        .invoke_handler(tauri::generate_handler![get_config, save_config, play_video, http_forward, load_image])
        .setup(|app| {
            let root_dir = if is_development() {
                &format!("loemby-{}", std::env::var("TAURI_DEV_MODE").unwrap_or("dev".to_string()))
            } else {
                "loemby"
            };
            println!("TAURI_DEV_MODE: {:?}, root_dir: {:?}", std::env::var("TAURI_DEV_MODE"), &root_dir);
            let root_dir = app.path().resolve(root_dir, tauri::path::BaseDirectory::AppLocalData)?;

            config::log::init(&root_dir, is_development());
            let config = config::app_config::get_config(app, &root_dir);
            if config.is_err() {
                tracing::error!("{:#?}", config);
                panic!("{}", config.unwrap_err())
            }
            tracing::debug!("Read Config: {:?}", &config);

            let axum_app_state = Arc::new(RwLock::new(None));

            let axum_app_state_clone = axum_app_state.clone();
            tauri::async_runtime::spawn(async move {
                let res = proxy_svc::init_proxy_svc(axum_app_state_clone).await;
                if res.is_err() {
                    tracing::error!("{:#?}", res);
                }
            });

            app.manage(AppState {
                app_config: RwLock::new(config.unwrap()),
                auxm_app_state: axum_app_state,
                reqwest_pool: RwLock::new(HashMap::new()),
                root_dir
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
