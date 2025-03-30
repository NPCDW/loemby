import { createApp } from 'vue'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './style.css'
import App from './App.vue'
import router from './router/router'
import { createPinia } from 'pinia'
import { useDb } from './store/db'
import { usePlayback } from './store/playback'
import svgIcon from "./components/SvgIcon/index.vue";
import 'virtual:svg-icons-register'
import VueLazyLoad from 'vue3-lazyload'
import updaer_util from './util/updater_util'
import {useRuntimeConfig} from "./store/runtimeConfig.ts";

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.component('svg-icon', svgIcon)
app.use(VueLazyLoad, {})

await useDb().init()
useRuntimeConfig().getRuntimeConfig()
usePlayback().listen_playback_progress()
updaer_util.getUpdate()

app.mount('#app')
