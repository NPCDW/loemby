import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import embyApi from '../api/embyApi';
import { ElMessage, ElNotification } from 'element-plus';
import { useEmbyServer } from './db/embyServer';
import { useEventBus } from './eventBus';
import traktApi from '../api/traktApi';
import { h, VNode } from 'vue';
import dayjs from 'dayjs'

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
                    const progress = (() => {
                        if (event.payload.run_time_ticks === 0) {
                            return event.payload.progress > 80 ? 100 : 0
                        } else {
                            return event.payload.progress
                        }
                    })()
                    embyApi.playingStopped(embyServer, event.payload.item_id, event.payload.media_source_id, event.payload.play_session_id, progress).then(() => {
                        if (event.payload.run_time_ticks === 0) {
                            ElMessage.success('播放结束，时长信息不存在，无法标记确切进度')
                        } else {
                            ElMessage.success('播放结束')
                        }
                        useEventBus().emit('playingStopped', event.payload)
                    })
                    useEmbyServer().updateEmbyServer({id: embyServer!.id!, last_playback_time: dayjs().locale('zh-cn').format('YYYY-MM-DD HH:mm:ss')})
                        .then(() => useEventBus().emit('EmbyServerChanged', {event: 'update', id: embyServer!.id!}))
                    if (event.payload.scrobble_trakt_param) {
                        const progress = (() => {
                            if (event.payload.run_time_ticks === 0) {
                                return event.payload.progress
                            } else {
                                return Number((event.payload.progress / (event.payload.run_time_ticks / 100)).toFixed(2))
                            }
                        })()
                        let scrobbleTraktParam = JSON.parse(event.payload.scrobble_trakt_param)
                        scrobbleTraktParam.progress = progress
                        traktApi.stop(scrobbleTraktParam).then(response => {
                            if (response.status_code == 401 || response.status_code == 429) {
                                return
                            }
                            if (response.status_code != 201) {
                                ElMessage.error('Trakt 同步失败：' + response.status_code + ' ' + response.status_text)
                                return
                            }
                            const json: {progress: number, movie?: {title: string, year: number}, episode?: {title: string, season: number, number: number}, show?: {title: string, year: number}} = JSON.parse(response.body);
                            let message: VNode[] = []
                            if (json.movie) {
                                message = [h('p', null, `${json.movie.title} (${json.movie.year})`)]
                            } else if (json.episode) {
                                message = [h('p', null, `${json.show?.title} (${json.show?.year})`), h('p', null, `S${json.episode.season}E${json.episode.number} ${json.episode.title}`)]
                            }
                            ElNotification.success({
                                title: 'Trakt 同步播放进度' + json.progress + '%',
                                message: h('div', null, message),
                                position: 'bottom-right',
                            })
                        }).catch(e => ElMessage.error("Trakt 同步失败：" + e))
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
