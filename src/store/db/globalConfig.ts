import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { waitUntilTrue } from '../../util/sleep';

export const useGlobalConfig = defineStore('globalConfig', () => {
    const cacheGlobalConfig = ref<{[key: string]: GlobalConfig}>({});
    const initCacheFinish = ref(false)
    /** 并发调用 initCache 时复用同一个请求，避免重复拉全量配置 */
    let initCachePromise: Promise<void> | null = null

    async function refreshCache(key: string) {
        let config = await getGlobalConfig(key)
        if (!config) {
            cacheGlobalConfig.value[key] = {};
            return;
        }
        cacheGlobalConfig.value[key] = config;
    }

    function initCache(): Promise<void> {
        if (initCachePromise) {
            return initCachePromise
        }
        initCachePromise = (async () => {
            try {
                const globalConfigList: GlobalConfig[] = await invoke('list_all_global_config');
                for (const item of globalConfigList) {
                    cacheGlobalConfig.value[item.config_key!] = item;
                }
                initCacheFinish.value = true
            } catch (e) {
                // 失败也要放行，否则所有读配置的地方会永久挂住
                initCacheFinish.value = true
                throw e
            }
        })()
        return initCachePromise
    }

    /**
     * 读一个配置值。
     *
     * 冷启动时（缓存还没就绪）会先等缓存；这里主动触发一次 initCache，
     * 避免调用方没预热缓存时永久等待。
     */
    async function getGlobalConfigValue(config_key: string) {
        if (!initCacheFinish.value) {
            initCache().catch(() => undefined)
            await waitUntilTrue(() => initCacheFinish.value, 50)
        }
        if (!cacheGlobalConfig.value[config_key]) {
            cacheGlobalConfig.value[config_key] = {};
            return '';
        }
        return cacheGlobalConfig.value[config_key].config_value || '';
    }

    // 这个方法不要加缓存
    async function getGlobalConfig(config_key: string): Promise<GlobalConfig> {
        return invoke('get_global_config', {configKey: config_key});
    }

    async function addGlobalConfig(globalConfig: GlobalConfig): Promise<unknown> {
        return invoke('add_global_config', {body: globalConfig}).then(async response => {
            await refreshCache(globalConfig.config_key!)
            return response
        });
    }

    async function updateGlobalConfig(globalConfig: GlobalConfig): Promise<unknown> {
        return invoke('update_global_config', {body: globalConfig}).then(async response => {
            await refreshCache(globalConfig.config_key!)
            return response
        });
    }

    async function delGlobalConfig(config_key: string): Promise<unknown> {
        return invoke('delete_global_config', {configKey: config_key}).then(async response => {
            await refreshCache(config_key)
            return response
        });
    }

    return { getGlobalConfigValue, getGlobalConfig, delGlobalConfig, addGlobalConfig, updateGlobalConfig, initCache, refreshCache }
})

export interface GlobalConfig {
    id?: string,
    create_time?: string,

    config_key?: string,
    config_value?: string,
}
