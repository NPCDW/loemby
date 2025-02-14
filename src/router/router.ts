import * as VueRouter from 'vue-router'
import NavMenu from '../components/NavMenu.vue'
import Search from '../page/Search.vue'

const routes = [
    { path: '/', redirect: '/nav/search' },
    {
        path: '/window',
        component: NavMenu,
        children: [
            {
                path: 'result',
                component: Search,
            },
        ],
    },
    // { path: '/screenshot', component: Screenshot },
]

const router = VueRouter.createRouter({
    history: VueRouter.createWebHistory(import.meta.env.BASE_URL),
    routes,
})

export default router