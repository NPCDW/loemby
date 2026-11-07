<template>
    <main class="android-history">
        <header class="history-header">
            <div>
                <p class="eyebrow">LOEMBY · 观看记录</p>
                <h1>播放历史</h1>
                <p class="subtitle" aria-live="polite">{{ loading ? '正在加载记录…' : error ? '记录暂不可用' : `共 ${total} 条记录` }}</p>
            </div>
            <button class="filter-toggle" type="button" :aria-expanded="filtersOpen" aria-controls="history-filters" @click="filtersOpen = !filtersOpen">
                {{ filtersOpen ? '收起筛选' : '展开筛选' }}
            </button>
        </header>

        <form v-show="filtersOpen" id="history-filters" class="filter-panel" @submit.prevent="applyFilters">
            <label>
                <span>服务器</span>
                <select v-model="filters.emby_server_id" :disabled="serversLoading" @change="applyFilters">
                    <option value="">{{ serversLoading ? '正在加载服务器…' : '全部服务器' }}</option>
                    <option v-for="server in servers" :key="server.id" :value="server.id">{{ server.server_name || '未命名服务器' }}</option>
                </select>
            </label>
            <label><span>剧名</span><input v-model="filters.series_name" type="search" placeholder="输入剧名" /></label>
            <label><span>集名 / 电影名</span><input v-model="filters.item_name" type="search" placeholder="输入集名或电影名" /></label>
            <div class="filter-actions">
                <button class="primary-button" type="submit">查询</button>
                <button type="button" @click="resetFilters">重置</button>
            </div>
            <div v-if="serversError" class="server-error" role="alert">
                <p>{{ serversError }}</p><button type="button" :disabled="serversLoading" @click="loadServers">重试服务器列表</button>
            </div>
        </form>

        <p class="mark-help">标记顺序：正常 → 置顶 → 隐藏 → 正常。切换只更新当前记录显示，不刷新列表；隐藏不会删除记录。</p>
        <p v-if="notice" class="notice" role="status">{{ notice }}</p>
        <section class="history-results" aria-label="历史记录" :aria-busy="loading">
            <div v-if="loading" class="loading-state" role="status">
                <p>正在加载播放历史…</p>
                <div v-for="n in 3" :key="n" class="skeleton-card" aria-hidden="true"><i></i><i></i><i></i></div>
            </div>
            <div v-else-if="error" class="state-panel" role="alert">
                <h2>暂时无法获取记录</h2><p>{{ error }}</p><button class="primary-button" type="button" @click="loadHistory">重试</button>
            </div>
            <div v-else-if="!list.length" class="state-panel">
                <h2>{{ hasFilters ? '没有匹配的播放记录' : '还没有播放记录' }}</h2>
                <p>{{ hasFilters ? '试试其他关键词，或清空筛选条件。' : '播放后的历史会出现在这里。' }}</p>
                <button v-if="hasFilters" type="button" @click="resetFilters">清空筛选</button>
            </div>
            <div v-else class="history-list">
                <article v-for="(row, index) in list" :key="row.id || `missing-${index}`" class="history-card" :class="{ 'is-pinned': row.pinned === 1, 'is-hidden': row.pinned === -1 }">
                    <div class="card-meta">
                        <span class="server-name">{{ row.emby_server_name || '未知服务器' }}</span>
                        <span class="status-tag">{{ statusText(row) }}</span>
                    </div>
                    <h2>
                        <RouterLink v-if="seriesPath(row)" :to="seriesPath(row)!" class="title-link">{{ row.series_name }}</RouterLink>
                        <RouterLink v-else-if="!row.series_name && itemPath(row)" :to="itemPath(row)!" class="title-link">{{ row.item_name || '未命名媒体' }}</RouterLink>
                        <span v-else>{{ row.series_name || row.item_name || '未命名媒体' }}</span>
                    </h2>
                    <template v-if="row.series_name">
                        <RouterLink v-if="itemPath(row)" :to="itemPath(row)!" class="item-link">{{ row.item_name || '未命名媒体' }}</RouterLink>
                        <p v-else class="item-name">{{ row.item_name || '未命名媒体' }}</p>
                    </template>
                    <p v-if="!row.series_name" class="media-type">{{ row.item_type === 'Movie' ? '电影 · 无剧集信息' : '无剧集信息' }}</p>
                    <div class="card-footer">
                        <div class="duration"><span>累计播放</span><strong>{{ durationText(row) }}</strong></div>
                        <button type="button" class="mark-button" :disabled="!row.id || pendingIds.has(row.id)" :aria-label="`${row.item_name || row.series_name || '未命名媒体'}：${markText(row)}`" @click="togglePinned(row)">
                            {{ row.id && pendingIds.has(row.id) ? '更新中…' : markText(row) }}
                        </button>
                    </div>
                    <p v-if="!row.id" class="record-hint">缺少记录 ID，无法更新标记。</p>
                    <p v-if="row.id && updateErrors[row.id]" class="record-error" role="alert">{{ updateErrors[row.id] }}</p>
                </article>
            </div>
        </section>
        <nav v-if="total > 0" class="pagination" aria-label="历史分页">
            <button type="button" :disabled="loading || query.page_number <= 1" @click="changePage(query.page_number - 1)">上一页</button>
            <span aria-live="polite">{{ query.page_number }} / {{ pageCount }}</span>
            <button type="button" :disabled="loading || query.page_number >= pageCount" @click="changePage(query.page_number + 1)">下一页</button>
        </nav>
    </main>
