import { defineStore } from 'pinia';
import { useDb } from '../db';
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
        let globalConfigList = await useDb().db?.select<GlobalConfig[]>('select * from global_config');
        if (!globalConfigList || globalConfigList.length == 0) {
            return;
        }
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
    async function getGlobalConfig(config_key: string) {
        let globalConfig = await useDb().db?.select<GlobalConfig[]>('select * from global_config where config_key = $1', [config_key]);
        if (!globalConfig || globalConfig.length == 0) {
            return;
        }
        return globalConfig[0];
    }

    async function addGlobalConfig(globalConfig: GlobalConfig) {
        let fields: string[] = [], values: string[] = [];
        for (const [key, value] of Object.entries(globalConfig)) {
            if (value != null && value != undefined && key != 'create_time') {
                fields.push(key);
                values.push(value);
            }
        }
        let sql = `insert into global_config (${fields.join(',')}) values (${fields.map((_item, index) => '$' + (index + 1)).join(',')})`;
        let res = await useDb().db?.execute(sql, values);
        await refreshCache(globalConfig.config_key!);
        return res?.rowsAffected;
    }

    async function updateGlobalConfig(globalConfig: GlobalConfig) {
        let values: string[] = [];
        values.push(globalConfig.config_key!);
        values.push(globalConfig.config_value!);
        let sql = `update global_config set config_value = $2 where config_key = $1`;
        let res = await useDb().db?.execute(sql, values);
        await refreshCache(globalConfig.config_key!);
        return res?.rowsAffected;
    }

    async function delGlobalConfig(config_key: string) {
        let res = await useDb().db?.execute('delete from global_config where config_key = $1', [config_key]);
        await refreshCache(config_key!);
        return res?.rowsAffected;
    }

    return { getGlobalConfigValue, getGlobalConfig, delGlobalConfig, addGlobalConfig, updateGlobalConfig, initCache }
})

export interface GlobalConfig {
    id?: string,
    create_time?: string,

    config_key?: string,
    config_value?: string,
}
