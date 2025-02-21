import { Config } from "../store/config";
import { invoke } from '@tauri-apps/api/core';

async function getConfig(): Promise<Config> {
    return invoke('get_config_command');
}

async function saveConfig(config: Config) {
    return invoke('save_config', {config});
}

async function playback(path: string, serverId: string, itemId: string): Promise<string> {
    return invoke('play_video', {path, serverId, itemId});
}

export default {
    getConfig, saveConfig, playback
}
