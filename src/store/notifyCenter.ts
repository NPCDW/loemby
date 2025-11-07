import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus';
import { h, ref, VNode } from 'vue';
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'
import { useEventBus } from './eventBus';

export const useNotifyCenter = defineStore('notifyCenter', () => {
    const notifyMessages = ref<NotifyMessage[]>([])

    function push(message: NotifyMessage) {
        notifyMessages.value.push(message)
        sessionStorage.setItem("notify_messages", JSON.stringify(notifyMessages.value))
        useEventBus().emit('notifyMessageChange', {force_open: true})
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
                    type: event.payload.message_type,
                    message: event.payload.message
                });
            } else if (event.payload.event_type === 'ElNotification') {
                ElNotification({
                    type: event.payload.message_type,
                    title: event.payload.title,
                    message: event.payload.message
                });
            } else if (event.payload.event_type === 'ElMessageBox') {
                ElMessageBox({
                    title: event.payload.title,
                    message: event.payload.message
                });
            } else if (event.payload.event_type === 'TraktStart') {
                const json: {progress: number, movie?: {title: string, year: number}, episode?: {title: string, season: number, number: number}, show?: {title: string, year: number}} = JSON.parse(event.payload.message);
                let message: VNode[] = []
                if (json.movie) {
                    message = [h('div', null, "开始播放"), h('div', null, `${json.movie.title} (${json.movie.year})`)]
                } else if (json.episode) {
                    message = [h('div', null, "开始播放"), h('div', null, `${json.show?.title} (${json.show?.year})`), h('div', null, `S${json.episode.season}E${json.episode.number}. ${json.episode.title}`)]
                }
                push({
                    username: "trakt",
                    datetime: dayjs().locale('zh-cn').format("HH:mm:ss"),
                    content: h('div', null, message),
                })
            } else if (event.payload.event_type === 'TraktStop') {
                const json: {progress: number, movie?: {title: string, year: number}, episode?: {title: string, season: number, number: number}, show?: {title: string, year: number}} = JSON.parse(event.payload.message);
                let message: VNode[] = []
                if (json.movie) {
                    message = [h('div', null, '停止播放，同步播放进度' + json.progress + '%'), h('div', null, `${json.movie.title} (${json.movie.year})`)]
                } else if (json.episode) {
                    message = [h('div', null, '停止播放，同步播放进度' + json.progress + '%'), h('div', null, `${json.show?.title} (${json.show?.year})`), h('div', null, `S${json.episode.season}E${json.episode.number}. ${json.episode.title}`)]
                }
                push({
                    username: "trakt",
                    datetime: dayjs().locale('zh-cn').format("HH:mm:ss"),
                    content: h('div', null, message),
                })
            } else if (event.payload.event_type === 'TraktError') {
                push({
                    username: "trakt",
                    datetime: dayjs().locale('zh-cn').format("HH:mm:ss"),
                    level: "danger",
                    content: event.payload.message,
                })
            } else if (event.payload.event_type === 'playingNotify') {
                const json: PlaybackNotifyParam = JSON.parse(event.payload.message);
                if (json.event === 'stop') {
                    let message: VNode[] = []
                    message = [
                        h('div', null, '停止播放'),
                        h('div', null, `${json.series_name}`),
                        h('div', null, `${json.item_name}`)]
                    push({username: "embyServer", datetime: dayjs().locale('zh-cn').format("HH:mm:ss"), "icon": json.emby_server_id, "content": h('div', null, message)})
                    useEventBus().emit('playingNotify', json)
                } else {
                    let message: VNode[] = []
                    message = [
                        h('div', null, '开始播放'),
                        h('div', null, `${json.series_name}`),
                        h('div', null, `${json.item_name}`)]
                    push({username: "embyServer", datetime: dayjs().locale('zh-cn').format("HH:mm:ss"), "icon": json.emby_server_id, "content": h('div', null, message)})
                    useEventBus().emit('playingNotify', json)
                }
            }
        });
    }
    
    return { listen_tauri_notify, push, refresh, notifyMessages }
})

export type TauriNotify = {
    event_type: 'ElMessage' | 'ElMessageBox' | 'ElNotification';
    message_type: 'info' | 'success' | 'warning' | 'error';
    title?: string;
    message: string;
};

export interface NotifyMessage {
    username: string,
    datetime: string,
    icon?: string,
    level?: "primary" | "success" | "warning" | "danger" | "info" | undefined,
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
