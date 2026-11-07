<template>
    <article class="mobile-media-card">
        <RouterLink v-if="path" :to="path" class="media-link">
            <div class="media-cover"><img v-if="cover && !failed" :src="cover" :alt="item.Name" loading="lazy" @error="failed = true" /><span v-else aria-hidden="true">{{ typeText }}</span></div>
            <div class="media-info"><h3>{{ item.Name || '未命名媒体' }}</h3><p v-if="episode.SeriesName && item.Type === 'Episode'">{{ episode.SeriesName }}</p><small>{{ typeText }}{{ item.ProductionYear ? ` · ${item.ProductionYear}` : '' }}{{ item.UserData?.Played ? ' · 已看' : '' }}{{ item.UserData?.IsFavorite ? ' · 已收藏' : '' }}</small></div>
        </RouterLink>
        <div v-else class="media-info"><h3>{{ item.Name || '未命名媒体' }}</h3><p class="mobile-muted">此媒体类型暂不支持打开</p></div>
        <div v-if="$slots.default" class="media-extra"><slot /></div>
    </article>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import type { BaseItem, EpisodeItem, MediaLibraryItem } from '../../api/embyApi'
import { useBrowseData } from '../data/browse'
const props = defineProps<{ item: BaseItem | MediaLibraryItem, serverId: string, detailQuery?: Record<string, string> }>()
const data = useBrowseData()
const cover = ref<string>()
const failed = ref(false)
const episode = computed(() => props.item as EpisodeItem)
const item = computed(() => props.item as BaseItem)
const typeText = computed(() => ({ Series: '系列', Movie: '电影', Episode: '剧集', Season: '季', BoxSet: '合集', Folder: '目录', CollectionFolder: '媒体库' }[props.item.Type] || '媒体'))
const path = computed(() => {
    if (!props.item.Id || !props.serverId) return undefined
    const root = `/nav/emby/${encodeURIComponent(props.serverId)}`
    const id = encodeURIComponent(props.item.Id)
    if (props.item.Type === 'Series') return `${root}/series/${id}`
    if (['Movie', 'Episode'].includes(props.item.Type)) return { path: `${root}/episodes/${id}`, query: props.detailQuery }
    if (props.item.Type === 'Season' && episode.value.SeriesId) return { path: `${root}/series/${encodeURIComponent(episode.value.SeriesId)}`, query: { season: props.item.Id } }
    if (['BoxSet', 'Folder', 'CollectionFolder'].includes(props.item.Type)) return `${root}/mediaLibrary/items/${id}`
    return undefined
})
let version = 0
onUnmounted(() => { version++ })
watch(() => [props.serverId, props.item.Id], async () => {
    const current = ++version
    cover.value = undefined
    failed.value = false
    try {
        const url = await data.cover(props.serverId, props.item)
        if (current === version) cover.value = url
    } catch { if (current === version) failed.value = true }
}, { immediate: true })
</script>

<style scoped>
.mobile-media-card { min-width: 0; overflow: hidden; border: 1px solid var(--el-border-color-lighter); border-radius: 16px; background: var(--el-bg-color-overlay); }
.media-link { display: block; text-decoration: none; color: inherit; }
.media-cover { aspect-ratio: 2 / 3; display: grid; place-items: center; background: var(--el-fill-color); color: var(--el-text-color-secondary); overflow: hidden; }
.media-cover img { width: 100%; height: 100%; object-fit: cover; }
.media-info { padding: 12px; overflow-wrap: anywhere; }
.media-info h3 { margin: 0 0 6px; font-size: 15px; }
.media-info p, .media-info small { margin: 0; font-size: 12px; color: var(--el-text-color-secondary); }
.media-extra { padding: 0 12px 12px; }
.media-extra :deep(button) { width: 100%; }
</style>
