import { Config } from "../store/config";
import { invoke } from '@tauri-apps/api/core';

async function getConfig(): Promise<Config> {
    return invoke('get_config');
}

async function saveConfig(config: Config) {
    return invoke('save_config', {config});
}

interface InvokePlayback {
    path: string,
    title: string,
    server_id: string,
    item_id: string,
    media_source_id: string,
    play_session_id: string,
    playback_position_ticks: number,
    aid: number,
    sid: number,
    external_audio: string[],
    external_subtitle: string[],
}

async function playback(param: InvokePlayback): Promise<string> {
    return invoke('play_video', {body: param});
}

interface HttpForwardParam {
    method: string,
    url: string,
    headers: {[key: string]: string},
    body?: string,
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

export default {
    getConfig, saveConfig, playback, httpForward
}
