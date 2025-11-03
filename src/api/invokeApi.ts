import { invoke } from '@tauri-apps/api/core';
import {RuntimeConfig} from "../store/runtimeConfig.ts";

async function getSysInfo(): Promise<string> {
    return invoke('get_sys_info');
}

interface PlayVideoParam {
    emby_server_id: string,
    series_id?: string,
    item_id: string,
    playback_position_ticks: number,
    use_direct_link: boolean,
    select_policy: string,
    video_select: number,
    audio_select: number,
    subtitle_select: number,
    version_select: number,
}

async function play_video(param: PlayVideoParam): Promise<string> {
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
    getSysInfo, play_video, go_trakt_auth, open_url, updater, restartApp, get_runtime_config, clean_emby_image_cache, clean_icon_cache
}
