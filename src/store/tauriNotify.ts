import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus';
import { h, VNode } from 'vue';

export const useTauriNotify = defineStore('tauriNotify', () => {
    async function listen_tauri_notify() {
        listen<TauriNotify>('tauri_notify', (event) => {
            console.log("tauri tauri_notify event", event)
            if (event.payload.alert_type === 'ElMessage') {
                ElMessage({
                    type: event.payload.message_type,
                    message: event.payload.message
                });
            } else if (event.payload.alert_type === 'ElNotification') {
                ElNotification({
                    type: event.payload.message_type,
                    title: event.payload.title,
                    message: event.payload.message
                });
            } else if (event.payload.alert_type === 'ElMessageBox') {
                ElMessageBox({
                    title: event.payload.title,
                    message: event.payload.message
                });
            } else if (event.payload.alert_type === 'TraktStart') {
                const json: {progress: number, movie?: {title: string, year: number}, episode?: {title: string, season: number, number: number}, show?: {title: string, year: number}} = JSON.parse(event.payload.message);
                let message: VNode[] = []
                if (json.movie) {
                    message = [h('p', null, `${json.movie.title} (${json.movie.year})`)]
                } else if (json.episode) {
                    message = [h('p', null, `${json.show?.title} (${json.show?.year})`), h('p', null, `S${json.episode.season}E${json.episode.number} ${json.episode.title}`)]
                }
                ElNotification.success({
                    title: 'Trakt 同步播放',
                    message: h('div', null, message),
                    position: 'bottom-right',
                })
            } else if (event.payload.alert_type === 'TraktStop') {
                const json: {progress: number, movie?: {title: string, year: number}, episode?: {title: string, season: number, number: number}, show?: {title: string, year: number}} = JSON.parse(event.payload.message);
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
            }
        });
    }
    
    return { listen_tauri_notify }
})

export type TauriNotify = {
    alert_type: 'ElMessage' | 'ElMessageBox' | 'ElNotification';
    message_type: 'info' | 'success' | 'warning' | 'error';
    title?: string;
    message: string;
};
