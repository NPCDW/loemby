import { defineStore } from 'pinia';
import { useDb } from '../db';

export const useEmbyLine = defineStore('embyLine', () => {
    async function getEmbyLine(id: string) {
        let embyLine = await useDb().db?.select<EmbyLine[]>('select * from emby_line where id = $1', [id]);
        if (!embyLine || embyLine.length == 0) {
            return;
        }
        return embyLine[0];
    }

    async function getUsingEmbyLine(emby_server_id: string) {
        let embyLine = await useDb().db?.select<EmbyLine[]>('select * from emby_line where emby_server_id = $1 and in_use = 1', [emby_server_id]);
        if (!embyLine || embyLine.length == 0) {
            return;
        }
        return embyLine[0];
    }

    async function listEmbyLine(emby_server_id: string) {
        let embyLine = await useDb().db?.select<EmbyLine[]>('select * from emby_line where emby_server_id = $1', [emby_server_id]);
        if (!embyLine || embyLine.length == 0) {
            return [];
        }
        return embyLine;
    }

    async function listAllEmbyLine() {
        let embyLine = await useDb().db?.select<EmbyLine[]>('select * from emby_line');
        if (!embyLine || embyLine.length == 0) {
            return [];
        }
        return embyLine;
    }

    async function addEmbyLine(embyLine: EmbyLine) {
        let fields: string[] = [], values: string[] = [];
        for (const [key, value] of Object.entries(embyLine)) {
            if (value != null && value != undefined && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `insert into emby_line (${fields.join(',')}) values (${fields.map((_item, index) => '$' + (index + 1)).join(',')})`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updateEmbyLine(embyLine: EmbyLine) {
        let fields: string[] = [], values: string[] = [];
        values.push(embyLine.id!);
        for (const [key, value] of Object.entries(embyLine)) {
            if (value != null && value != undefined && key != 'id' && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `update emby_line set ${fields.map((item, index) => item + ' = $' + (index + 2)).join(',')} where id = $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updateEmbyServerName(emby_server_id: string, emby_server_name: string) {
        let values: string[] = [];
        values.push(emby_server_id);
        values.push(emby_server_name);
        let sql = `update emby_line set emby_server_id = $2 where emby_server_name = $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updateEmbyUsing(emby_server_id: string) {
        let values: string[] = [];
        values.push(emby_server_id);
        let sql = `update emby_line set in_use = 0 where emby_server_id = $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function delEmbyServer(emby_server_id: string) {
        let res = await useDb().db?.execute('delete from emby_line where emby_server_id = $1', [emby_server_id]);
        return res?.rowsAffected;
    }

    async function delEmbyLine(id: string) {
        let res = await useDb().db?.execute('delete from emby_line where id = $1', [id]);
        return res?.rowsAffected;
    }

    return { getEmbyLine, delEmbyLine, addEmbyLine, updateEmbyLine, listAllEmbyLine, getUsingEmbyLine, listEmbyLine, updateEmbyServerName, delEmbyServer, updateEmbyUsing }
})

export interface EmbyLine {
    id?: string,
    create_time?: string,

    name?: string,
    emby_server_id?: string,
    emby_server_name?: string,
    base_url?: string,
    in_use?: number,
    browse_proxy_id?: string,
    play_proxy_id?: string,
}
