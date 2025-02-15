import { Config } from "../store/config";

const invoke = window.__TAURI__.core.invoke;

async function getConfig() {
    return invoke('get_config_command');
}

async function saveConfig(config: {config: Config}) {
    return invoke('save_config', config);
}

export default {
    getConfig, saveConfig
}
