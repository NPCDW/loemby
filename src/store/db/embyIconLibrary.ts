import { defineStore } from 'pinia';
import { useDb } from '../db';

export const useEmbyIconLibrary = defineStore('embyIconLibrary', () => {
    async function getEmbyIconLibrary(id: string) {
        let embyIconLibrary = await useDb().db?.select<EmbyIconLibrary[]>('select * from emby_icon_library where id = $1', [id]);
        if (!embyIconLibrary || embyIconLibrary.length == 0) {
            return;
        }
        return embyIconLibrary[0];
    }

    async function listAllEmbyIconLibrary() {
        let embyIconLibrary = await useDb().db?.select<EmbyIconLibrary[]>('select * from emby_icon_library');
        if (!embyIconLibrary || embyIconLibrary.length == 0) {
            return [];
        }
        return embyIconLibrary;
    }

    async function addEmbyIconLibrary(embyIconLibrary: EmbyIconLibrary) {
        let fields: string[] = [], values: string[] = [];
        for (const [key, value] of Object.entries(embyIconLibrary)) {
            if (value != null && value != undefined && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `insert into emby_icon_library (${fields.join(',')}) values (${fields.map((_item, index) => '$' + (index + 1)).join(',')})`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function updateEmbyIconLibrary(embyIconLibrary: EmbyIconLibrary) {
        let fields: string[] = [], values: string[] = [];
        values.push(embyIconLibrary.id!);
        for (const [key, value] of Object.entries(embyIconLibrary)) {
            if (value != null && value != undefined && key != 'id' && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `update emby_icon_library set ${fields.map((item, index) => item + ' = $' + (index + 2)).join(',')} where id = $1`;
        let res = await useDb().db?.execute(sql, values);
        return res?.rowsAffected;
    }

    async function delEmbyIconLibrary(id: string) {
        let res = await useDb().db?.execute('delete from emby_icon_library where id = $1', [id]);
        return res?.rowsAffected;
    }

    return { getEmbyIconLibrary, delEmbyIconLibrary, addEmbyIconLibrary, updateEmbyIconLibrary, listAllEmbyIconLibrary }
})

export interface EmbyIconLibrary {
    id?: string,
    create_time?: string,

    name?: string,
    url?: string,
}
