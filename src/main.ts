import { createApp } from 'vue'
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
import { useEmbyServer } from './store/db/embyServer.ts'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.component('svg-icon', svgIcon)
app.use(VueLazyLoad, {})

await useRuntimeConfig().getRuntimeConfig()

app.mount('#app')

useGlobalConfig().initCache()
useProxyServer().initCache()
useNotifyCenter().listen_tauri_notify()
useEmbyServer().listenEmbyServerChange()
useNotifyCenter().refresh()
// updaer_util.getUpdate()
