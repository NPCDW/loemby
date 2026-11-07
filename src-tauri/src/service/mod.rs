pub mod player_svc;
pub mod mpv_ipc;
mod player_options;
pub mod axum_svc;
#[cfg(desktop)]
pub mod updater_svc;
pub mod cache_svc;
pub mod emby_http_svc;
pub mod app_http_svc;
pub mod trakt_http_svc;
pub mod yamtrack_http_svc;
pub mod simkl_http_svc;