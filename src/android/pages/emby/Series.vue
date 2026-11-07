<template>
    <main class="mobile-page series-page">
        <LoadState :loading="seriesLoading" :error="seriesError" :empty="!series" empty-text="未找到系列信息" @retry="loadSeries">
            <template v-if="series">
                <DetailHeader :item="series" :server-id="serverId">
                    <p v-if="series.EndDate" class="mobile-muted">完结日期：{{ series.EndDate.slice(0, 10) }}</p>
                    <div class="mobile-actions">
                        <button type="button" :disabled="!series.UserData || pending.has(series.Id)" :aria-pressed="!!series.UserData?.IsFavorite" @click="favorite(series)">
                            {{ series.UserData?.IsFavorite ? '取消收藏' : '收藏' }}
                        </button>
                        <button type="button" :disabled="!series.UserData || pending.has(series.Id)" :aria-pressed="!!series.UserData?.Played" @click="markSeriesPlayed">
                            {{ series.UserData?.Played ? '标记未看' : '标记已看' }}
                        </button>
                    </div>
                    <p v-if="pending.has(series.Id)" class="mobile-muted" role="status">正在保存标记…</p>
                    <p v-if="!series.UserData" class="mobile-muted">暂无观看与收藏记录，标记操作不可用。</p>
                    <p v-if="errors[series.Id]" class="mobile-error" role="alert">{{ errors[series.Id] }}</p>
                </DetailHeader>

                <DetailMetadata :item="series" />

                <section class="mobile-section" aria-labelledby="series-seasons-title">
                    <h2 id="series-seasons-title">选择季</h2>
                    <div class="mobile-tabs season-tabs" role="group" aria-label="季选择">
                        <button type="button" :class="{ active: !seasonId }" :aria-pressed="!seasonId" @click="selectSeason('')">全部剧集</button>
                        <button v-for="season in seasons?.Items || []" :key="season.Id" type="button" :class="{ active: seasonId === season.Id }" :aria-pressed="seasonId === season.Id" @click="selectSeason(season.Id)">
                            {{ season.Name || `第 ${season.IndexNumber ?? '?'} 季` }}<template v-if="season.UserData?.UnplayedItemCount"> · {{ season.UserData.UnplayedItemCount }} 集未看</template>
                        </button>
                    </div>
                    <LoadState :loading="seasonsLoading" :error="seasonsError" :empty="!seasons?.Items?.length" empty-text="暂无季信息，仍可浏览全部剧集" @retry="loadSeasons">
                        <p class="mobile-muted">{{ selectedSeason?.Name || '正在浏览全部剧集' }}</p>
                        <p v-if="selectedSeason?.Overview" class="season-overview">{{ selectedSeason.Overview }}</p>
                        <template v-if="selectedSeason">
                            <p v-if="selectedSeason.UserData" class="mobile-muted">{{ selectedSeason.UserData.UnplayedItemCount || 0 }} 集未看</p>
                            <div class="mobile-actions">
                                <button type="button" :disabled="!selectedSeason.UserData || pending.has(selectedSeason.Id)" :aria-pressed="!!selectedSeason.UserData?.IsFavorite" @click="favorite(selectedSeason)">{{ selectedSeason.UserData?.IsFavorite ? '取消收藏本季' : '收藏本季' }}</button>
                                <button type="button" :disabled="!selectedSeason.UserData || pending.has(selectedSeason.Id)" :aria-pressed="!!selectedSeason.UserData?.Played" @click="markSeasonPlayed">{{ selectedSeason.UserData?.Played ? '本季标记未看' : '本季标记已看' }}</button>
                            </div>
                            <p v-if="errors[selectedSeason.Id]" class="mobile-error" role="alert">{{ errors[selectedSeason.Id] }}</p>
                        </template>
                    </LoadState>
                </section>

                <section class="mobile-section" aria-labelledby="series-episodes-title" :aria-busy="episodesLoading">
                    <div class="section-heading">
                        <h2 id="series-episodes-title">{{ selectedSeason?.Name || '全部剧集' }}</h2>
                        <span v-if="episodes" class="mobile-muted">共 {{ episodeTotal }} 集</span>
                    </div>
                    <LoadState :loading="episodesLoading" :error="episodesError" :empty="!episodes?.Items?.length" empty-text="当前季暂无剧集" @retry="loadEpisodes">
                        <div class="mobile-grid">
                            <MediaCard v-for="episode in episodes?.Items || []" :key="`${serverId}:${episode.Id}`" :item="episode" :server-id="serverId" :detail-query="{ season: seasonId, page: String(page) }">
                                <p class="episode-specs">S{{ episode.ParentIndexNumber ?? '?' }}E{{ episode.IndexNumber ?? '?' }}<template v-if="episode.PremiereDate"> · {{ episode.PremiereDate.slice(0, 10) }}</template></p>
                                <p v-for="(source, index) in episode.MediaSources || []" :key="source.Id || index" class="episode-specs">{{ source.Name || `版本 ${index + 1}` }} · {{ source.Size > 0 ? formatBytes(source.Size) : '大小未知' }} · {{ getResolutionFromMediaSources(source) }}</p>
                                <div class="mobile-actions">
                                    <button type="button" :disabled="!episode.UserData || pending.has(episode.Id)" :aria-pressed="!!episode.UserData?.IsFavorite" @click="favorite(episode)">{{ episode.UserData?.IsFavorite ? '取消收藏' : '收藏' }}</button>
                                    <button type="button" :disabled="!episode.UserData || pending.has(episode.Id)" :aria-pressed="!!episode.UserData?.Played" @click="played(episode)">{{ episode.UserData?.Played ? '标记未看' : '标记已看' }}</button>
                                </div>
                                <p v-if="errors[episode.Id]" class="mobile-error" role="alert">{{ errors[episode.Id] }}</p>
                            </MediaCard>
                        </div>
                    </LoadState>
                    <Pagination v-if="episodeTotal > pageSize" :page="page" :total="episodeTotal" :page-size="pageSize" :loading="episodesLoading" @change="changePage" />
                </section>
            </template>
        </LoadState>
    </main>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { EmbyPageList, EpisodeItem, SeasonItem, SeriesItem } from '../../../api/embyApi'
