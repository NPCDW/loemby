import { invoke } from '@tauri-apps/api/core';
import { useProxyServer } from '../store/db/proxyServer';
import {RuntimeConfig, useRuntimeConfig} from "../store/runtimeConfig.ts";

async function getSysInfo(): Promise<string> {
    return invoke('get_sys_info');
}

interface PlaybackParam {
    mpv_path: string,
    mpv_startup_dir?: string,
    mpv_args?: string,
    mpv_cache_max_bytes?: number,
    mpv_cache_back_max_bytes?: number,
    path: string,
    proxy?: string,
    title: string,
    user_agent: string,
    server_id: string,
    item_id: string,
    item_name: string,
    series_id?: string,
    series_name?: string,
    media_source_id: string,
    play_session_id: string,
    playback_position_ticks: number,
    run_time_ticks: number,
    vid: number,
    aid: number,
    sid: number,
    external_audio: string[],
    external_subtitle: string[],
    scrobble_trakt_param?: string,
    start_time: number,
}

async function playback(param: PlaybackParam): Promise<string> {
    return invoke('play_video', {body: param});
}

interface HttpForwardParam {
    method: string,
    url: string,
    headers: {[key: string]: string},
    body?: string,
    proxy?: string,
}

export interface HttpForwardResult {
    status_code: number,
    status_text: string,
    headers: {[key: string]: string},
    body: string,
}

async function httpForward(param: HttpForwardParam): Promise<HttpForwardResult> {
    return invoke('http_forward', {param});
}

interface LoadImageParam {
    image_url: string,
    proxy_url?: string,
    user_agent: string,
    cache_prefix: string[],
}

function loadImage(param: LoadImageParam): string {
    let port = useRuntimeConfig().runtimeConfig!.axum_port;
    let url = `http://127.0.0.1:${port}/image?image_url=${encodeURIComponent(param.image_url)}&user_agent=${encodeURIComponent(param.user_agent)}&cache_prefix=${encodeURIComponent(param.cache_prefix.join("/"))}`;
    if (param.proxy_url) {
        url += `&proxy_url=${encodeURIComponent(param.proxy_url)}`
    }
    return url
}

async function go_trakt_auth(): Promise<void> {
    return invoke('go_trakt_auth');
}

async function open_url(url: string): Promise<string> {
    return invoke('open_url', {url: url});
}

async function updater(): Promise<boolean> {
    return invoke('updater', {body: {
        proxy_url: await useProxyServer().getAppProxyUrl(),
        user_agent: 'loemby/' + import.meta.env.VITE_APP_VERSION,
    }});
}

async function restartApp(): Promise<boolean> {
    return invoke('restart_app', {});
}

async function get_runtime_config(): Promise<RuntimeConfig> {
    return invoke('get_runtime_config', {});
}

export default {
    getSysInfo, playback, httpForward, loadImage, go_trakt_auth, open_url, updater, restartApp, get_runtime_config
}
