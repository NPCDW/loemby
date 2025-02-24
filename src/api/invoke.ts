import { Config } from "../store/config";
import { invoke } from '@tauri-apps/api/core';

async function getConfig(): Promise<Config> {
    return invoke('get_config_command');
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
    return invoke('play_video', {...param});
}

export default {
    getConfig, saveConfig, playback
}
