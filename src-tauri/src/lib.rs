use tauri::{async_runtime::RwLock, Manager};

mod config;
mod util;

use config::{
    app_config::{get_config_command, save_config},
    app_state::AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![get_config_command, save_config])
        .setup(|app| {
            let debug = std::env::var("TAURI_ENV_DEBUG").unwrap_or("false".to_string()).parse::<bool>().unwrap_or(false);
            let root_dir = app.path().resolve(
                format!("loemby{}/", if debug { "-dev" } else { "" }),
                tauri::path::BaseDirectory::AppLocalData,
            )?;

            config::log::init(&root_dir, "info");
            let config = config::app_config::get_config(app, &root_dir);
            if config.is_err() {
                tracing::error!("{:#?}", config);
                panic!("{}", config.unwrap_err())
            }
            tracing::debug!("Read Config: {:#?}", &config);

            app.manage(AppState {
                app_config: RwLock::new(config.unwrap()),
                app: app.app_handle().clone(),
                debug,
                root_dir
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
