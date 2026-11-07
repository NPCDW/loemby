<template>
    <main class="mobile-page">
        <header class="mobile-actions page-heading">
            <h1>首页</h1>
            <RouterLink :to="libraryPath">媒体库</RouterLink>
        </header>

        <nav class="mobile-tabs" aria-label="首页内容">
            <button v-for="tab in tabs" :key="tab.value" type="button"
                :class="{ active: activeTab === tab.value }" :aria-pressed="activeTab === tab.value"
                @click="activeTab = tab.value">{{ tab.label }}</button>
        </nav>

        <section class="mobile-section" :aria-label="activeLabel">
            <LoadState :loading="content.loading.value" :error="content.error.value" :empty="isEmpty"
                :empty-text="emptyText" @retry="content.load">
                <div v-if="activeTab === 'resume'" class="mobile-grid">
                    <MediaCard v-for="item in resumeItems" :key="`${serverId}:${item.Id}`"
                        :item="item" :server-id="serverId">
                        <div class="resume-actions" @click.stop>
                            <div class="resume-progress">
                                <progress :value="playedPercentage(item)" max="100" :aria-label="`${item.Name}观看进度`" />
                                <span class="mobile-muted">{{ playedPercentage(item) }}%</span>
                            </div>
                            <span v-if="hiddenIds.has(item.Id)" class="mobile-muted" role="status">已隐藏，可恢复</span>
                            <button type="button" :disabled="pendingIds.has(item.Id)"
                                :aria-label="`${hiddenIds.has(item.Id) ? '恢复' : '隐藏'}${item.Name}`"
                                @click.stop.prevent="toggleResume(item)">
                                {{ pendingIds.has(item.Id) ? '处理中…' : hiddenIds.has(item.Id) ? '恢复继续观看' : '隐藏继续观看' }}
                            </button>
                            <p v-if="updateErrors[item.Id]" class="mobile-error" role="alert">{{ updateErrors[item.Id] }}</p>
                        </div>
                    </MediaCard>
                </div>
                <div v-else-if="activeTab === 'favorites'" class="mobile-grid">
                    <MediaCard v-for="item in favoriteItems" :key="`${serverId}:${item.Id}`"
                        :item="item" :server-id="serverId" />
                </div>
                <dl v-else class="statistics">
                    <div v-for="stat in statistics" :key="stat.label">
                        <dt class="mobile-muted">{{ stat.label }}</dt>
                        <dd>{{ stat.value.toLocaleString() }}</dd>
                    </div>
                </dl>
            </LoadState>
            <Pagination v-if="activeTab !== 'count'" :page="currentPage" :total="total"
                :page-size="pageSize" :loading="content.loading.value" @change="changePage" />
        </section>
    </main>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import type { EmbyPageList, EpisodeItem, MediaLibraryCount, SearchItem } from '../../../api/embyApi'
import LoadState from '../../components/LoadState.vue'
import MediaCard from '../../components/MediaCard.vue'
import Pagination from '../../components/Pagination.vue'
import { useResource } from '../../composables/useResource'
import { useBrowseData } from '../../data/browse'

type Tab = 'resume' | 'favorites' | 'count'
type HomeContent =
    | { tab: 'resume'; list: EmbyPageList<EpisodeItem> }
    | { tab: 'favorites'; list: EmbyPageList<SearchItem> }
    | { tab: 'count'; count: MediaLibraryCount }

const route = useRoute()
const data = useBrowseData()
const serverId = computed(() => String(route.params.embyId ?? ''))
const libraryPath = computed(() => `/nav/emby/${encodeURIComponent(serverId.value)}/mediaLibrary`)
const tabs: { value: Tab; label: string }[] = [
    { value: 'resume', label: '继续观看' },
    { value: 'favorites', label: '收藏' },
    { value: 'count', label: '统计' },
]
const activeTab = ref<Tab>('resume')
const resumePage = ref(1)
const favoritePage = ref(1)
const activeLabel = computed(() => tabs.find(tab => tab.value === activeTab.value)?.label ?? '')
const currentPage = computed(() => activeTab.value === 'favorites' ? favoritePage.value : resumePage.value)
const pageSize = computed(() => activeTab.value === 'resume' ? 6 : 12)
const hiddenIds = ref(new Set<string>())
const pendingIds = ref(new Set<string>())
const updateErrors = ref<Record<string, string>>({})
let mutationEpoch = 0
let disposed = false

