import { listen } from '@tauri-apps/api/event';
import _ from 'lodash';
import { defineStore } from 'pinia'
import { ElMessage, ElNotification } from 'element-plus';

export const useTauriNotify = defineStore('tauriNotify', () => {
    async function listen_tauri_notify() {
        listen<TauriNotify>('tauri_notify', (event) => {
            console.log(`store tauri_notify: ${JSON.stringify(event.payload)}`);
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
