import { Config } from "../store/config";
import { invoke } from '@tauri-apps/api/core';

async function getConfig(): Promise<Config> {
    return invoke('get_config_command');
}

async function saveConfig(config: Config) {
    return invoke('save_config', {config});
}

async function playback(path: string, playback_id: string): Promise<string> {
    return invoke('play_video', {path, id: playback_id});
}

export default {
    getConfig, saveConfig, playback
}