// 同步重置页码，确保资源监听器按新服务器或标签读取请求参数。
watch([serverId, activeTab], () => {
    resumePage.value = 1
    favoritePage.value = 1
}, { flush: 'sync' })
watch([serverId, activeTab, resumePage, favoritePage], () => {
    mutationEpoch++
    hiddenIds.value = new Set()
    pendingIds.value = new Set()
    updateErrors.value = {}
}, { flush: 'sync' })

const content = useResource<HomeContent>(async () => {
    const server = serverId.value
    if (activeTab.value === 'resume') {
        return { tab: 'resume', list: await data.resume(server, (resumePage.value - 1) * 6, 6) }
    }
    if (activeTab.value === 'favorites') {
        return { tab: 'favorites', list: await data.favorites(server, (favoritePage.value - 1) * 12, 12) }
    }
    return { tab: 'count', count: await data.count(server) }
}, () => [serverId.value, activeTab.value, resumePage.value, favoritePage.value])

const resumeItems = computed(() => content.value.value?.tab === 'resume' ? content.value.value.list.Items : [])
watch([() => content.value.value, () => content.loading.value], ([value, loading]) => {
    if (!value || loading || value.tab === 'count' || value.tab !== activeTab.value) return
    const size = value.tab === 'resume' ? 6 : 12
    const lastPage = Math.max(1, Math.ceil(value.list.TotalRecordCount / size))
    const page = value.tab === 'resume' ? resumePage : favoritePage
    if (page.value > lastPage) page.value = lastPage
})
const favoriteItems = computed(() => content.value.value?.tab === 'favorites' ? content.value.value.list.Items : [])
const total = computed(() => {
    const value = content.value.value
    return value && value.tab !== 'count' ? value.list.TotalRecordCount : 0
})
const isEmpty = computed(() => activeTab.value === 'resume' ? resumeItems.value.length === 0
    : activeTab.value === 'favorites' ? favoriteItems.value.length === 0 : false)
const emptyText = computed(() => activeTab.value === 'resume' ? '暂无继续观看内容' : '暂无收藏内容')
const statistics = computed(() => {
    const count = content.value.value?.tab === 'count' ? content.value.value.count : undefined
    return [
        { label: '电影', value: count?.MovieCount ?? 0 },
        { label: '电视剧', value: count?.SeriesCount ?? 0 },
        { label: '剧集', value: count?.EpisodeCount ?? 0 },
    ]
})

function playedPercentage(item: EpisodeItem) {
    if (item.UserData?.Played) return 100
    const percentage = item.UserData?.PlayedPercentage ?? 0
    return Number.isFinite(percentage) ? Math.trunc(Math.min(100, Math.max(0, percentage))) : 0
}

function changePage(page: number) {
    if (content.loading.value || page < 1 || page > Math.max(1, Math.ceil(total.value / pageSize.value))) return
    if (activeTab.value === 'resume') resumePage.value = page
    else if (activeTab.value === 'favorites') favoritePage.value = page
}

async function toggleResume(item: EpisodeItem) {
    if (disposed || !serverId.value || !item.Id || pendingIds.value.has(item.Id)) return
    const epoch = mutationEpoch
    const server = serverId.value
    const hide = !hiddenIds.value.has(item.Id)
    pendingIds.value.add(item.Id)
    delete updateErrors.value[item.Id]
    try {
        await data.hideResume(server, item.Id, hide)
        if (disposed || epoch !== mutationEpoch) return
        // 保留当前页卡片和总数，通过同一卡片提供恢复操作。
        if (hide) hiddenIds.value.add(item.Id)
        else hiddenIds.value.delete(item.Id)
    } catch {
        if (!disposed && epoch === mutationEpoch) updateErrors.value[item.Id] = '操作失败，状态未更改，请重试。'
    } finally {
        if (!disposed && epoch === mutationEpoch) pendingIds.value.delete(item.Id)
    }
}

onUnmounted(() => {
    disposed = true
    mutationEpoch++
})
</script>

<style scoped>
.page-heading { justify-content: space-between; margin-bottom: 16px; }
.page-heading h1 { margin: 0; }
.resume-actions { display: grid; gap: 8px; margin-top: 8px; }
.resume-actions button { width: 100%; min-height: 44px; }
.resume-actions p { margin: 0; font-size: 0.85rem; }
.resume-progress { display: flex; align-items: center; gap: 8px; font-size: 0.8rem; }
.resume-progress progress { flex: 1; width: 0; height: 8px; accent-color: var(--el-color-primary); }
.statistics { display: grid; gap: 12px; margin: 0; }
.statistics > div { display: flex; justify-content: space-between; align-items: center; padding: 16px; border: 1px solid var(--el-border-color-lighter); border-radius: 12px; }
.statistics dd { margin: 0; font-size: 1.5rem; font-weight: 600; }
</style>
