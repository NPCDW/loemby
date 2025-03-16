import { defineStore } from 'pinia';
import { useDb } from '../db';

export const useEmbyServer = defineStore('embyServer', () => {
    async function getEmbyServer(id: string) {
        let embyServer = await useDb().db?.select<EmbyServer[]>('select * from emby_server where id = $1', [id]);
        if (!embyServer || embyServer.length == 0) {
            return;
        }
        return embyServer[0];
    }

    async function getMaxOrderBy() {
        let embyServer = await useDb().db?.select<{max_order_by: number}[]>('select max(order_by) as max_order_by from emby_server');
        if (!embyServer || embyServer.length == 0) {
            return;
        }
        return embyServer[0].max_order_by;
    }

    async function listAllEmbyServer() {
        let embyServer = await useDb().db?.select<EmbyServer[]>('select * from emby_server');
        if (!embyServer || embyServer.length == 0) {
            return [];
        }
        return embyServer;
    }

    async function addEmbyServer(embyServer: EmbyServer) {
        let fields: string[] = [], values: string[] = [];
        for (const [key, value] of Object.entries(embyServer)) {
            if (value != null && value != undefined && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `insert into emby_server (${fields.join(',')}) values (${fields.map((_item, index) => '$' + (index + 1)).join(',')})`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updateEmbyServer(embyServer: EmbyServer) {
        let fields: string[] = [], values: string[] = [];
        values.push(embyServer.id!);
        for (const [key, value] of Object.entries(embyServer)) {
            if (value != null && value != undefined && key != 'id' && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        console.log(embyServer)
        let sql = `update emby_server set ${fields.map((item, index) => item + ' = $' + (index + 2)).join(',')} where id = $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updateOrder(orderNumber: number) {
        let values: number[] = [];
        values.push(orderNumber);
        let sql = `update emby_server set order_by = order_by + 1 where order_by >= $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function delEmbyServer(id: string) {
        let res = await useDb().db?.execute('delete from emby_server where id = $1', [id]);
        return res?.rowsAffected;
    }

    return { getEmbyServer, delEmbyServer, addEmbyServer, updateEmbyServer, listAllEmbyServer, updateOrder, getMaxOrderBy }
})

export interface EmbyServer {
    id?: string,
    create_time?: string,

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

    order_by?: number,

    browse_proxy_id?: string,
    play_proxy_id?: string,

    last_playback_time?: string,
    keep_alive_days?: number,

    disabled?: number,
}
