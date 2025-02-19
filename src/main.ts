import { createApp } from 'vue'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './style.css'
import App from './App.vue'
import router from './router/router'
import { createPinia } from 'pinia'
import svgIcon from "./components/SvgIcon/index.vue";
import 'virtual:svg-icons-register'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.component('svg-icon', svgIcon)

app.mount('#app')
