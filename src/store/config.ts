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

    function getEmbyServer(id: string) {
        if (!config.value.emby_server) {
            return
        }
        for (let index = 0; index < config.value.emby_server.length; index++) {
            if (config.value.emby_server[index].id === id) {
                return config.value.emby_server[index]
            }
        }
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

    function getBrowseProxyUrl(id?: string) {
        if (!id || id == 'follow') {
            if (config.value.global_proxy && config.value.global_proxy.browse_proxy_id) {
                return getProxyUrl(config.value.global_proxy.browse_proxy_id)
            }
            return
        }
        return getProxyUrl(id)
    }

    function getPlayProxyUrl(id?: string) {
        if (!id || id == 'follow') {
            if (config.value.global_proxy && config.value.global_proxy.play_proxy_id) {
                return getProxyUrl(config.value.global_proxy.play_proxy_id)
            }
            return
        }
        return getProxyUrl(id)
    }

    function getProxyUrl(id: string) {
        if (id == 'no' || !config.value.proxy_server) {
            return
        }
        for (let index = 0; index < config.value.proxy_server.length; index++) {
            if (config.value.proxy_server[index].id === id) {
                let proxy = config.value.proxy_server[index]
                let username = proxy.username ? proxy.username : ""
                let password = proxy.password ? ":" + proxy.password : ""
                let auth = username || password ? username + password + "@" : ""
                return proxy.proxy_type + "://" + auth + proxy.addr
            }
        }
    }

    return { get_config, sync_config, save_config, getEmbyServer, saveEmbyServer, delEmbyServer, getBrowseProxyUrl, getPlayProxyUrl, getProxyUrl }
})

export interface ProxyServerConfig {
    id?: string,
    name?: string,
    proxy_type?: string,
    addr?: string,
    username?: string,
    password?: string,
    // 前端展示字段
    location?: string,
}

export interface ServerLine {
    id?: string,
    name?: string,
    base_url?: string,
    using?: boolean,
    browse_proxy_id?: string,
    play_proxy_id?: string,
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

    browse_proxy_id?: string,
    play_proxy_id?: string,

    line?: ServerLine[],

    disabled?: boolean,
    // 前端状态字段
    request_status?: boolean,
    request_fail?: boolean,
}

export interface GlobalProxy {
    browse_proxy_id?: string,
    play_proxy_id?: string,
}

export interface Config {
    log_level?: string,
    mpv_path?: string,
    emby_server?: EmbyServerConfig[],
    proxy_server?: ProxyServerConfig[],
    global_proxy?: GlobalProxy,
}
