import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { waitUntilTrue } from '../../util/sleep';

export const useReverseProxyServer = defineStore('reverseProxyServer', () => {
    const cacheReverseProxyServer = ref<{[key: string]: string}>({});
    const initCacheFinish = ref(false)

    async function getReverseProxyServerName(id: string) {
        if (!id || id == 'no') {
            return '不使用反代';
        }
        if (!initCacheFinish.value) {
            await waitUntilTrue(() => initCacheFinish.value, 100)
        }
        let name = cacheReverseProxyServer.value[id]
        return name || "不使用反代"
    }

    async function refreshCache(id: string) {
        let server = await getReverseProxyServer(id)
        if (!server) {
            cacheReverseProxyServer.value[id] = '';
            return;
        }
        cacheReverseProxyServer.value[id] = server.name!;
    }

    async function initCache() {
        let list = await listAllReverseProxyServer();
        cacheReverseProxyServer.value = {};
        for (let i = 0; i < list.length; i++) {
            cacheReverseProxyServer.value[list[i].id!] = list[i].name!;
        }
        initCacheFinish.value = true
    }

    async function getReverseProxyServer(id: string): Promise<ReverseProxyServer> {
        return invoke('get_reverse_proxy_server', {id});
    }

    async function listAllReverseProxyServer(): Promise<ReverseProxyServer[]> {
        return invoke('list_all_reverse_proxy_server');
    }

    async function addReverseProxyServer(reverseProxyServer: ReverseProxyServer): Promise<unknown> {
        return invoke('add_reverse_proxy_server', {body: reverseProxyServer}).then(async response => {
            await refreshCache(reverseProxyServer.id!)
            return response
        })
    }

    async function updateReverseProxyServer(reverseProxyServer: ReverseProxyServer): Promise<unknown> {
        return invoke('update_reverse_proxy_server', {body: reverseProxyServer}).then(async response => {
            await refreshCache(reverseProxyServer.id!)
            return response
        })
    }

    async function delReverseProxyServer(id: string): Promise<unknown> {
        return invoke('delete_reverse_proxy_server', {id: id}).then(async response => {
            await refreshCache(id)
            return response
        })
    }

    return { getReverseProxyServer, delReverseProxyServer, addReverseProxyServer, updateReverseProxyServer, listAllReverseProxyServer, initCache, refreshCache, getReverseProxyServerName }
})

export interface ReverseProxyServer {
    id?: string,
    create_time?: string,

    name?: string,
    url?: string,
}
