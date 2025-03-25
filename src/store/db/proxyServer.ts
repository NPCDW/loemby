import { defineStore } from 'pinia';
import { useDb } from '../db';
import { useGlobalConfig } from './globalConfig';
import { ref } from 'vue';

export const useProxyServer = defineStore('proxyServer', () => {
    const cacheProxyServer = ref<{[key: string]: ProxyServer}>({});

    async function refreshCache() {
        let proxyServer = await listAllProxyServer();
        cacheProxyServer.value = {};
        for (let i = 0; i < proxyServer.length; i++) {
            cacheProxyServer.value[proxyServer[i].id!] = proxyServer[i];
        }
    }

    async function getProxyServer(id: string) {
        let proxyServer = await useDb().db?.select<ProxyServer[]>('select * from proxy_server where id = $1', [id]);
        if (!proxyServer || proxyServer.length == 0) {
            return;
        }
        return proxyServer[0];
    }

    async function listAllProxyServer() {
        let proxyServer = await useDb().db?.select<ProxyServer[]>('select * from proxy_server');
        if (!proxyServer || proxyServer.length == 0) {
            return [];
        }
        return proxyServer;
    }

    async function addProxyServer(proxyServer: ProxyServer) {
        let fields: string[] = [], values: string[] = [];
        for (const [key, value] of Object.entries(proxyServer)) {
            if (value != null && value != undefined && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `insert into proxy_server (${fields.join(',')}) values (${fields.map((_item, index) => '$' + (index + 1)).join(',')})`;
        let res = await useDb().db?.execute(sql, values);
        await refreshCache();
        return res?.rowsAffected;
    }

    async function updateProxyServer(proxyServer: ProxyServer) {
        let fields: string[] = [], values: string[] = [];
        values.push(proxyServer.id!);
        for (const [key, value] of Object.entries(proxyServer)) {
            if (value != null && value != undefined && key != 'id' && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `update proxy_server set ${fields.map((item, index) => item + ' = $' + (index + 2)).join(',')} where id = $1`;
        let res = await useDb().db?.execute(sql, values);
        await refreshCache();
        return res?.rowsAffected;
    }

    async function delProxyServer(id: string) {
        let res = await useDb().db?.execute('delete from proxy_server where id = $1', [id]);
        await refreshCache();
        return res?.rowsAffected;
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
            if (value) {
                return getProxyUrl(value)
            }
            return
        })
    }

    async function getProxyUrl(id: string) {
        if (id == 'no') {
            return
        }
        if (!cacheProxyServer.value[id]) {
            await refreshCache();
        }
        let proxyServer = cacheProxyServer.value[id]
        let username = proxyServer.username ? proxyServer.username : ""
        let password = proxyServer.password ? ":" + proxyServer.password : ""
        let auth = username || password ? username + password + "@" : ""
        return proxyServer.proxy_type + "://" + auth + proxyServer.addr
    }

    return { getProxyServer, delProxyServer, addProxyServer, updateProxyServer, listAllProxyServer, getBrowseProxyUrl, getPlayProxyUrl, getTraktProxyUrl, getProxyUrl }
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
