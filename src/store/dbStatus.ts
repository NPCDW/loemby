import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';

/**
 * 数据库状态。
 * 数据库连不上时应用不会退出，而是进入降级状态，由 FatalError 页面统一提示，
 * 避免用户只看到白屏或一闪而过的报错。
 */
export const useDbStatus = defineStore('dbStatus', () => {
    const fatalReason = ref<string>()
    const listening = ref(false)

    async function listenDbFatalError() {
        if (listening.value) {
            return
        }
        listening.value = true
        await listen<{ reason: string }>('db_fatal_error', event => {
            fatalReason.value = event.payload.reason
        })
    }

    return { fatalReason, listenDbFatalError }
})