</template>

<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useEmbyServer } from '../../store/db/embyServer'
import { usePlayHistory } from '../../store/db/playHistory'
import type { PlayHistory } from '../../store/db/playHistory'
import { secondsToHMS } from '../../util/str_util'
import { historyDataKey } from '../data/history'
import type { HistoryData } from '../data/history'
import { useHistory } from '../composables/useHistory'

const previewData = inject(historyDataKey, undefined)
const data: HistoryData = previewData ?? {
    listServers: () => useEmbyServer().listAllEmbyServer(),
    pageHistory: query => usePlayHistory().pagePlayHistory(query),
    updateHistory: (row: PlayHistory) => usePlayHistory().updatePlayHistory(row),
}
const {
    servers, serversLoading, serversError, list, total, loading, error, notice,
    pendingIds, updateErrors, filters, query, pageCount,
    loadServers, loadHistory, applyFilters, resetFilters, changePage, togglePinned,
} = useHistory(data)
const filtersOpen = ref(false)
const hasFilters = computed(() => !!(query.value.emby_server_id || query.value.series_name || query.value.item_name))

function statusText(row: PlayHistory) {
    return row.pinned === 1 ? '已置顶' : row.pinned === -1 ? '已隐藏' : '正常'
}
function markText(row: PlayHistory) {
    return !row.id ? '无法标记' : row.pinned === 1 ? '标记隐藏' : row.pinned === -1 ? '恢复正常' : '置顶记录'
}
function seriesPath(row: PlayHistory) {
    if (!row.emby_server_id || !row.series_id || !row.series_name) return undefined
    return `/nav/emby/${encodeURIComponent(row.emby_server_id)}/series/${encodeURIComponent(row.series_id)}`
}
function itemPath(row: PlayHistory) {
    if (!row.emby_server_id || !row.item_id) return undefined
    return `/nav/emby/${encodeURIComponent(row.emby_server_id)}/episodes/${encodeURIComponent(row.item_id)}`
}
function durationText(row: PlayHistory) {
    const seconds = row.played_duration
    return seconds === undefined || !Number.isFinite(seconds) || seconds < 0 ? '暂无时长' : secondsToHMS(seconds)
}
</script>

