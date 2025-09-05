import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { waitUntilTrue } from '../../util/sleep';

export const useGlobalConfig = defineStore('globalConfig', () => {
    const cacheGlobalConfig = ref<{[key: string]: GlobalConfig}>({});
    const initCacheFinish = ref(false)

    async function refreshCache(key: string) {
        let config = await getGlobalConfig(key)
        if (!config) {
            cacheGlobalConfig.value[key] = {};
            return;
        }
        cacheGlobalConfig.value[key] = config;
    }

    async function initCache() {
        let globalConfigList: GlobalConfig[] = await invoke('list_all_global_config');
        for (const item of globalConfigList) {
            cacheGlobalConfig.value[item.config_key!] = item;
        }
        initCacheFinish.value = true
    }

    async function getGlobalConfigValue(config_key: string) {
        if (!initCacheFinish.value) {
            await waitUntilTrue(() => initCacheFinish.value, 100)
        }
        if (!cacheGlobalConfig.value[config_key]) {
            cacheGlobalConfig.value[config_key] = {};
            return '';
        }
        return cacheGlobalConfig.value[config_key].config_value || '';
    }

    // 这个方法不要加缓存
    async function getGlobalConfig(config_key: string): Promise<GlobalConfig> {
        return invoke('get_emby_icon_library', {configKey: config_key});
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

    return { getGlobalConfigValue, getGlobalConfig, delGlobalConfig, addGlobalConfig, updateGlobalConfig, initCache }
})

export interface GlobalConfig {
    id?: string,
    create_time?: string,

    config_key?: string,
    config_value?: string,
}
