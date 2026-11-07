import type { RouteRecordRaw } from 'vue-router'

export const androidRoutes: RouteRecordRaw[] = [{
    path: '/nav',
    component: () => import('./Layout.vue'),
    children: [
        { path: 'history', component: () => import('./pages/History.vue') },
        { path: 'emby/:embyId', component: () => import('./pages/emby/Home.vue') },
        { path: 'emby/:embyId/series/:serieId', component: () => import('./pages/emby/Series.vue') },
        { path: 'emby/:embyId/episodes/:episodeId', component: () => import('./pages/emby/Episodes.vue') },
        { path: 'emby/:embyId/mediaLibrary', component: () => import('./pages/emby/MediaLibrary.vue') },
        { path: 'emby/:embyId/mediaLibrary/items/:parentId', component: () => import('./pages/emby/MediaLibraryItems.vue') },
    ],
}]
