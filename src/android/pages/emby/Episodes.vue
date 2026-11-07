<template>
    <main class="mobile-page episodes-page">
        <LoadState :loading="loading" :error="error" :empty="!item" empty-text="未找到媒体信息" @retry="load">
            <template v-if="item">
                <nav v-if="isEpisode" class="series-navigation" aria-label="所属系列">
                    <RouterLink v-if="item.SeriesId" :to="seriesPath">返回系列：{{ item.SeriesName || '查看系列' }}</RouterLink>
                    <span v-else class="mobile-muted">暂无所属系列信息</span>
                </nav>
                <DetailHeader :item="item" :server-id="serverId">
                    <p v-if="isEpisode" class="episode-meta">{{ item.SeriesName || '未知系列' }} · S{{ item.ParentIndexNumber ?? '?' }}E{{ item.IndexNumber ?? '?' }}</p>
                    <p v-if="item.SeasonName" class="mobile-muted">{{ item.SeasonName }}</p>
                    <p v-if="item.PremiereDate" class="mobile-muted">首映：{{ item.PremiereDate.slice(0, 10) }}</p>
                    <div class="mobile-actions">
                        <button type="button" :disabled="!item.UserData || pending.has(item.Id)" :aria-pressed="!!item.UserData?.IsFavorite" @click="favorite(item)">
                            {{ item.UserData?.IsFavorite ? '取消收藏' : '收藏' }}
                        </button>
                        <button type="button" :disabled="!item.UserData || pending.has(item.Id)" :aria-pressed="!!item.UserData?.Played" @click="played(item)">
                            {{ item.UserData?.Played ? '标记未看' : '标记已看' }}
                        </button>
                    </div>
                    <p v-if="pending.has(item.Id)" class="mobile-muted" role="status">正在保存标记…</p>
                    <p v-if="!item.UserData" class="mobile-muted">暂无观看与收藏记录，标记操作不可用。</p>
                    <p v-if="errors[item.Id]" class="mobile-error" role="alert">{{ errors[item.Id] }}</p>
                </DetailHeader>

                <DetailMetadata :item="item" />

                <section class="mobile-section" aria-labelledby="episode-progress-title">
                    <h2 id="episode-progress-title">{{ isMovie ? '电影' : isEpisode ? '单集' : '媒体' }}观看进度</h2>
                    <template v-if="item.UserData">
                        <div class="progress-heading">
                            <span>{{ item.UserData.Played ? '已看' : progress > 0 || position > 0 ? '观看中' : '未看' }}</span>
                            <span>{{ Math.round(progress) }}%</span>
                        </div>
                        <progress class="watch-progress" :value="progress" max="100" aria-label="观看进度">{{ Math.round(progress) }}%</progress>
                        <p class="mobile-muted">上次播放位置：{{ formatTicks(position) }} · 当前版本时长：{{ formatTicks(source?.RunTimeTicks) }}</p>
                    </template>
                    <p v-else class="mobile-muted">暂无观看进度</p>
                    <div class="mobile-actions playback-actions">
                        <button type="button" :disabled="preparing" @click="play()">{{ position > 0 ? '从头播放' : '播放' }}</button>
                        <button v-if="position > 0" type="button" :disabled="preparing" @click="play(true)">续播</button>
                    </div>
                    <p v-if="preparing" class="mobile-muted" role="status">正在准备播放…</p>
                    <p v-if="playError" class="mobile-error" role="alert">{{ playError }}</p>
                    <p v-if="refreshing" class="mobile-muted" role="status">正在刷新观看进度…</p>
                    <div v-if="refreshError" class="mobile-actions">
                        <p class="mobile-error" role="alert">{{ refreshError }}</p>
                        <button type="button" :disabled="refreshing || pending.has(item.Id)" @click="refreshProgress">重试刷新进度</button>
                    </div>
                    <p class="mobile-muted">播放将启动 Android 原生播放器，按下方选择的策略播放；同季剧集支持连续播放。</p>
                </section>

                <section class="mobile-section" aria-labelledby="episode-media-title">
                    <h2 id="episode-media-title">媒体版本与轨道信息</h2>
                    <label class="selection-toggle"><input v-model="rememberSelect" type="checkbox" :disabled="preparing" @change="selectionModeChange" />手动选择（本次连播沿用选择）</label>
                    <p class="mobile-muted">{{ rememberSelect ? '按当前版本及轨道播放；切换版本后请确认轨道。' : '自动模式按全局版本策略及默认音轨、中文字幕优选规则播放。更改下拉会切换为手动模式。' }}</p>
                    <p v-if="policyError" class="mobile-error" role="alert">{{ policyError }}</p>
                    <p v-if="!sources.length" class="mobile-muted">暂无媒体源信息；播放时将由服务器重新获取可用媒体源。</p>
                    <template v-else>
                        <label class="version-field">
                            <span>媒体版本（共 {{ sources.length }} 个）</span>
                            <select v-model.number="versionIndex" :disabled="preparing || sources.length < 2" aria-label="选择媒体版本" @change="manualVersionChange">
                                <option v-for="(version, index) in sources" :key="`${version.Id || 'source'}:${index}`" :value="index">{{ version.Name || `版本 ${index + 1}` }} · {{ formatSize(version.Size) }} · {{ formatBitrate(version.Bitrate) }} · {{ getResolutionFromMediaSources(version) }}</option>
                            </select>
                        </label>
                        <template v-if="source">
                            <dl class="source-details">
                                <div><dt>时长</dt><dd>{{ formatTicks(source?.RunTimeTicks) }}</dd></div>
                                <div><dt>大小</dt><dd>{{ formatSize(source.Size) }}</dd></div>
                                <div><dt>码率</dt><dd>{{ formatBitrate(source.Bitrate) }}</dd></div>
                                <div><dt>分辨率</dt><dd>{{ getResolutionFromMediaSources(source) }}</dd></div>
                                <div><dt>容器</dt><dd>{{ source.Container || '未知' }}</dd></div>
                                <div><dt>来源</dt><dd>{{ source.IsRemote ? '远程媒体' : '媒体文件' }}</dd></div>
                            </dl>
                            <label v-if="supportDirectLink" class="selection-toggle"><input v-model="useDirectLink" type="checkbox" :disabled="preparing" />直链播放</label>
                            <p v-else class="mobile-muted">当前版本不支持直链播放。</p>
                            <section v-for="group in streamGroups" :key="group.type" class="stream-group" :aria-label="`${group.label}信息`">
                                <h3>{{ group.label }} <span class="mobile-muted">({{ group.streams.length }})</span></h3>
                                <label class="version-field">
                                    <span>选择{{ group.label }}</span>
                                    <select :value="trackSelection(group.type)" :disabled="preparing || group.options.length < 2" :aria-label="`选择${group.label}轨道`" @change="selectTrack(group.type, $event)">
                                        <option v-for="option in group.options" :key="option.value" :value="option.value">{{ option.label }}</option>
                                    </select>
                                </label>
                                <p v-if="!group.streams.length" class="mobile-muted">暂无{{ group.label }}信息</p>
                                <ul v-else class="stream-list">
                                    <li v-for="(stream, index) in group.streams" :key="`${stream.Index ?? index}:${index}`">
                                        <strong>{{ stream.DisplayTitle || stream.Title || `${group.label}轨道 ${index + 1}` }}</strong>
                                        <p class="mobile-muted">{{ describeStream(stream) }}</p>
                                    </li>
                                </ul>
                            </section>
                        </template>
                    </template>
                </section>

                <section class="mobile-section" aria-labelledby="episode-chapters-title">
                    <h2 id="episode-chapters-title">章节 <span class="mobile-muted">({{ chapters.length }})</span></h2>
                    <p v-if="!chapters.length" class="mobile-muted">暂无章节信息</p>
                    <template v-else>
                        <p class="mobile-muted">选择章节后，将使用当前播放选项从该位置开始。</p>
                        <ol class="chapter-list">
                            <li v-for="(chapter, index) in chapters" :key="`${chapter.ChapterIndex ?? index}:${index}`">
                                <button type="button" class="chapter-button" :disabled="preparing || !isValidTicks(chapter.StartPositionTicks)" @click="play(false, chapter.StartPositionTicks)">
                                    <span class="chapter-time">{{ formatTicks(chapter.StartPositionTicks) }}</span>
                                    <span>
                                        <strong>{{ chapter.Name || `章节 ${index + 1}` }}</strong>
                                        <span class="chapter-description mobile-muted">章节编号：{{ chapter.ChapterIndex ?? index + 1 }}<template v-if="chapter.MarkerType"> · {{ chapter.MarkerType }}</template></span>
                                    </span>
                                </button>
                            </li>
                        </ol>
                    </template>
                </section>
            </template>
        </LoadState>
    </main>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import type { EpisodeItem, MediaStream } from '../../../api/embyApi'
