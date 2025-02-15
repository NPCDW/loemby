import invoke from '../api/invoke'
import _ from 'lodash';
import { defineStore } from 'pinia'
import { ref } from "vue";

export const useConfig = defineStore('config', () => {
    const config = ref<Config>()

    async function get_config() {
        config.value = await invoke.getConfig();
        return config.value
    }

    async function save_config(tmp: Config) {
        config.value = _.cloneDeep(tmp);
        await invoke.saveConfig({ config: config.value! });
    }

    return { get_config, save_config }
})

interface ProxyServerConfig {
    id: string,
    proxy_type: string,
    addr: string,
    username?: string,
    password?: string,
}

export interface EmbyServerConfig {
    id: string,

    base_url: string,
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

    proxy_id?: string,

    disabled: boolean,
    // 前端状态字段
    request_status?: boolean,
    request_fail?: boolean,
}

export interface Config {
    log_level: string,
    emby_server: EmbyServerConfig[],
    proxy_server: ProxyServerConfig[],
}
