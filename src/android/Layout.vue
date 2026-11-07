<template>
    <div class="android-layout">
        <header class="mobile-toolbar">
            <button type="button" aria-label="返回上一页" :disabled="route.path === '/nav/history'" @click="back">返回</button>
            <RouterLink to="/nav/history" class="brand">LOEMBY</RouterLink>
            <button type="button" :aria-expanded="menuOpen" aria-controls="mobile-servers" @click="menuOpen = !menuOpen">服务器</button>
        </header>
        <section v-if="menuOpen" id="mobile-servers" class="server-panel" aria-label="服务器选择">
            <h2>选择服务器</h2>
            <LoadState :loading="serversLoading" :error="serversError" :empty="!servers?.length" empty-text="没有已配置的服务器；服务器管理将在后续适配。" @retry="loadServers">
                <RouterLink v-for="server in servers" :key="server.id" :to="`/nav/emby/${encodeURIComponent(server.id!)}`" @click="menuOpen = false">{{ server.server_name || '未命名服务器' }}</RouterLink>
            </LoadState>
        </section>
        <nav class="mobile-main-nav" aria-label="主导航">
            <RouterLink to="/nav/history" :class="{ selected: route.path === '/nav/history' }">播放历史</RouterLink>
            <template v-if="serverId">
                <RouterLink :to="serverRoot" :class="{ selected: route.path === serverRoot }">服务器首页</RouterLink>
                <RouterLink :to="`${serverRoot}/mediaLibrary`" :class="{ selected: route.path.includes('/mediaLibrary') }">媒体库</RouterLink>
            </template>
            <button v-else type="button" @click="menuOpen = true">浏览服务器</button>
        </nav>
        <RouterView />
    </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import LoadState from './components/LoadState.vue'
import { useResource } from './composables/useResource'
import { useBrowseData } from './data/browse'
import './style.css'
const data = useBrowseData()
const route = useRoute()
const router = useRouter()
const menuOpen = ref(false)
const serverId = computed(() => String(route.params.embyId || ''))
const serverRoot = computed(() => `/nav/emby/${encodeURIComponent(serverId.value)}`)
const { value: servers, loading: serversLoading, error: serversError, load: loadServers } = useResource(async () =>
    (await data.servers()).filter(server => server.id).sort((a, b) => (a.order_by ?? 0) - (b.order_by ?? 0)), () => 'servers')
function back() {
    if (router.options.history.state.back) { router.back(); return }
    if (route.path.includes('/mediaLibrary/items/')) void router.replace(`${serverRoot.value}/mediaLibrary`)
    else if (route.params.serieId || route.params.episodeId || route.path.endsWith('/mediaLibrary')) void router.replace(serverRoot.value)
    else void router.replace('/nav/history')
}
</script>

<style scoped>
.mobile-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: max(8px, env(safe-area-inset-top)) max(16px, env(safe-area-inset-right)) 8px max(16px, env(safe-area-inset-left)); border-bottom: 1px solid var(--el-border-color-lighter); }
.brand { font-weight: 800; letter-spacing: .1em; text-decoration: none; color: inherit; }
.mobile-main-nav { display: flex; gap: 8px; padding: 8px 16px; flex-wrap: wrap; border-bottom: 1px solid var(--el-border-color-lighter); }
.mobile-main-nav a { display: flex; align-items: center; min-height: 44px; padding: 0 12px; border-radius: 12px; text-decoration: none; color: inherit; }
.selected { background: var(--el-color-primary-light-9); color: var(--el-color-primary) !important; }
.server-panel { padding: 16px; background: var(--el-bg-color-overlay); border-bottom: 1px solid var(--el-border-color-lighter); }
.server-panel h2 { font-size: 18px; margin: 0 0 12px; }
.server-panel a { display: block; padding: 14px; margin: 8px 0; border: 1px solid var(--el-border-color-lighter); border-radius: 12px; color: inherit; text-decoration: none; }
</style>