<style scoped>
.android-history {
    --history-accent: #1768c4;
    --history-accent-soft: #eaf3ff;
    --history-surface: var(--el-bg-color-overlay, #fff);
    --history-background: var(--el-bg-color-page, #f5f7fa);
    --history-text: var(--el-text-color-primary, #242b36);
    --history-muted: var(--el-text-color-regular, #606976);
    --history-border: var(--el-border-color-lighter, #e1e5eb);
    --history-danger: #b42336;
    box-sizing: border-box;
    min-height: 100vh;
    min-height: 100dvh;
    width: 100%;
    padding: max(24px, env(safe-area-inset-top)) max(16px, env(safe-area-inset-right)) max(24px, env(safe-area-inset-bottom)) max(16px, env(safe-area-inset-left));
    background: var(--history-background);
    color: var(--history-text);
    font: 15px/1.5 -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    overflow-wrap: anywhere;
    color-scheme: light;
}
:global(html.dark) .android-history {
    --history-accent: #82baff;
    --history-accent-soft: #22354b;
    --history-danger: #ff99a4;
    color-scheme: dark;
}
.android-history *, .android-history *::before, .android-history *::after { box-sizing: border-box; }
.history-header, .filter-panel, .mark-help, .notice, .history-results, .pagination { max-width: 760px; margin-left: auto; margin-right: auto; }
p, h1, h2 { margin: 0; }
.history-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 20px; }
.eyebrow { font-size: 11px; letter-spacing: 1.5px; color: var(--history-muted); }
h1 { margin-top: 6px; font-size: 28px; line-height: 1.3; }
.subtitle { margin-top: 6px; color: var(--history-muted); font-size: 13px; }
button, input, select { font: inherit; min-height: 44px; border: 1px solid var(--history-border); border-radius: 9px; }
button { padding: 10px 14px; background: var(--history-surface); color: var(--history-text); cursor: pointer; touch-action: manipulation; }
button:disabled { cursor: default; opacity: .5; }
button:focus-visible, input:focus-visible, select:focus-visible, a:focus-visible { outline: 2px solid var(--history-accent); outline-offset: 3px; }
.filter-toggle { flex-shrink: 0; color: var(--history-accent); }
.filter-panel { display: grid; gap: 14px; padding: 16px; margin-bottom: 16px; border: 1px solid var(--history-border); border-radius: 14px; background: var(--history-surface); }
label { display: grid; gap: 6px; min-width: 0; font-size: 13px; color: var(--history-muted); }
input, select { width: 100%; min-width: 0; padding: 10px 12px; background: var(--history-background); color: var(--history-text); font-size: 16px; }
.filter-actions { display: flex; gap: 10px; }
.filter-actions button { flex: 1; }
.primary-button { background: var(--history-accent); color: var(--history-surface); border-color: transparent; font-weight: 600; }
.mark-help { font-size: 12px; color: var(--history-muted); margin-bottom: 16px; }
.notice { padding: 12px; margin-bottom: 12px; background: var(--history-accent-soft); border-radius: 9px; color: var(--history-accent); }
.history-list { display: grid; gap: 12px; }
.history-card { padding: 16px; background: var(--history-surface); border: 1px solid var(--history-border); border-radius: 14px; min-width: 0; }
.history-card.is-pinned { border-color: var(--history-accent); }
.card-meta { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; margin-bottom: 8px; }
.server-name { font-size: 12px; color: var(--history-muted); min-width: 0; }
.status-tag { flex-shrink: 0; padding: 2px 8px; border-radius: 6px; background: var(--history-background); font-size: 11px; color: var(--history-muted); }
.is-pinned .status-tag { background: var(--history-accent-soft); color: var(--history-accent); }
.is-hidden .status-tag { color: var(--history-danger); }
h2 { font-size: 17px; line-height: 1.5; }
a { color: var(--history-text); text-decoration: none; }
.title-link, .item-link { display: flex; align-items: center; min-height: 44px; padding: 6px 0; }
.item-link { color: var(--history-accent); font-size: 14px; }
.item-name { padding: 6px 0; }
.media-type, .record-hint { font-size: 12px; color: var(--history-muted); margin-top: 4px; }
.card-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-top: 1px solid var(--history-border); margin-top: 12px; padding-top: 12px; }
.duration { display: grid; gap: 2px; min-width: 0; }
.duration span { font-size: 11px; color: var(--history-muted); }
.duration strong { font-size: 15px; font-variant-numeric: tabular-nums; font-weight: 600; }
.mark-button { flex-shrink: 0; font-size: 13px; }
.record-error, .server-error { color: var(--history-danger); font-size: 13px; margin-top: 12px; }
.server-error { grid-column: 1 / -1; }
.server-error button { margin-top: 8px; }
.state-panel { padding: 36px 20px; text-align: center; background: var(--history-surface); border: 1px solid var(--history-border); border-radius: 14px; }
.state-panel p { color: var(--history-muted); margin: 10px 0 20px; }
.loading-state > p { color: var(--history-muted); margin-bottom: 12px; }
.skeleton-card { padding: 20px; margin-bottom: 12px; border: 1px solid var(--history-border); border-radius: 14px; background: var(--history-surface); }
.skeleton-card i { display: block; background: var(--history-border); height: 16px; border-radius: 4px; margin-bottom: 18px; animation: history-pulse 1.3s ease-in-out infinite alternate; }
.skeleton-card i:first-child { width: 40%; }.skeleton-card i:last-child { width: 60%; margin-bottom: 0; }
.pagination { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 20px; }
.pagination span { font-size: 13px; color: var(--history-muted); font-variant-numeric: tabular-nums; }
@keyframes history-pulse { to { opacity: .4; } }
@media (prefers-reduced-motion: reduce) { .skeleton-card i { animation: none; } }
@media (min-width: 600px) { .filter-panel { grid-template-columns: repeat(2, minmax(0, 1fr)); }.filter-actions { align-self: end; } }
</style>
