import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import embyApi from '../api/embyApi';
import { ElMessage } from 'element-plus';
import { useEmbyServer } from './db/embyServer';
import { useEventBus } from './eventBus';
import traktApi from '../api/traktApi';

export const usePlayback = defineStore('playback', () => {
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
                        useEventBus().emit('playingStopped', event.payload)
                    })
                    if (event.payload.scrobble_trakt_param) {
                        const progress = Number((event.payload.progress / (event.payload.run_time_ticks / 100)).toFixed(2))
                        let scrobbleTraktParam = JSON.parse(event.payload.scrobble_trakt_param)
                        scrobbleTraktParam.progress = progress
                        traktApi.stop(scrobbleTraktParam)
                    }
                } else {
                    embyApi.playingProgress(embyServer!, event.payload.item_id, event.payload.media_source_id, event.payload.play_session_id, event.payload.progress)
                }
            })
        });
    }
    
    return { listen_playback_progress }
})

export type PlaybackProgress = {
    server_id: string;
    item_id: string;
    media_source_id: string;
    play_session_id: string;
    progress: number;
    run_time_ticks: number;
    scrobble_trakt_param?: string;
    playback_status: number;
};
