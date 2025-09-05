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
import { usePlayHistory } from './db/playHistory';

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
                            return event.payload.progress > 80 ? 100_0000_0000 : 0
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
                    const played_duration = Number(((new Date().getTime() - event.payload.start_time) / 1000).toFixed())
                    if (played_duration > 5 * 60) {
                        useEmbyServer().updateEmbyServer({id: embyServer!.id!, last_playback_time: dayjs().locale('zh-cn').format('YYYY-MM-DD HH:mm:ss')})
                            .then(() => useEventBus().emit('EmbyServerChanged', {event: 'update', id: embyServer!.id!}))
                    } else {
                        ElMessage.warning('播放时间不足 5 分钟，不更新最后播放时间')
                    }
                    usePlayHistory().getPlayHistory(embyServer!.id!, event.payload.item_id).then(response => {
                        if (response) {
                            usePlayHistory().updatePlayHistory({
                                id: response.id!,
                                update_time: dayjs().locale('zh-cn').format('YYYY-MM-DD HH:mm:ss'),
                                emby_server_name: embyServer!.server_name!,
                                item_name: event.payload.item_name,
                                item_type: event.payload.item_type,
                                series_id: event.payload.series_id,
                                series_name: event.payload.series_name,
                                played_duration: response.played_duration! + played_duration})
                        } else {
                            ElMessage.warning('播放记录不存在，无法更新播放记录')
                        }
                    })
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
    item_type: string;
    item_name: string;
    series_id?: string;
    series_name?: string;
    media_source_id: string;
    play_session_id: string;
    progress: number;
    run_time_ticks: number;
    scrobble_trakt_param?: string;
    playback_status: number;
    start_time: number;
};