import { formatBytes, formatMbps, secondsToHMS2 } from '../../../util/str_util'
import { useBrowseData } from '../../data/browse'
import { useResource } from '../../composables/useResource'
import { useMediaActions } from '../../composables/useMediaActions'
import { usePlayback } from '../../composables/usePlayback'
import DetailHeader from '../../components/DetailHeader.vue'
import LoadState from '../../components/LoadState.vue'
import DetailMetadata from '../../components/DetailMetadata.vue'
import { usePlatformData } from '../../data/platform'
import { getResolutionFromMediaSources } from '../../../util/play_info_util'
import { preferredTracks, preferredVersion, selectionQuery, supportsDirectLink, trackOptions, validTrack } from '../../util/playbackSelection'
import type { TrackType } from '../../util/playbackSelection'

const route = useRoute()
const router = useRouter()
const platform = usePlatformData()
const data = useBrowseData()
const param = (value: unknown) => typeof value === 'string' ? value : Array.isArray(value) ? String(value[0] || '') : ''
const serverId = computed(() => param(route.params.embyId))
const itemId = computed(() => param(route.params.episodeId))
const versionIndex = ref(0)
const rememberSelect = ref(false)
const useDirectLink = ref(false)
const videoSelect = ref(0)
const audioSelect = ref(0)
const subtitleSelect = ref(0)
const versionPolicy = ref('high-resolution')
const policyError = ref('')
let disposed = false
onUnmounted(() => { disposed = true })
void platform.versionPolicy().then(policy => {
    if (disposed) return
    versionPolicy.value = policy || 'high-resolution'
    if (!rememberSelect.value) applyAutomaticSelection()
}).catch(() => { if (!disposed) policyError.value = '读取版本策略失败，暂按高分辨率优先。' })
watch(() => [serverId.value, itemId.value], () => {
    versionIndex.value = 0
    rememberSelect.value = false
    useDirectLink.value = false
    videoSelect.value = audioSelect.value = subtitleSelect.value = 0
}, { flush: 'sync' })

