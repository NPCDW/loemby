import { createApp } from 'vue'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './style.css'
import App from './App.vue'
import router from './router/router'
import { createPinia } from 'pinia'

import 'vuetify/styles'
import { createVuetify } from 'vuetify'

const app = createApp(App)
const pinia = createPinia()

const vuetify = createVuetify()

app.use(pinia)
app.use(router)
app.use(vuetify)

app.mount('#app')
