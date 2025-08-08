import { defineStore } from 'pinia';
import * as fs from '@tauri-apps/plugin-fs';
import { ElMessage } from 'element-plus';
import { sleep } from '../util/sleep';
import { useGlobalConfig } from './db/globalConfig';

export const useCache = defineStore('cache', () => {
    async function cleanLogs() {
        console.log('60秒后开始清理日志文件');
        await sleep(60_000)
        const logsDirExists = await fs.exists('logs', { baseDir: fs.BaseDirectory.AppLocalData });
        if (logsDirExists) {
            ElMessage.warning('日志目录不存在，清理失败')
            return
        }
        const logStoredDays = await useGlobalConfig().getGlobalConfigValue("logStoredDays")
        const daysToKeep = logStoredDays ? Number(logStoredDays) : 30;
        const now = new Date();
        const cutoffTime = now.getTime() - (daysToKeep * 24 * 60 * 60 * 1000);
    
        for (const file of await fs.readDir('logs', { baseDir: fs.BaseDirectory.AppLocalData })) {
            if (!file.name) {
                console.log(file.name + '文件没有文件名');
                continue
            };
            if (!file.isFile) {
                console.log(file.name + '不是一个文件');
                continue
            }
    
            if (file.name.indexOf('.log') === -1) {
                console.log(file.name + '文件不包含.log');
                continue
            };
    
            // 获取文件元数据
            const metadata = await fs.stat('logs/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
    
            // 检查文件修改时间
            if (!metadata.mtime) {
                console.log(file.name + '文件没有修改时间');
                continue
            }
            const modifiedTime = metadata.mtime!.getTime();
    
            // 如果文件早于截止时间，则删除
            if (modifiedTime < cutoffTime) {
                await fs.remove('logs/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                console.log(`Deleted old log file: ${file.name}`);
            }
        }
    }

    async function cleanIconsTime() {
        console.log('70秒后开始清理icon文件');
        await sleep(70_000)
        cleanIcons()
    }

    async function cleanIcons(cleanAll: boolean = false) {
        const iconDirExists = await fs.exists('cache/icon', { baseDir: fs.BaseDirectory.AppLocalData });
        if (iconDirExists) {
            return
        }
        const iconStoredDays = await useGlobalConfig().getGlobalConfigValue("iconStoredDays")
        const daysToKeep = iconStoredDays ? Number(iconStoredDays) : 365;
        const now = new Date();
        const cutoffTime = now.getTime() - (daysToKeep * 24 * 60 * 60 * 1000);
    
        for (const file of await fs.readDir('cache/icon', { baseDir: fs.BaseDirectory.AppLocalData })) {
            if (!file.name) {
                console.log(file.name + '文件没有文件名');
                continue
            };
            if (!file.isFile) {
                console.log(file.name + '不是一个文件');
                continue
            }
    
            if (file.name.indexOf('.png') === -1) {
                console.log(file.name + '文件不包含.png');
                continue
            };

            if (cleanAll) {
                await fs.remove('cache/icon/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                console.log(`Deleted icon file: ${file.name}`);
                continue
            }
    
            // 获取文件元数据
            const metadata = await fs.stat('cache/icon/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
    
            // 检查文件修改时间
            if (!metadata.mtime) {
                console.log(file.name + '文件没有修改时间');
                continue
            }
            const modifiedTime = metadata.mtime!.getTime();
    
            // 如果文件早于截止时间，则删除
            if (modifiedTime < cutoffTime) {
                await fs.remove('cache/icon/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                console.log(`Deleted icon file: ${file.name}`);
            }
        }
    }

    async function cleanEmbyCacheTime() {
        console.log('80秒后开始清理Emby缓存文件');
        await sleep(80_000)
        cleanEmbyCache()
    }

    async function cleanEmbyCache(cleanAll: boolean = false, embyId: string = '') {
        let cleanDir = 'cache/image'
        if (embyId) {
            cleanDir = 'cache/image/' + embyId
        }
        const iconDirExists = await fs.exists(cleanDir, { baseDir: fs.BaseDirectory.AppLocalData });
        if (iconDirExists) {
            return
        }
        const coverImageStoredDays = await useGlobalConfig().getGlobalConfigValue("coverImageStoredDays")
        const daysToKeep = coverImageStoredDays ? Number(coverImageStoredDays) : 30;
        const now = new Date();
        const cutoffTime = now.getTime() - (daysToKeep * 24 * 60 * 60 * 1000);
    
        cleanEmbyCacheR(cleanDir, cutoffTime, cleanAll)
    }

    async function cleanEmbyCacheR(dir: string, cutoffTime: number, cleanAll: boolean = false) {
        for (const file of await fs.readDir(dir, { baseDir: fs.BaseDirectory.AppLocalData })) {
            if (!file.name) {
                console.log(file.name + '文件没有文件名');
                continue
            };
            if (file.isDirectory) {
                cleanEmbyCacheR(dir + '/' + file.name, cutoffTime)
                continue
            }
    
            if (file.name.indexOf('.png') === -1) {
                console.log(file.name + '文件不包含.png');
                continue
            };
    
            if (cleanAll) {
                await fs.remove('cache/icon/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                console.log(`Deleted icon file: ${file.name}`);
                continue
            }
    
            // 获取文件元数据
            const metadata = await fs.stat(dir + '/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
    
            // 检查文件修改时间
            if (!metadata.mtime) {
                console.log(file.name + '文件没有修改时间');
                continue
            }
            const modifiedTime = metadata.mtime!.getTime();
    
            // 如果文件早于截止时间，则删除
            if (modifiedTime < cutoffTime) {
                await fs.remove(dir + '/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                console.log(`Deleted emby cache file: ${file.name}`);
            }
        }
    }

    return { cleanLogs, cleanIcons, cleanEmbyCache, cleanIconsTime, cleanEmbyCacheTime }
})
