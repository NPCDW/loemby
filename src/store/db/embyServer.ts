import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEventBus } from '../eventBus';

export const useEmbyServer = defineStore('embyServer', () => {
    async function getEmbyServer(id: string): Promise<EmbyServer> {
        return invoke('get_emby_server', {id: id});
    }

    async function listAllEmbyServer(): Promise<EmbyServer[]> {
        return invoke('list_all_emby_server');
    }

    async function addEmbyServer(embyServer: EmbyServer): Promise<number> {
        return invoke('add_emby_server', {body: embyServer});
    }

    async function updateEmbyServer(embyServer: EmbyServer): Promise<number> {
        return invoke('update_emby_server', {body: embyServer});
    }

    async function updateOrder(removed_id: string, removed_index: number, added_index: number): Promise<number> {
        return invoke('update_emby_server_order', {body: {removed_id, removed_index, added_index}});
    }

    async function deferOrder(): Promise<number> {
        return invoke('defer_emby_server_order');
    }

    async function delEmbyServer(id: string): Promise<number> {
        return invoke('delete_emby_server', {id: id});
    }

    async function listenEmbyServerChange() {
        listen<EmbyServerChangeParam>('EmbyServerChange', (event) => {
            console.log("tauri EmbyServerChange event", event)
            useEventBus().emit('EmbyServerChanged', {event: event.payload.event, id: event.payload.id})
        });
    }

    return { getEmbyServer, delEmbyServer, addEmbyServer, updateEmbyServer, listAllEmbyServer, updateOrder, deferOrder, listenEmbyServerChange }
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
    icon_url?: string,

    browse_proxy_id?: string,
    play_proxy_id?: string,
    line_id?: string,

    last_playback_time?: string,
    keep_alive_days?: number,

    disabled?: number,
}

interface EmbyServerChangeParam {
    id: string;
    event: string;
}