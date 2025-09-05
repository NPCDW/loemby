import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const useEmbyIconLibrary = defineStore('embyIconLibrary', () => {
    async function getEmbyIconLibrary(id: string): Promise<EmbyIconLibrary> {
        return invoke('get_emby_icon_library', {id});
    }

    async function listAllEmbyIconLibrary(): Promise<EmbyIconLibrary[]> {
        return invoke('list_all_emby_icon_library');
    }

    async function addEmbyIconLibrary(embyIconLibrary: EmbyIconLibrary): Promise<number> {
        return invoke('add_emby_icon_library', {body: embyIconLibrary});
    }

    async function updateEmbyIconLibrary(embyIconLibrary: EmbyIconLibrary): Promise<number> {
        return invoke('update_emby_icon_library', {body: embyIconLibrary});
    }

    async function delEmbyIconLibrary(id: string): Promise<number> {
        return invoke('delete_emby_icon_library', {id: id});
    }

    return { getEmbyIconLibrary, delEmbyIconLibrary, addEmbyIconLibrary, updateEmbyIconLibrary, listAllEmbyIconLibrary }
})

export interface EmbyIconLibrary {
    id?: string,
    create_time?: string,

    name?: string,
    url?: string,
}
