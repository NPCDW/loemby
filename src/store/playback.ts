import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import embyApi from '../api/embyApi';
import { useConfig } from './config';
import { ElMessage } from 'element-plus';

export const usePlayback = defineStore('playback', () => {

    async function listen_playback_progress() {
        listen<DownloadStarted>('playback_progress', (event) => {
            console.log(`playback ${event.payload.progress / 1000 / 10000} second from ${event.payload.server_id} ${event.payload.item_id} ${event.payload.media_source_id}`);
            let embyServer = useConfig().getEmbyServer(event.payload.server_id);
            if (!embyServer) {
                console.error(`Emby服务器Id: ${event.payload.server_id} 不存在`);
                return
            }
            embyApi.playingStopped(embyServer, event.payload.item_id, event.payload.media_source_id, event.payload.play_session_id, event.payload.progress).then(() => {
                ElMessage.success({
                    message: '播放结束'
                })
            })
        });
    }
    
    return { listen_playback_progress }
})

export type DownloadStarted = {
    server_id: string;
    item_id: string;
    media_source_id: string;
    play_session_id: string;
    progress: number;
};
