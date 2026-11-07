import { createApp, watch } from 'vue'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './style.css'
import App from './App.vue'
import router from './router/router'
import { createPinia } from 'pinia'
import svgIcon from "./components/SvgIcon/index.vue";
import 'virtual:svg-icons-register'
import VueLazyLoad from 'vue3-lazyload'
// import updaer_util from './util/updater_util'
import {useRuntimeConfig} from "./store/runtimeConfig.ts";
import { useNotifyCenter } from './store/notifyCenter.ts'
import { useGlobalConfig } from './store/db/globalConfig.ts'
import { useProxyServer } from './store/db/proxyServer.ts'
import { useReverseProxyServer } from './store/db/reverseProxyServer.ts'
import { useEmbyServer } from './store/db/embyServer.ts'
import { createStartupInitializer } from './util/startup'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.component('svg-icon', svgIcon)
app.use(VueLazyLoad, {})

const runtime = useRuntimeConfig()
const initialize = createStartupInitializer([
    { name: '全局配置缓存', run: () => useGlobalConfig().initCache() },
    { name: '代理缓存', run: () => useProxyServer().initCache() },
    { name: '反向代理缓存', run: () => useReverseProxyServer().initCache() },
    { name: '通知监听', run: () => useNotifyCenter().listen_tauri_notify() },
    { name: 'Emby 服务监听', run: () => useEmbyServer().listenEmbyServerChange() },
])
const stopStartupWatch = watch(() => runtime.startupStatus, status => {
    if (status !== 'ready') return
    stopStartupWatch()
    void initialize()
}, { flush: 'sync' })

app.mount('#app')
// 挂载启动壳后再请求；失败由 store 转为可重试状态。
void runtime.getRuntimeConfig()
// updaer_util.getUpdate()
