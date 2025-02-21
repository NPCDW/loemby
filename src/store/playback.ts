import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'

export const usePlayback = defineStore('playback', () => {

    async function listen_playback_progress() {
        listen<DownloadStarted>('playback_progress', (event) => {
            console.log(
              `playback ${event.payload.progress / 1000 / 10000} second from ${event.payload.playback_id}`
            );
        });
    }
    
    return { listen_playback_progress }
})

export type DownloadStarted = {
    playback_id: string;
    progress: number;
};
