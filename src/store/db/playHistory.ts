import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const usePlayHistory = defineStore('playHistory', () => {
    async function pagePlayHistory(body: PagePlayHistoryParam): Promise<[number, PlayHistory[]]> {
        return invoke('page_play_history', {body});
    }

    async function getPlayHistory(emby_server_id: string, item_id: string): Promise<PlayHistory> {
        return invoke('get_play_history', {body: {emby_server_id, item_id}});
    }

    async function addPlayHistory(playHistory: PlayHistory): Promise<number> {
        return invoke('add_play_history', {body: playHistory});
    }

    async function updatePlayHistory(playHistory: PlayHistory): Promise<number> {
        return invoke('update_play_history', {body: playHistory});
    }

    async function cancelPinned(emby_server_id: string, series_id: string): Promise<number> {
        return invoke('cancel_pinned_play_history', {body: {emby_server_id, series_id}});
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

export interface PagePlayHistoryParam {
    page_number: number,
    page_size: number,
    emby_server_id?: string,
    series_name?: string,
    item_name?: string,
}