const { value: item, loading, error, load } = useResource<EpisodeItem>(async () => {
    if (!serverId.value || !itemId.value) throw new Error('缺少服务器或媒体编号')
    return data.item(serverId.value, itemId.value)
}, () => [serverId.value, itemId.value])
const { pending, errors, favorite, played } = useMediaActions(() => serverId.value)
const selection = () => ({ rememberSelect: rememberSelect.value, useDirectLink: useDirectLink.value, versionIndex: versionIndex.value, videoSelect: videoSelect.value, audioSelect: audioSelect.value, subtitleSelect: subtitleSelect.value })
const { preparing, error: playError, refreshing, refreshError, play, refreshProgress } = usePlayback({
    serverId: () => serverId.value,
    itemId: () => itemId.value,
    item: () => item.value,
    selection,
    followEpisode: notification => router.replace({
        path: `/nav/emby/${encodeURIComponent(serverId.value)}/episodes/${encodeURIComponent(notification.item_id)}`,
        query: { ...returnQuery(), ...selectionQuery(selection()) },
    }),
})

const isMovie = computed(() => item.value?.Type === 'Movie')
const isEpisode = computed(() => item.value?.Type === 'Episode')
const seriesPath = computed(() => ({
    path: `/nav/emby/${encodeURIComponent(serverId.value)}/series/${encodeURIComponent(item.value?.SeriesId || '')}`,
    query: { season: typeof route.query.season === 'string' ? route.query.season : item.value?.SeasonId || '', page: param(route.query.page) || '1' },
}))
const sources = computed(() => item.value?.MediaSources || [])
const source = computed(() => sources.value[versionIndex.value] || sources.value[0])
const chapters = computed(() => item.value?.Chapters || [])
const duration = computed(() => validNumber(source.value?.RunTimeTicks))
const position = computed(() => validNumber(item.value?.UserData?.PlaybackPositionTicks))
const progress = computed(() => {
    const user = item.value?.UserData
    if (user?.Played) return 100
    const reported = validNumber(user?.PlayedPercentage)
    const inferred = duration.value > 0 ? position.value / duration.value * 100 : 0
    return Math.min(100, reported > 0 ? reported : inferred)
})
const streamGroups = computed(() => ([
    { type: 'Video', label: '视频' },
    { type: 'Audio', label: '音频' },
    { type: 'Subtitle', label: '字幕' },
] as const).map(group => ({
    ...group,
    streams: (source.value?.MediaStreams || []).filter(stream => stream.Type === group.type),
    options: trackOptions(source.value, group.type),
})))
const supportDirectLink = computed(() => supportsDirectLink(source.value))
watch(versionIndex, () => { applyPreferredTracks(); if (!supportDirectLink.value) useDirectLink.value = false }, { flush: 'sync' })
watch(() => [item.value, route.query.rememberSelect, route.query.useDirectLink, route.query.versionSelect, route.query.videoSelect, route.query.audioSelect, route.query.subtitleSelect], () => {
    if (!item.value) return
    rememberSelect.value = route.query.rememberSelect === 'true'
    const version = Number(route.query.versionSelect)
    versionIndex.value = rememberSelect.value && Number.isInteger(version) && version > 0 && version <= sources.value.length
        ? version - 1 : preferredVersion(sources.value, versionPolicy.value)
    applyPreferredTracks()
    if (rememberSelect.value) {
        videoSelect.value = validTrack(route.query.videoSelect, source.value, 'Video', videoSelect.value)
        audioSelect.value = validTrack(route.query.audioSelect, source.value, 'Audio', audioSelect.value)
        subtitleSelect.value = validTrack(route.query.subtitleSelect, source.value, 'Subtitle', subtitleSelect.value)
    }
    useDirectLink.value = supportDirectLink.value && route.query.useDirectLink === 'true'
}, { flush: 'sync' })

