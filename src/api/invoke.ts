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
    serverId: string,
    itemId: string,
    mediaSourceId: string,
    playSessionId: string,
    playbackPositionTicks: number,
    aid: number,
    sid: number,
    externalAudio: string[],
    externalSubtitle: string[],
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
