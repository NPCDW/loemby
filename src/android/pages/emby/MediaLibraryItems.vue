<template>
    <main class="mobile-page">
        <header class="mobile-actions page-heading">
            <h1>{{ library.value.value?.Name || '库内媒体' }}</h1>
            <RouterLink :to="librariesPath">全部媒体库</RouterLink>
        </header>
        <p v-if="library.loading.value" class="mobile-muted" role="status">正在读取媒体库名称…</p>
        <div v-else-if="library.error.value" class="mobile-actions title-error" role="alert">
            <span class="mobile-muted">媒体库名称加载失败，不影响浏览。</span>
            <button type="button" @click="library.load">重试名称</button>
        </div>

        <section class="mobile-section" aria-label="库内媒体列表">
            <p v-if="!children.loading.value && !children.error.value" class="mobile-muted">共 {{ total.toLocaleString() }} 项</p>
            <LoadState :loading="children.loading.value" :error="children.error.value"
                :empty="items.length === 0" empty-text="此媒体库暂无内容" @retry="children.load">
                <div class="mobile-grid">
                    <MediaCard v-for="item in items" :key="`${serverId}:${parentId}:${item.Id}`"
                        :item="item" :server-id="serverId" />
                </div>
            </LoadState>
            <Pagination :page="page" :total="total" :page-size="pageSize"
                :loading="children.loading.value" @change="changePage" />
        </section>
    </main>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import type { EmbyPageList, EpisodeItem, SearchItem } from '../../../api/embyApi'
import LoadState from '../../components/LoadState.vue'
import MediaCard from '../../components/MediaCard.vue'
import Pagination from '../../components/Pagination.vue'
import { useResource } from '../../composables/useResource'
import { useBrowseData } from '../../data/browse'

const route = useRoute()
const data = useBrowseData()
const serverId = computed(() => String(route.params.embyId ?? ''))
const parentId = computed(() => String(route.params.parentId ?? ''))
const librariesPath = computed(() => `/nav/emby/${encodeURIComponent(serverId.value)}/mediaLibrary`)
const page = ref(1)
const pageSize = 18

// 目录或合集复用当前路由，读取新父级内容前先同步重置页码。
watch([serverId, parentId], () => { page.value = 1 }, { flush: 'sync' })
const library = useResource<EpisodeItem>(
    () => data.item(serverId.value, parentId.value),
    () => [serverId.value, parentId.value],
)
const children = useResource<EmbyPageList<SearchItem>>(
    () => data.children(serverId.value, parentId.value, (page.value - 1) * pageSize, pageSize),
    () => [serverId.value, parentId.value, page.value],
)
const items = computed(() => children.value.value?.Items ?? [])
const total = computed(() => children.value.value?.TotalRecordCount ?? 0)
watch([() => children.value.value, () => children.loading.value], ([value, loading]) => {
    if (!value || loading) return
    const lastPage = Math.max(1, Math.ceil(value.TotalRecordCount / pageSize))
    if (page.value > lastPage) page.value = lastPage
})

function changePage(nextPage: number) {
    if (children.loading.value || nextPage < 1 || nextPage > Math.max(1, Math.ceil(total.value / pageSize))) return
    page.value = nextPage
}
</script>

<style scoped>
.page-heading { justify-content: space-between; margin-bottom: 16px; }
.page-heading h1 { margin: 0; min-width: 0; overflow-wrap: anywhere; }
.page-heading a { flex-shrink: 0; }
.title-error { margin-top: 12px; }
.title-error button { min-height: 44px; }
</style>
