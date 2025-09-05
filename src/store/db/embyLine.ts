import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const useEmbyLine = defineStore('embyLine', () => {
    async function getEmbyLine(id: string): Promise<EmbyLine> {
        return invoke('get_emby_line', {id});
    }

    async function listEmbyLine(emby_server_id: string): Promise<EmbyLine[]> {
        return invoke('list_emby_server_line', {embyServerId: emby_server_id});
    }

    async function listAllEmbyLine(): Promise<EmbyLine[]> {
        return invoke('list_all_emby_line');
    }

    async function addEmbyLine(embyLine: EmbyLine): Promise<number> {
        return invoke('add_emby_line', {body: embyLine});
    }

    async function updateEmbyLine(embyLine: EmbyLine): Promise<number> {
        return invoke('update_emby_line', {body: embyLine});
    }

    async function updateEmbyServerName(emby_server_id: string, emby_server_name: string): Promise<number> {
        return invoke('update_emby_line', {body: {emby_server_id, emby_server_name}});
    }

    async function delEmbyServer(emby_server_id: string): Promise<number> {
        return invoke('delete_emby_line', {embyServerId: emby_server_id});
    }

    async function delEmbyLine(id: string): Promise<number> {
        return invoke('delete_emby_line', {id: id});
    }

    return { getEmbyLine, delEmbyLine, addEmbyLine, updateEmbyLine, listAllEmbyLine, listEmbyLine, updateEmbyServerName, delEmbyServer }
})

export interface EmbyLine {
    id?: string,
    create_time?: string,

    name?: string,
    emby_server_id?: string,
    emby_server_name?: string,
    base_url?: string,
    browse_proxy_id?: string,
    play_proxy_id?: string,
}
