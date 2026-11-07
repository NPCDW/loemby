import { createApp } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './style.css'
import Preview from './Preview.vue'
import { androidRoutes } from '../routes'
import { browseDataKey } from '../data/browse'
import { previewBrowseData } from './browse'
import { playbackDataKey } from '../data/playback'
import { platformDataKey } from '../data/platform'
import { historyDataKey } from '../data/history'
import { previewData } from './data'

if (!import.meta.env.DEV) throw new Error('Android preview is development-only')

const router = createRouter({
    history: createWebHashHistory(),
    routes: [
        { path: '/', redirect: '/nav/history' },
        ...androidRoutes,
    ],
})
createApp(Preview)
    .provide(historyDataKey, previewData)
    .provide(browseDataKey, previewBrowseData)
    .provide(platformDataKey, {
        openUrl: async () => { throw new Error('开发预览不打开系统浏览器') },
        logo: async () => undefined,
        versionPolicy: async () => 'high-resolution',
    })
    .provide(playbackDataKey, {
        play: async () => { throw new Error('开发预览不启动真实播放器，请在 Android 应用中播放') },
        subscribe: () => () => {},
    })
    .use(router).mount('#app')
