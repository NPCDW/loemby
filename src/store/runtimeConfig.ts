import { defineStore } from 'pinia';
import invokeApi from "../api/invokeApi.ts";
import { createRuntimeStartup } from '../util/startup';

export const useRuntimeConfig = defineStore('runtimeConfig', () => {
    return createRuntimeStartup(() => invokeApi.get_runtime_config());
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