import { invoke } from '@tauri-apps/api/core';

async function getSysInfo(): Promise<string> {
    return invoke('get_sys_info');
}

interface InvokePlayback {
    mpv_path: string,
    path: string,
    proxy?: string,
    title: string,
    user_agent: string,
    server_id: string,
    item_id: string,
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
}

async function playback(param: InvokePlayback): Promise<string> {
    return invoke('play_video', {body: param});
}

interface HttpForwardParam {
    method: string,
    url: string,
    headers: {[key: string]: string},
    body?: string,
    proxy?: string,
}

interface HttpForwardResult {
    status_code: number,
    status_text: string,
    headers: {[key: string]: string},
    body: string,
}

async function httpForward(param: HttpForwardParam): Promise<HttpForwardResult> {
    return invoke('http_forward', {param});
}

interface LoadImageParam {
    image_url: String,
    proxy_url?: String,
    user_agent: String,
}

async function loadImage(param: LoadImageParam): Promise<string> {
    return invoke('load_image', {body: param});
}

async function go_trakt_auth(): Promise<void> {
    return invoke('go_trakt_auth');
}

async function open_url(url: string): Promise<string> {
    return invoke('open_url', {url: url});
}

export default {
    getSysInfo, playback, httpForward, loadImage, go_trakt_auth, open_url
}
