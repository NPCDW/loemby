import invoke from '../api/invoke'
import _ from 'lodash';
import { defineStore } from 'pinia'
import { ref } from "vue";

export const useConfig = defineStore('config', () => {
    const config = ref<Config>({})

    function get_config() {
        return config.value
    }

    async function sync_config() {
        config.value = await invoke.getConfig();
        return config.value
    }

    async function save_config(tmp: Config) {
        config.value = _.cloneDeep(tmp);
        await invoke.saveConfig(config.value!);
    }

    async function saveEmbyServer(embyServers: EmbyServerConfig[]) {
        config.value.emby_server = _.cloneDeep(embyServers);
        await save_config(config.value)
    }

    async function delEmbyServer(id: string) {
        if (!config.value.emby_server) {
            return
        }
        for (let index = 0; index < config.value.emby_server.length; index++) {
            if (config.value.emby_server[index].id === id) {
                config.value.emby_server.splice(index, 1)
                await save_config(config.value)
                return
            }
        }
    }

    return { get_config, sync_config, save_config, saveEmbyServer, delEmbyServer }
})

interface ProxyServerConfig {
    id?: string,
    proxy_type?: string,
    addr?: string,
    username?: string,
    password?: string,
}

export interface EmbyServerConfig {
    id?: string,

    base_url?: string,
    username?: string,
    password?: string,

    server_name?: string,
    server_id?: string,
    auth_token?: string,
    user_id?: string,

    client?: string,
    device?: string,
    device_id?: string,
    client_version?: string,
    user_agent?: string,

    proxy_id?: string,

    disabled?: boolean,
    // 前端状态字段
    request_status?: boolean,
    request_fail?: boolean,
}

export interface Config {
    log_level?: string,
    mpv_path?: string,
    emby_server?: EmbyServerConfig[],
    proxy_server?: ProxyServerConfig[],
}
