<template>
    <main class="mobile-page">
        <header class="mobile-actions page-heading">
            <h1>媒体库</h1>
            <RouterLink :to="homePath">首页</RouterLink>
        </header>

        <section class="mobile-section" aria-label="媒体库入口">
            <h2>全部媒体库</h2>
            <LoadState :loading="libraries.loading.value" :error="libraries.error.value"
                :empty="libraryItems.length === 0" empty-text="暂无媒体库" @retry="libraries.load">
                <div class="library-grid">
                    <RouterLink v-for="library in libraryItems" :key="`${serverId}:${library.Id}`"
                        :to="libraryPath(library.Id)" class="library-entry">
                        <div class="library-cover">
                            <img v-if="covers[library.Id]" :src="covers[library.Id]" alt="" loading="lazy"
                                @error="covers[library.Id] = undefined" />
                            <span v-else class="mobile-muted">暂无封面</span>
                        </div>
                        <strong>{{ library.Name || '未命名媒体库' }}</strong>
                        <span class="mobile-muted">{{ libraryType(library) }}</span>
                    </RouterLink>
                </div>
            </LoadState>
        </section>

        <section v-for="library in latestLibraries" :key="`${serverId}:${library.Id}`"
            class="mobile-section" :aria-label="`${library.Name}最新内容`">
            <header class="mobile-actions section-heading">
                <h2>{{ library.Name }} · 最新内容</h2>
                <RouterLink :to="libraryPath(library.Id)">查看全部</RouterLink>
            </header>
            <LatestLibrary :server-id="serverId" :parent-id="library.Id" />
        </section>
    </main>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import type { EmbyPageList, MediaLibraryItem, SearchItem } from '../../../api/embyApi'
import LoadState from '../../components/LoadState.vue'
import MediaCard from '../../components/MediaCard.vue'
import { useResource } from '../../composables/useResource'
import { useBrowseData } from '../../data/browse'

// 每个媒体库独立管理加载、错误和请求，单库失败不影响其他区域。
const LatestLibrary = defineComponent({
    name: 'LatestLibrary',
    props: {
        serverId: { type: String, required: true },
        parentId: { type: String, required: true },
    },
    setup(props) {
        const data = useBrowseData()
        const latest = useResource<SearchItem[]>(
            () => data.latest(props.serverId, props.parentId, 8),
            () => [props.serverId, props.parentId],
        )
        return () => h(LoadState, {
            loading: latest.loading.value,
            error: latest.error.value,
            empty: !latest.value.value?.length,
            emptyText: '暂无最新内容',
            onRetry: latest.load,
        }, {
            default: () => h('div', { class: 'mobile-grid' },
                (latest.value.value ?? []).map(item => h(MediaCard, {
                    key: item.Id,
                    item,
                    serverId: props.serverId,
                })),
            ),
        })
    },
})

const route = useRoute()
const data = useBrowseData()
const serverId = computed(() => String(route.params.embyId ?? ''))
const homePath = computed(() => `/nav/emby/${encodeURIComponent(serverId.value)}`)
const libraries = useResource<EmbyPageList<MediaLibraryItem>>(
    () => data.libraries(serverId.value),
    () => [serverId.value],
)
const libraryItems = computed(() => libraries.value.value?.Items ?? [])
const latestLibraries = computed(() => libraryItems.value.filter(library =>
    library.CollectionType === 'movies' || library.CollectionType === 'tvshows',
))
const covers = ref<Record<string, string | undefined>>({})
let coverEpoch = 0
let disposed = false

watch([serverId, () => libraries.value.value], () => {
    const epoch = ++coverEpoch
    const server = serverId.value
    covers.value = {}
    for (const library of libraryItems.value) {
        void data.cover(server, library).then(url => {
            if (!disposed && epoch === coverEpoch) covers.value[library.Id] = url
        }).catch(() => {
            // 封面失败仍保留可访问的媒体库入口与占位内容。
        })
    }
}, { immediate: true })

function libraryPath(parentId: string) {
    return `${homePath.value}/mediaLibrary/items/${encodeURIComponent(parentId)}`
}

function libraryType(library: MediaLibraryItem) {
    if (library.CollectionType === 'movies') return '电影'
    if (library.CollectionType === 'tvshows') return '电视剧'
    return '媒体库'
}

onUnmounted(() => {
    disposed = true
    coverEpoch++
})
</script>

<style scoped>
.page-heading, .section-heading { justify-content: space-between; margin-bottom: 16px; }
.page-heading h1, .section-heading h2 { margin: 0; }
.section-heading h2 { min-width: 0; overflow-wrap: anywhere; font-size: 1.1rem; }
.section-heading a { flex-shrink: 0; }
.library-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.library-entry { display: flex; flex-direction: column; gap: 6px; min-width: 0; text-decoration: none; color: inherit; }
.library-entry strong { overflow-wrap: anywhere; }
.library-entry > .mobile-muted { font-size: 0.8rem; }
.library-cover { display: flex; align-items: center; justify-content: center; aspect-ratio: 16 / 9; overflow: hidden; border-radius: 10px; background: var(--el-fill-color); border: 1px solid var(--el-border-color-lighter); font-size: 0.85rem; }
.library-cover img { width: 100%; height: 100%; object-fit: cover; }
@media (min-width: 640px) { .library-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
</style>
