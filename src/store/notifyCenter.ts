import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import { ElLink, ElMessage, ElMessageBox, ElNotification } from 'element-plus';
import { h, ref, VNode } from 'vue';
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'
import { useEventBus } from './eventBus';
import { generateGuid } from '../util/uuid_util';
import invokeApi from '../api/invokeApi';
import { useRouter } from 'vue-router';

export const useNotifyCenter = defineStore('notifyCenter', () => {
    const notifyMessages = ref<NotifyMessage[]>([])
    // let request = window.indexedDB.open("notify_messages");
    // let db;
    // request.onsuccess = function (event) {
    //     db = request.result;
    //     console.log('indexedDB 数据库打开成功');
    // };
    // request.onupgradeneeded = function (event) {
    //     db = event.target?.result;
    //     console.log('indexedDB 数据库新建成功');
    // }

    function push(message: NotifyMessage) {
        notifyMessages.value.push(message)
        useEventBus().emit('notifyMessageChange', {force_open: true})
    //     sessionStorage.setItem("notify_messages", JSON.stringify(notifyMessages.value))
    // console.log("emit notifyMessageChange")
    }
    
    function refresh() {
        const notify_messages = sessionStorage.getItem("notify_messages")
        if (notify_messages) {
            notifyMessages.value = JSON.parse(notify_messages);
        }
    }
    
    async function listen_tauri_notify() {
        listen<TauriNotify>('tauri_notify', (event) => {
            console.log("tauri tauri_notify event", event)
            if (event.payload.event_type === 'ElMessage') {
                ElMessage({
                    type: event.payload.message_type as "success" | "info" | "warning" | "error",
                    message: event.payload.message
                });
            } else if (event.payload.event_type === 'ElNotification') {
                ElNotification({
                    type: event.payload.message_type as "success" | "info" | "warning" | "error",
                    title: event.payload.title,
                    message: event.payload.message
                });
            } else if (event.payload.event_type === 'ElMessageBox') {
                ElMessageBox({
                    title: event.payload.title,
                    message: event.payload.message
                });
            } else if (event.payload.event_type === 'TraktNotify') {
                if (event.payload.message_type == 'error') {
                    push({
                        id: generateGuid(),
                        username: "trakt",
                        datetime: dayjs().locale('zh-cn').format("HH:mm:ss"),
                        level: "danger",
                        content: event.payload.message,
                    })
                }
                const json: TraktScrobbleResponse = JSON.parse(event.payload.message);
                let message: VNode[] = []
                if (event.payload.message_type == 'start') {
                    message.push(h('div', null, "开始播放"))
                } else {
                    message.push(h('div', null, '停止播放，同步播放进度' + json.progress + '%'))
                }
                if (json.movie) {
                    message.push(h('div', null, h(ElLink, {underline: false, onClick: () => invokeApi.open_url(`https://trakt.tv/movies/${json.movie?.ids.slug}`)}, () => `${json.movie?.title} (${json.movie?.year})`)))
                } else if (json.episode) {
                    message.push(h('div', null, h(ElLink, {underline: false, onClick: () => invokeApi.open_url(`https://trakt.tv/shows/${json.show?.ids.slug}`)}, () => `${json.show?.title} (${json.show?.year})`)))
                    message.push(h('div', null, h(ElLink, {underline: false, onClick: () => invokeApi.open_url(`https://trakt.tv/shows/${json.show?.ids.slug}/seasons/${json.episode?.season}/episodes/${json.episode?.number}`)}, () => `S${json.episode?.season}E${json.episode?.number}. ${json.episode?.title}`)))
                }
                push({
                    id: generateGuid(),
                    username: "trakt",
                    datetime: dayjs().locale('zh-cn').format("HH:mm:ss"),
                    content: h('div', null, message),
                })
            } else if (event.payload.event_type === 'playingNotify') {
                const json: PlaybackNotifyParam = JSON.parse(event.payload.message);
                let message: VNode[] = []
                if (json.event === 'stop') {
                    message.push(h('div', null, '停止播放'));
                } else {
                    message.push(h('div', null, '开始播放'));
                }
                if (json.series_id) {
                    message.push(h('div', null, h(ElLink, {underline: false, onClick: () => gotoSeries(json.emby_server_id, json.series_id!)}, () => `${json.series_name}`)))
                }
                message.push(h('div', null, h(ElLink, {underline: false, onClick: () => gotoEpisodes(json.emby_server_id, json.item_id)}, () => `${json.item_name}`)))
                push({id: generateGuid(), username: "embyServer", datetime: dayjs().locale('zh-cn').format("HH:mm:ss"), "embyServerId": json.emby_server_id, "content": h('div', null, message)})
                useEventBus().emit('playingNotify', json)
            }
        });
    }
    
    const router = useRouter()
    function gotoEpisodes(embyServerId: string, episodesId: string) {
        router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
    }
    function gotoSeries(embyServerId: string, seriesId: string) {
        router.push('/nav/emby/' + embyServerId + '/series/' + seriesId)
    }

    return { listen_tauri_notify, push, refresh, notifyMessages }
})

export type TauriNotify = {
    event_type: string;
    message_type: string;
    title?: string;
    message: string;
};

export interface NotifyMessage {
    id: string,
    username: string,
    datetime: string,
    icon?: string,
    embyServerId?: string,
    level?: "primary" | "success" | "warning" | "danger" | "info",
    content: string | VNode,
};

export interface PlaybackNotifyParam {
    emby_server_id: string;
    series_id?: string;
    series_name: string;
    item_id: string;
    item_name: string
    event: string;
}

interface TraktScrobbleResponse {
    progress: number,
    movie?: {
        title: string,
        year: number,
        ids: {
            slug: string,
        },
    },
    episode?: {
        title: string,
        season: number,
        number: number,
    },
    show?: {
        title: string,
        year: number,
        ids: {
            slug: string,
        },
    }
}
