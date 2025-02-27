use tauri::{async_runtime::RwLock, Manager};

mod controller;
mod config;
mod service;
mod util;

use controller::invoke_ctl::{get_config, save_config, play_video, http_forward};
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
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_config, save_config, play_video, http_forward])
        .setup(|app| {
            let root_dir = app.path().resolve(
                format!("loemby{}/", if is_development() { "-dev" } else { "" }),
                tauri::path::BaseDirectory::AppLocalData,
            )?;

            config::log::init(&root_dir, is_development());
            let config = config::app_config::get_config(app, &root_dir);
            if config.is_err() {
                tracing::error!("{:#?}", config);
                panic!("{}", config.unwrap_err())
            }
            tracing::debug!("Read Config: {:#?}", &config);

            app.manage(AppState {
                app_config: RwLock::new(config.unwrap()),
                app: app.app_handle().clone(),
                debug: is_development(),
                root_dir
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
