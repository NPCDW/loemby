import * as VueRouter from 'vue-router';
import AppShell from '../layout/AppShell.vue';

/**
 * 路由表。
 *
 * 所有页面都挂在 AppShell 之下，外壳提供服务器上下文；
 * keepAlive 的页面用组件名标识（见 AppShell 里的 KEEP_ALIVE），
 * 这样从详情页返回列表时滚动位置与已加载的数据都还在。
 */
const routes: VueRouter.RouteRecordRaw[] = [
    { path: '/', redirect: '/nav/history' },
    {
        path: '/nav',
        component: AppShell,
        children: [
            {
                path: 'history',
                name: 'HistoryPage',
                component: () => import('../pages/History.vue'),
            },
            {
                path: 'search',
                name: 'AggregateSearchPage',
                component: () => import('../pages/Search.vue'),
            },
            {
                path: 'setting',
                name: 'SettingPage',
                component: () => import('../pages/Setting.vue'),
            },
            {
                path: 'emby/:embyId',
                name: 'ServerHomePage',
                component: () => import('../pages/emby/Home.vue'),
            },
            {
                path: 'emby/:embyId/mediaLibrary',
                name: 'MediaLibraryPage',
                component: () => import('../pages/emby/MediaLibrary.vue'),
            },
            {
                path: 'emby/:embyId/mediaLibrary/items/:parentId',
                name: 'LibraryItemsPage',
                component: () => import('../pages/emby/MediaLibraryItems.vue'),
            },
            {
                path: 'emby/:embyId/search',
                name: 'ServerSearchPage',
                component: () => import('../pages/emby/Search.vue'),
            },
            {
                path: 'emby/:embyId/series/:serieId',
                name: 'SeriesPage',
                component: () => import('../pages/emby/Series.vue'),
            },
            {
                path: 'emby/:embyId/episodes/:episodeId',
                name: 'EpisodePage',
                component: () => import('../pages/emby/Episodes.vue'),
            },
        ],
    },
];

const router = VueRouter.createRouter({
    history: VueRouter.createWebHistory(import.meta.env.BASE_URL),
    routes,
});

export default router;
