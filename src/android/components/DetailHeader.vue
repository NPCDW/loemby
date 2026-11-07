<template>
    <section class="detail-header mobile-section">
        <div class="detail-hero">
            <div class="detail-cover"><img v-if="cover && !failed" :src="cover" :alt="item.Name" @error="failed = true" /><span v-else>暂无封面</span></div>
            <div class="detail-heading"><img v-if="logo && !logoFailed" class="detail-logo" :src="logo" :alt="`${item.Name} Logo`" @error="logoFailed = true" /><p class="mobile-muted">{{ typeText }}{{ item.ProductionYear ? ` · ${item.ProductionYear}` : '' }}</p><h1>{{ item.Name || '未命名媒体' }}</h1><slot /></div>
        </div>
        <div v-if="item.Overview" class="detail-overview"><p :class="{ collapsed: !expanded }">{{ item.Overview }}</p><button type="button" :aria-expanded="expanded" @click="expanded = !expanded">{{ expanded ? '收起简介' : '展开简介' }}</button></div>
        <p v-else class="mobile-muted">暂无简介</p>
    </section>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import type { EpisodeItem, SeriesItem } from '../../api/embyApi'
import { useBrowseData } from '../data/browse'
import { usePlatformData } from '../data/platform'
const props = defineProps<{ item: EpisodeItem | SeriesItem, serverId: string }>()
const data = useBrowseData()
const platform = usePlatformData()
const logo = ref<string>()
const logoFailed = ref(false)
const cover = ref<string>()
const failed = ref(false)
const expanded = ref(false)
const typeText = computed(() => props.item.Type === 'Series' ? '系列' : props.item.Type === 'Movie' ? '电影' : '剧集')
let version = 0
onUnmounted(() => { version++ })
watch(() => [props.serverId, props.item.Id], async () => {
    const current = ++version
    cover.value = undefined
    failed.value = false
    expanded.value = false
    logo.value = undefined
    logoFailed.value = false
    void platform.logo(props.serverId, props.item).then(url => { if (current === version) logo.value = url }).catch(() => { if (current === version) logoFailed.value = true })
    try {
        const url = await data.cover(props.serverId, props.item)
        if (current === version) cover.value = url
    } catch { if (current === version) failed.value = true }
}, { immediate: true })
</script>

<style scoped>
.detail-hero { display: flex; align-items: flex-start; gap: 16px; }
.detail-cover { width: 100px; aspect-ratio: 2/3; flex-shrink: 0; border-radius: 12px; background: var(--el-fill-color); display: grid; place-items: center; overflow: hidden; font-size: 12px; color: var(--el-text-color-secondary); }
.detail-cover img { width: 100%; height: 100%; object-fit: cover; }
.detail-heading { min-width: 0; flex: 1; }
.detail-logo { display: block; max-width: 100%; max-height: 80px; object-fit: contain; object-position: left; margin-bottom: 12px; }
.detail-heading h1 { font-size: 23px; line-height: 1.35; margin: 8px 0 16px; }
.detail-heading :deep(.mobile-actions) { flex-direction: column; align-items: stretch; }
.detail-overview { margin-top: 20px; }
.detail-overview p { white-space: pre-line; }
.collapsed { display: -webkit-box; -webkit-line-clamp: 4; -webkit-box-orient: vertical; overflow: hidden; }
@media (min-width: 600px) { .detail-cover { width: 150px; } .detail-heading :deep(.mobile-actions) { flex-direction: row; } }
</style>
