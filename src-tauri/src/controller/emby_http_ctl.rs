use serde::{Deserialize, Serialize};

use crate::{config::app_state::AppState, service::emby_http_svc};


#[derive(Serialize, Deserialize)]
pub struct EmbyGetServerInfoParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_get_server_info(body: EmbyGetServerInfoParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_server_info(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyAuthenticateByNameParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_authenticate_by_name(body: EmbyAuthenticateByNameParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::authenticate_by_name(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyLogoutParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_logout(body: EmbyLogoutParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::logout(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbySearchParam {
    pub emby_server_id: String,
    pub search_str: String,
    pub item_types: Vec<String>,
    pub start_index: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_search(body: EmbySearchParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::search(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetContinuePlayListParam {
    pub emby_server_id: String,
    pub start_index: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_get_continue_play_list(body: EmbyGetContinuePlayListParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_continue_play_list(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetFavoriteListParam {
    pub emby_server_id: String,
    pub start_index: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_get_favorite_list(body: EmbyGetFavoriteListParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_favorite_list(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyNextUpParam {
    pub emby_server_id: String,
    pub series_id: String,
    pub start_index: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_next_up(body: EmbyNextUpParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::next_up(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetMediaLibraryListParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_get_media_library_list(body: EmbyGetMediaLibraryListParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_media_library_list(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetMediaLibraryChildLatestParam {
    pub emby_server_id: String,
    pub parent_id: String,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_get_media_library_child_latest(body: EmbyGetMediaLibraryChildLatestParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_media_library_child_latest(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetMediaLibraryChildParam {
    pub emby_server_id: String,
    pub parent_id: String,
    pub start_index: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_get_media_library_child(body: EmbyGetMediaLibraryChildParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_media_library_child(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyCountParam {
    pub emby_server_id: String,
}

#[tauri::command]
pub async fn emby_count(body: EmbyCountParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::count(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyItemsParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_items(body: EmbyItemsParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::items(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbySeasonsParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_seasons(body: EmbySeasonsParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::seasons(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyEpisodesParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub season_id: String,
    pub start_index: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn emby_episodes(body: EmbyEpisodesParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::episodes(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyPlaybackInfoParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_playback_info(body: EmbyPlaybackInfoParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::playback_info(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetDirectStreamUrlParam {
    pub emby_server_id: String,
    pub direct_stream_url: String,
}

#[tauri::command]
pub async fn emby_get_direct_stream_url(body: EmbyGetDirectStreamUrlParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_direct_stream_url(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetAudioStreamUrlParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_item_id: Option<String>,
    pub media_streams_codec: String,
    pub media_streams_index: String,
    pub media_streams_is_external: bool,
}

#[tauri::command]
pub async fn emby_get_audio_stream_url(body: EmbyGetAudioStreamUrlParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_audio_stream_url(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyGetSubtitleStreamUrlParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub media_source_id: String,
    pub media_source_item_id: Option<String>,
    pub media_streams_codec: String,
    pub media_streams_index: String,
    pub media_streams_is_external: bool,
}

#[tauri::command]
pub async fn emby_get_subtitle_stream_url(body: EmbyGetSubtitleStreamUrlParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::get_subtitle_stream_url(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyStarParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_star(body: EmbyStarParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::star(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyUnstarParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_unstar(body: EmbyUnstarParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::unstar(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyPlayedParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_played(body: EmbyPlayedParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::played(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyUnplayedParam {
    pub emby_server_id: String,
    pub item_id: String,
}

#[tauri::command]
pub async fn emby_unplayed(body: EmbyUnplayedParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::unplayed(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct EmbyHideFromResumeParam {
    pub emby_server_id: String,
    pub item_id: String,
    pub hide: bool,
}

#[tauri::command]
pub async fn emby_hide_from_resume(body: EmbyHideFromResumeParam, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let res = emby_http_svc::hide_from_resume(body, &state).await;
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }
    Ok(res.unwrap())
}
