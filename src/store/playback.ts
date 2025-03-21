import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import embyApi from '../api/embyApi';
import { ElMessage } from 'element-plus';
import { ref } from 'vue';
import { useEmbyServer } from './db/embyServer';

export const usePlayback = defineStore('playback', () => {
    const playingStopped = ref({
        server_id: '',
        item_id: '',
    });

    async function listen_playback_progress() {
        listen<PlaybackProgress>('playback_progress', (event) => {
            console.log(`store playback_progress: playback ${event.payload.progress / 1000 / 10000} second from ${event.payload.server_id} ${event.payload.item_id} ${event.payload.media_source_id}`);
            useEmbyServer().getEmbyServer(event.payload.server_id).then((embyServer) => {
                if (!embyServer) {
                    console.error(`Emby服务器Id: ${event.payload.server_id} 不存在`);
                    return
                }
                if (event.payload.playback_status === 0) {
                    embyApi.playingStopped(embyServer, event.payload.item_id, event.payload.media_source_id, event.payload.play_session_id, event.payload.progress).then(() => {
                        ElMessage.success({
                            message: '播放结束'
                        })
                        playingStopped.value = { server_id: event.payload.server_id, item_id: event.payload.item_id }
                        console.log('store playingStopped', playingStopped.value);
                    })
                } else {
                    embyApi.playingProgress(embyServer!, event.payload.item_id, event.payload.media_source_id, event.payload.play_session_id, event.payload.progress)
                }
            })
        });
    }
    
    return { listen_playback_progress, playingStopped }
})

export type PlaybackProgress = {
    server_id: string;
    item_id: string;
    media_source_id: string;
    play_session_id: string;
    progress: number;
    playback_status: number;
};
