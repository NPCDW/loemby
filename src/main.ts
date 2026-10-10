import { createApp } from 'vue'
import { ElMessage } from 'element-plus'
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
import { useDbStatus } from './store/dbStatus.ts'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.component('svg-icon', svgIcon)
app.use(VueLazyLoad, {})

// 数据库连不上时后端不会退出，这里先把状态监听挂上，避免错过启动阶段的事件
const dbStatus = useDbStatus()
dbStatus.listenDbFatalError()

// 运行时配置读取失败通常是后端启动异常，提示用户而不是留个白屏
let runtimeConfigReady = true
await useRuntimeConfig().getRuntimeConfig().catch((e) => {
    runtimeConfigReady = false
    console.error('获取运行时配置失败', e)
})

app.mount('#app')

if (!runtimeConfigReady) {
    ElMessage.error('应用初始化失败，部分功能不可用，请查看日志或重启应用')
}

useGlobalConfig().initCache()
useProxyServer().initCache()
useReverseProxyServer().initCache()
useNotifyCenter().listen_tauri_notify()
useEmbyServer().listenEmbyServerChange()
// updaer_util.getUpdate()