import { useBrowseData } from '../../data/browse'
import { useResource } from '../../composables/useResource'
import { useMediaActions } from '../../composables/useMediaActions'
import DetailHeader from '../../components/DetailHeader.vue'
import LoadState from '../../components/LoadState.vue'
import MediaCard from '../../components/MediaCard.vue'
import Pagination from '../../components/Pagination.vue'
import DetailMetadata from '../../components/DetailMetadata.vue'
import { formatBytes } from '../../../util/str_util'
import { getResolutionFromMediaSources } from '../../../util/play_info_util'

const route = useRoute()
const router = useRouter()
const data = useBrowseData()
const param = (value: unknown) => typeof value === 'string' ? value : Array.isArray(value) ? String(value[0] || '') : ''
const serverId = computed(() => param(route.params.embyId))
const seriesId = computed(() => param(route.params.serieId))
const seasonId = ref(param(route.query.season))
const pageNumber = (value: unknown) => {
    const number = Number(param(value))
    return Number.isSafeInteger(number) && number > 0 ? number : 1
}
const page = ref(pageNumber(route.query.page))
const pageSize = 12

// 路由复用时先重置季与页码，避免对新服务器请求旧系列的分页。
watch(() => [serverId.value, seriesId.value, route.query.season, route.query.page], () => {
    seasonId.value = param(route.query.season)
    page.value = pageNumber(route.query.page)
}, { flush: 'sync' })

function syncQuery() {
    void router.replace({ path: route.path, query: { ...route.query, season: seasonId.value || undefined, page: page.value > 1 ? String(page.value) : undefined } })
}

function requireRoute() {
    if (!serverId.value || !seriesId.value) throw new Error('缺少服务器或系列编号')
}

const { value: series, loading: seriesLoading, error: seriesError, load: loadSeries } = useResource<SeriesItem>(async () => {
    requireRoute()
    return await data.item(serverId.value, seriesId.value) as unknown as SeriesItem
}, () => [serverId.value, seriesId.value])

const { value: seasons, loading: seasonsLoading, error: seasonsError, load: loadSeasons } = useResource<EmbyPageList<SeasonItem>>(async () => {
    requireRoute()
    return data.seasons(serverId.value, seriesId.value)
}, () => [serverId.value, seriesId.value])

const { value: episodes, loading: episodesLoading, error: episodesError, load: loadEpisodes } = useResource<EmbyPageList<EpisodeItem>>(async () => {
    requireRoute()
    return data.episodes(serverId.value, seriesId.value, seasonId.value, (page.value - 1) * pageSize, pageSize)
}, () => [serverId.value, seriesId.value, seasonId.value, page.value])

const selectedSeason = computed(() => seasons.value?.Items?.find(season => season.Id === seasonId.value))
watch(seasons, result => {
    if (result && seasonId.value && !result.Items.some(season => season.Id === seasonId.value)) { seasonId.value = ''; page.value = 1; syncQuery() }
})
const episodeTotal = computed(() => Math.max(0, episodes.value?.TotalRecordCount || 0))
// 剧集总数缩减后回退到有效末页，加载中的 undefined 不参与计算。
watch(episodes, result => {
    if (!result) return
    const lastPage = Math.max(1, Math.ceil(Math.max(0, result.TotalRecordCount || 0) / pageSize))
    if (page.value > lastPage) { page.value = lastPage; syncQuery() }
})
const { pending, errors, favorite, played } = useMediaActions(() => serverId.value)

function selectSeason(id: string) {
    if (seasonId.value === id) return
    page.value = 1
    seasonId.value = id
    syncQuery()
}

async function markSeriesPlayed() {
    const item = series.value
    const server = serverId.value
    if (!item?.UserData || pending.value.has(item.Id)) return
    await played(item)
    if (server === serverId.value && series.value === item && !errors.value[item.Id]) await Promise.all([loadSeasons(), loadEpisodes()])
}

async function markSeasonPlayed() {
    const season = selectedSeason.value
    const server = serverId.value
    if (!season?.UserData || pending.value.has(season.Id)) return
    await played(season)
    if (server === serverId.value && selectedSeason.value === season && !errors.value[season.Id]) await loadEpisodes()
}

function changePage(next: number) {
    if (episodesLoading.value || !Number.isInteger(next)) return
    if (next >= 1 && next <= Math.max(1, Math.ceil(episodeTotal.value / pageSize))) { page.value = next; syncQuery() }
}
</script>

<style scoped>
.series-page { min-width: 0; }
.series-page button { min-height: 44px; }
.season-tabs { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 12px; }
.season-tabs button { max-width: 100%; overflow-wrap: anywhere; }
.season-tabs .active { color: var(--el-color-primary); border-color: var(--el-color-primary); background: var(--el-color-primary-light-9); }
.season-overview { white-space: pre-wrap; overflow-wrap: anywhere; line-height: 1.7; }
.section-heading { display: flex; align-items: baseline; justify-content: space-between; flex-wrap: wrap; gap: 8px; }
.section-heading h2 { margin-bottom: 12px; }
.episode-specs { font-size: 12px; color: var(--el-text-color-secondary); overflow-wrap: anywhere; line-height: 1.6; }
</style>
