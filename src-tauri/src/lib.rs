use tauri::Manager;

mod config;
mod util;

use config::{app_config::get_config_command, app_state::AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config_command
        ])
        .setup(|app| {
            let config = config::app_config::get_config(app);
            config::log::init(app, if config.is_err() { "info" } else { &config.as_ref().unwrap().log_level });
            if config.is_err() {
                tracing::error!("{:#?}", config);
                panic!("{}", config.unwrap_err())
            }
            tracing::debug!("Read Config: {:#?}", &config);
        
            app.manage(AppState {
                app_config: config.unwrap(),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