function applyPreferredTracks() {
    const tracks = preferredTracks(source.value)
    videoSelect.value = tracks.videoSelect
    audioSelect.value = tracks.audioSelect
    subtitleSelect.value = tracks.subtitleSelect
}

function applyAutomaticSelection() {
    versionIndex.value = preferredVersion(sources.value, versionPolicy.value)
    applyPreferredTracks()
}

function selectionModeChange() {
    if (!rememberSelect.value) applyAutomaticSelection()
}

function manualVersionChange() { rememberSelect.value = true }
function trackSelection(type: TrackType) { return type === 'Video' ? videoSelect.value : type === 'Audio' ? audioSelect.value : subtitleSelect.value }
function selectTrack(type: TrackType, event: Event) {
    const value = Number((event.target as HTMLSelectElement).value)
    if (!trackOptions(source.value, type).some(option => option.value === value)) return
    rememberSelect.value = true
    if (type === 'Video') videoSelect.value = value
    else if (type === 'Audio') audioSelect.value = value
    else subtitleSelect.value = value
}
function returnQuery() { return { season: param(route.query.season), page: param(route.query.page) } }
function isValidTicks(value: number | undefined) { return typeof value === 'number' && Number.isFinite(value) && value >= 0 }

function validNumber(value: number | undefined) {
    return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : 0
}

// 展示格式复用桌面工具；缺失或无效的服务端字段明确显示未知。
function formatTicks(value: number | undefined) {
    if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return '未知'
    return secondsToHMS2(value / 10_000_000)
}

function formatSize(value: number | undefined) {
    return typeof value === 'number' && validNumber(value) > 0 ? formatBytes(value) : '未知'
}

function formatBitrate(value: number | undefined) {
    return typeof value === 'number' && validNumber(value) > 0 ? formatMbps(value) : '未知'
}

function describeStream(stream: MediaStream) {
    const parts = [stream.Codec || '未知编码', stream.DisplayLanguage || stream.Language || '未知语言']
    if (stream.Type === 'Video' && stream.Width > 0 && stream.Height > 0) parts.push(`${stream.Width} × ${stream.Height}`)
    if (stream.BitRate > 0) parts.push(formatBitrate(stream.BitRate))
    if (stream.IsDefault) parts.push('默认轨道')
    if (stream.IsExternal) parts.push('外挂')
    return parts.join(' · ')
}
</script>

<style scoped>
.episodes-page { min-width: 0; }
.episodes-page button, .episodes-page select { min-height: 44px; }
.series-navigation { margin-bottom: 16px; overflow-wrap: anywhere; }
.series-navigation a { display: inline-flex; align-items: center; min-height: 44px; color: var(--el-color-primary); }
.episode-meta { overflow-wrap: anywhere; line-height: 1.6; }
.progress-heading { display: flex; justify-content: space-between; gap: 12px; }
.watch-progress { width: 100%; height: 14px; margin-top: 8px; accent-color: var(--el-color-primary); }
.playback-actions { margin-top: 16px; }
.version-field { display: grid; gap: 8px; }
.version-field select { width: 100%; min-width: 0; padding: 8px 12px; border: 1px solid var(--el-border-color); border-radius: 8px; color: var(--el-text-color-primary); background: var(--el-bg-color); }
.source-details { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; margin: 20px 0; }
.source-details dt { color: var(--el-text-color-secondary); font-size: 13px; }
.source-details dd { margin: 4px 0 0; overflow-wrap: anywhere; }
.stream-group { margin-top: 20px; }
.stream-group h3 { font-size: 16px; }
.stream-list, .chapter-list { list-style: none; margin: 0; padding: 0; }
.stream-list li, .chapter-list li { padding: 12px 0; border-bottom: 1px solid var(--el-border-color-lighter); overflow-wrap: anywhere; }
.stream-list p, .chapter-list p { margin: 6px 0 0; line-height: 1.6; }
.chapter-button { display: flex; align-items: flex-start; gap: 12px; width: 100%; text-align: left; padding: 12px; }
.chapter-button > span:last-child { min-width: 0; }
.chapter-description { display: block; margin-top: 6px; }
.selection-toggle { display: flex; align-items: center; gap: 10px; min-height: 44px; }
.selection-toggle input { width: 20px; height: 20px; flex-shrink: 0; accent-color: var(--el-color-primary); }
.chapter-time { flex-shrink: 0; font-variant-numeric: tabular-nums; color: var(--el-color-primary); }
@media (min-width: 600px) { .source-details { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
</style>
