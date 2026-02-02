import { defineStore } from 'pinia';
import { ref } from 'vue';
import invokeApi from "../api/invokeApi.ts";

export const useRuntimeConfig = defineStore('runtimeConfig', () => {
    const runtimeConfig = ref<RuntimeConfig>()

    async function getRuntimeConfig() {
        runtimeConfig.value = await invokeApi.get_runtime_config();
    }

    return { getRuntimeConfig, runtimeConfig }
})

export interface RuntimeConfig {
    version: string,
    app_config: {
        log_level: string,
        database_type: string,
        database_url: string,
    },
    axum_port: number,
}