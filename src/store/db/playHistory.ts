import { defineStore } from 'pinia';
import { useDb } from '../db';

export const usePlayHistory = defineStore('playHistory', () => {
    async function pagePlayHistory(pageNumber: number, pageSize: number): Promise<{count: number, list: PlayHistory[]}> {
        let count = await useDb().db?.select<number>('select count(*) from play_history');
        if (count == null || count == undefined || count == 0) {
            return {count: 0, list: []};
        }
        let playHistory = await useDb().db?.select<PlayHistory[]>('select * from play_history order by pinned desc, update_time desc limit $1 offset $2', [pageSize, (pageNumber - 1) * pageSize]);
        return {count, list: playHistory || []};
    }

    async function getPlayHistory(emby_server_id: string, item_id: string) {
        let playHistory = await useDb().db?.select<PlayHistory[]>('select * from play_history where emby_server_id = $1 and item_id = $2', [emby_server_id, item_id]);
        if (!playHistory || playHistory.length == 0) {
            return;
        }
        return playHistory[0];
    }

    async function addPlayHistory(playHistory: PlayHistory) {
        let fields: string[] = [], values: string[] = [];
        for (const [key, value] of Object.entries(playHistory)) {
            if (value != null && value != undefined && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `insert into play_history (${fields.join(',')}) values (${fields.map((_item, index) => '$' + (index + 1)).join(',')})`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updatePlayHistory(playHistory: PlayHistory) {
        let fields: string[] = [], values: string[] = [];
        values.push(playHistory.id!);
        for (const [key, value] of Object.entries(playHistory)) {
            if (value != null && value != undefined && key != 'id' && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `update play_history set ${fields.map((item, index) => item + ' = $' + (index + 2)).join(',')} where id = $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function cancelPinned(emby_server_id: string, series_id: string) {
        let values: string[] = [];
        values.push(emby_server_id);
        values.push(series_id);
        let sql = `update play_history set pinned = 0 where emby_server_id = $1 and series_id = $2 and pinned = 1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    return { getPlayHistory, addPlayHistory, updatePlayHistory, pagePlayHistory, cancelPinned }
})

export interface PlayHistory {
    id?: string,
    create_time?: string,
    update_time?: string,

    emby_server_id?: string,
    emby_server_name?: string,
    item_type?: string,
    item_id?: string,
    item_name?: string,
    series_id?: string,
    series_name?: string,
    played_duration?: number,
    pinned?: number,
}
