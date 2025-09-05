import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { useGlobalConfig } from './globalConfig';
import { ref } from 'vue';

export const useProxyServer = defineStore('proxyServer', () => {
    const cacheProxyServer = ref<{[key: string]: ProxyServer}>({});

    async function refreshCache(id: string) {
        let server = await getProxyServer(id)
        if (!server) {
            cacheProxyServer.value[id] = {};
            return;
        }
        cacheProxyServer.value[id] = server;
    }

    async function initCache() {
        let proxyServer = await listAllProxyServer();
        cacheProxyServer.value = {};
        for (let i = 0; i < proxyServer.length; i++) {
            cacheProxyServer.value[proxyServer[i].id!] = proxyServer[i];
        }
    }

    async function getProxyServer(id: string): Promise<ProxyServer> {
        return invoke('get_proxy_server', {id});
    }

    async function listAllProxyServer(): Promise<ProxyServer[]> {
        return invoke('list_all_proxy_server');
    }

    async function addProxyServer(proxyServer: ProxyServer): Promise<number> {
        return invoke('add_proxy_server', {body: proxyServer});
    }

    async function updateProxyServer(proxyServer: ProxyServer): Promise<number> {
        return invoke('update_proxy_server', {body: proxyServer});
    }

    async function delProxyServer(id: string): Promise<number> {
        return invoke('delete_proxy_server', {id: id});
    }

    async function getProxyServerName(id: string) {
        if (id == 'no') {
            return '不使用代理'
        }
        if (!cacheProxyServer.value[id]) {
            await refreshCache(id);
        }
        let proxyServer = cacheProxyServer.value[id]
        return proxyServer.name!
    }

    async function getBrowseProxyUrl(id?: string) {
        if (!id || id == 'follow') {
            return useGlobalConfig().getGlobalConfigValue('global_browse_proxy_id').then(value => {
                if (value) {
                    return getProxyUrl(value)
                }
                return
            })
        }
        return getProxyUrl(id)
    }

    async function getPlayProxyUrl(id?: string) {
        if (!id || id == 'follow') {
            return useGlobalConfig().getGlobalConfigValue('global_play_proxy_id').then(value => {
                if (value) {
                    return getProxyUrl(value)
                }
                return
            })
        }
        return getProxyUrl(id)
    }

    async function getTraktProxyUrl() {
        return useGlobalConfig().getGlobalConfigValue('trakt_proxy_id').then(value => {
            if (!value || value == 'followBrowse') {
                return useGlobalConfig().getGlobalConfigValue('global_browse_proxy_id').then(value => {
                    if (value) {
                        return getProxyUrl(value)
                    }
                    return
                })
            }
            if (value == 'followPlay') {
                return useGlobalConfig().getGlobalConfigValue('global_play_proxy_id').then(value => {
                    if (value) {
                        return getProxyUrl(value)
                    }
                    return
                })
            }
            return getProxyUrl(value)
        })
    }

    async function getAppProxyUrl() {
        return useGlobalConfig().getGlobalConfigValue('app_proxy_id').then(value => {
            if (!value || value == 'followBrowse') {
                return useGlobalConfig().getGlobalConfigValue('global_browse_proxy_id').then(value => {
                    if (value) {
                        return getProxyUrl(value)
                    }
                    return
                })
            }
            if (value == 'followPlay') {
                return useGlobalConfig().getGlobalConfigValue('global_play_proxy_id').then(value => {
                    if (value) {
                        return getProxyUrl(value)
                    }
                    return
                })
            }
            return getProxyUrl(value)
        })
    }

    async function getProxyUrl(id: string) {
        if (id == 'no') {
            return
        }
        if (!cacheProxyServer.value[id]) {
            await refreshCache(id);
        }
        let proxyServer = cacheProxyServer.value[id]
        let username = proxyServer.username ? proxyServer.username : ""
        let password = proxyServer.password ? ":" + proxyServer.password : ""
        let auth = username || password ? username + password + "@" : ""
        return proxyServer.proxy_type + "://" + auth + proxyServer.addr
    }

    return { getProxyServer, delProxyServer, addProxyServer, updateProxyServer, listAllProxyServer, getBrowseProxyUrl, getPlayProxyUrl, getTraktProxyUrl, getProxyUrl, getAppProxyUrl, initCache, refreshCache, getProxyServerName }
})

export interface ProxyServer {
    id?: string,
    create_time?: string,

    name?: string,
    proxy_type?: string,
    addr?: string,
    username?: string,
    password?: string,

    // 前端展示字段
    location?: string,
}
