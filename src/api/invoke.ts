import { Config } from "../store/config";
import { invoke } from '@tauri-apps/api/core';

async function getConfig(): Promise<Config> {
    return invoke('get_config_command');
}

async function saveConfig(config: {config: Config}) {
    return invoke('save_config', config);
}

export default {
    getConfig, saveConfig
}
