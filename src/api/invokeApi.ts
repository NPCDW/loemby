import { invoke } from '@tauri-apps/api/core';
import {RuntimeConfig} from "../store/runtimeConfig.ts";

async function getSysInfo(): Promise<string> {
    return invoke('get_sys_info');
}

interface PlaybackParam {
    path: string,
    title: string,
    item_id: string,
    item_type: string,
    item_name: string,
    emby_server_id: string,
    emby_server_name: string,
    series_id?: string,
    series_name?: string,
    media_source_id: string,
    play_session_id: string,
    playback_position_ticks: number,
    run_time_ticks: number,
    bitrate?: number,
    vid: number,
    aid: number,
    sid: number,
    external_audio: string[],
    external_subtitle: string[],
    scrobble_trakt_param?: string,
    start_time: number,
    track_titles: string,
}

async function playback(param: PlaybackParam): Promise<string> {
    return invoke('play_video', {body: param});
}

async function go_trakt_auth(): Promise<void> {
    return invoke('go_trakt_auth');
}

async function open_url(url: string): Promise<string> {
    return invoke('open_url', {url: url});
}

async function updater(): Promise<boolean> {
    return invoke('updater');
}

async function restartApp(): Promise<boolean> {
    return invoke('restart_app', {});
}

async function get_runtime_config(): Promise<RuntimeConfig> {
    return invoke('get_runtime_config', {});
}

async function clean_emby_image_cache(emby_server_id?: string): Promise<void> {
    return invoke('clean_emby_image_cache', {body: {emby_server_id}});
}

async function clean_icon_cache(): Promise<void> {
    return invoke('clean_icon_cache');
}

export default {
    getSysInfo, playback, go_trakt_auth, open_url, updater, restartApp, get_runtime_config, clean_emby_image_cache, clean_icon_cache
}
