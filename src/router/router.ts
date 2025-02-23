import * as VueRouter from 'vue-router'
import NavMenu from '../components/NavMenu.vue'
import Search from '../page/Search.vue'
import EmbyHome from '../page/EmbyHome.vue'

const routes = [
    { path: '/', redirect: '/nav/search' },
    {
        path: '/nav',
        component: NavMenu,
        children: [
            {
                path: 'search',
                component: Search,
            },
            {
                path: 'embyHome/:id',
                component: EmbyHome,
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