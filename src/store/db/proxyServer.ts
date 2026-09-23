import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { waitUntilTrue } from '../../util/sleep';

export const useProxyServer = defineStore('proxyServer', () => {
    const cacheProxyServer = ref<{[key: string]: string}>({});
    const initCacheFinish = ref(false)

    async function getProxyServerName(id: string) {
        if (!id || id == 'no') {
            return '不使用代理'
        }
        if (!initCacheFinish.value) {
            await waitUntilTrue(() => initCacheFinish.value, 100)
        }
        let proxyServerName = cacheProxyServer.value[id]
        return proxyServerName || "不使用代理"
    }

    async function refreshCache(id: string) {
        let server = await getProxyServer(id)
        if (!server) {
            cacheProxyServer.value[id] = '';
            return;
        }
        cacheProxyServer.value[id] = server.name!;
    }

    async function initCache() {
        let proxyServer = await listAllProxyServer();
        cacheProxyServer.value = {};
        for (let i = 0; i < proxyServer.length; i++) {
            cacheProxyServer.value[proxyServer[i].id!] = proxyServer[i].name!;
        }
        initCacheFinish.value = true
    }

    async function getProxyServer(id: string): Promise<ProxyServer> {
        return invoke('get_proxy_server', {id});
    }

    async function listAllProxyServer(): Promise<ProxyServer[]> {
        return invoke('list_all_proxy_server');
    }

    async function addProxyServer(proxyServer: ProxyServer): Promise<unknown> {
        return invoke('add_proxy_server', {body: proxyServer}).then(async response => {
            await refreshCache(proxyServer.id!)
            return response
        })
    }

    async function updateProxyServer(proxyServer: ProxyServer): Promise<unknown> {
        return invoke('update_proxy_server', {body: proxyServer}).then(async response => {
            await refreshCache(proxyServer.id!)
            return response
        })
    }

    async function delProxyServer(id: string): Promise<unknown> {
        return invoke('delete_proxy_server', {id: id}).then(async response => {
            await refreshCache(id)
            return response
        })
    }

    return { getProxyServer, delProxyServer, addProxyServer, updateProxyServer, listAllProxyServer, initCache, refreshCache, getProxyServerName }
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
