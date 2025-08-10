import { defineStore } from 'pinia';
import * as fs from '@tauri-apps/plugin-fs';
import { ElMessage } from 'element-plus';
import { sleep } from '../util/sleep';
import { useGlobalConfig } from './db/globalConfig';

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
        const logsDirExists = await fs.exists('logs', { baseDir: fs.BaseDirectory.AppLocalData });
        if (!logsDirExists) {
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
    
            // if (file.name.indexOf('.log') === -1) {
            //     console.log(file.name + '文件不包含.log');
            //     continue
            // };
    
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
        const iconDirExists = await fs.exists('cache/icon', { baseDir: fs.BaseDirectory.AppLocalData });
        if (!iconDirExists) {
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
    
            // if (file.name.indexOf('.png') === -1) {
            //     console.log(file.name + '文件不包含.png');
            //     continue
            // };

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
        const iconDirExists = await fs.exists(cleanDir, { baseDir: fs.BaseDirectory.AppLocalData });
        if (!iconDirExists) {
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
                if ((await fs.readDir(dir + '/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData })).length === 0) {
                    await fs.remove(dir + '/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                    console.log(`Deleted empty directory: ${file.name}`);
                }
                continue
            }
    
            // if (file.name.indexOf('.png') === -1) {
            //     console.log(file.name + '文件不包含.png');
            //     continue
            // };
    
            if (cleanAll) {
                await fs.remove('cache/icon/' + file.name, { baseDir: fs.BaseDirectory.AppLocalData });
                console.log(`Deleted emby cache file: ${file.name}`);
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
