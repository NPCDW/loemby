import * as VueRouter from 'vue-router'
import NavMenu from '../components/NavMenu.vue'
import Search from '../page/Search.vue'
import Setting from '../page/Setting.vue'
import EmbyHome from '../page/emby/Home.vue'
import EmbyEpisodes from '../page/emby/Episodes.vue'
import EmbySeries from '../page/emby/Series.vue'
import EmbyMediaLibrary from '../page/emby/MediaLibrary.vue'
import EmbyMediaLibraryItems from '../page/emby/MediaLibraryItems.vue'
import EmbySearch from '../page/emby/Search.vue'

const routes = [
    { path: '/', redirect: '/nav/search' },
    {
        path: '/nav',
        component: NavMenu,
        children: [
            {
                path: 'search',
                component: Search,
                meta: {
                    keepAlive: true,
                }
            },
            {
                path: 'setting',
                component: Setting,
            },
            {
                path: 'emby/:embyId',
                component: EmbyHome,
            },
            {
                path: 'emby/:embyId/episodes/:episodeId',
                component: EmbyEpisodes,
            },
            {
                path: 'emby/:embyId/series/:serieId',
                component: EmbySeries,
                meta: {
                    keepAlive: true,
                }
            },
            {
                path: 'emby/:embyId/mediaLibrary',
                component: EmbyMediaLibrary,
                meta: {
                    keepAlive: true,
                }
            },
            {
                path: 'emby/:embyId/mediaLibrary/items/:parentId',
                component: EmbyMediaLibraryItems,
                meta: {
                    keepAlive: true,
                }
            },
            {
                path: 'emby/:embyId/search',
                component: EmbySearch,
                meta: {
                    keepAlive: true,
                }
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