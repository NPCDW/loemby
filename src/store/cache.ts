import { defineStore } from 'pinia';
import { sleep } from '../util/sleep';
import { useGlobalConfig } from './db/globalConfig';
import invokeApi from '../api/invokeApi';

export const useCache = defineStore('cache', () => {
    async function cleanLogs() {
        const cleanLogsConfig = await useGlobalConfig().getGlobalConfigValue("lastCleanLogsTime");
        if (cleanLogsConfig) {
            const lastCleanLogsTime = Number(cleanLogsConfig);
            if (new Date().getTime() - lastCleanLogsTime < 24 * 60 * 60 * 1000) {
                console.log('日志清理被强制忽略，最后一次执行时间：' + new Date(lastCleanLogsTime).toLocaleString('zh-cn'));
                return;
            }
        }
        console.log('60秒后开始清理日志文件');
        await sleep(60_000)
        const logStoredDays = await useGlobalConfig().getGlobalConfigValue("logStoredDays")
        const daysToKeep = logStoredDays ? Number(logStoredDays) : 30;
        
        invokeApi.clean_cache('logs', daysToKeep)

        if (cleanLogsConfig) {
            await useGlobalConfig().updateGlobalConfig({ config_key: 'lastCleanLogsTime', config_value: new Date().getTime() + '' })
        } else {
            await useGlobalConfig().addGlobalConfig({ config_key: 'lastCleanLogsTime', config_value: new Date().getTime() + '' })
        }
    }

    async function cleanIconsTime() {
        const cleanIconConfig = await useGlobalConfig().getGlobalConfigValue("lastCleanIconTime");
        if (cleanIconConfig) {
            const lastCleanIconTime = Number(cleanIconConfig);
            if (new Date().getTime() - lastCleanIconTime < 24 * 60 * 60 * 1000) {
                console.log('图标缓存清理被强制忽略，最后一次执行时间：' + new Date(lastCleanIconTime).toLocaleString('zh-cn'));
                return;
            }
        }
        console.log('70秒后开始清理icon文件');
        await sleep(70_000)
        cleanIcons()
        if (cleanIconConfig) {
            await useGlobalConfig().updateGlobalConfig({ config_key: 'lastCleanIconTime', config_value: new Date().getTime() + '' })
        } else {
            await useGlobalConfig().addGlobalConfig({ config_key: 'lastCleanIconTime', config_value: new Date().getTime() + '' })
        }
    }

    async function cleanIcons(cleanAll: boolean = false) {
        const iconStoredDays = await useGlobalConfig().getGlobalConfigValue("iconStoredDays")
        const daysToKeep = iconStoredDays ? Number(iconStoredDays) : 365;
        
        invokeApi.clean_cache('cache/icon', daysToKeep, cleanAll)
    }

    async function cleanEmbyCacheTime() {
        const cleanEmbyConfig = await useGlobalConfig().getGlobalConfigValue("lastCleanEmbyTime");
        if (cleanEmbyConfig) {
            const lastCleanEmbyTime = Number(cleanEmbyConfig);
            if (new Date().getTime() - lastCleanEmbyTime < 24 * 60 * 60 * 1000) {
                console.log('Emby图片缓存清理被强制忽略，最后一次执行时间：' + new Date(lastCleanEmbyTime).toLocaleString('zh-cn'));
                return;
            }
        }
        console.log('80秒后开始清理Emby缓存文件');
        await sleep(80_000)
        cleanEmbyCache()
        if (cleanEmbyConfig) {
            await useGlobalConfig().updateGlobalConfig({ config_key: 'lastCleanEmbyTime', config_value: new Date().getTime() + '' })
        } else {
            await useGlobalConfig().addGlobalConfig({ config_key: 'lastCleanEmbyTime', config_value: new Date().getTime() + '' })
        }
    }

    async function cleanEmbyCache(cleanAll: boolean = false, embyId: string = '') {
        let cleanDir = 'cache/image'
        if (embyId) {
            cleanDir = 'cache/image/' + embyId
        }
        const coverImageStoredDays = await useGlobalConfig().getGlobalConfigValue("coverImageStoredDays")
        const daysToKeep = coverImageStoredDays ? Number(coverImageStoredDays) : 30;
    
        invokeApi.clean_cache(cleanDir, daysToKeep, cleanAll)
    }

    return { cleanLogs, cleanIcons, cleanEmbyCache, cleanIconsTime, cleanEmbyCacheTime }
})
