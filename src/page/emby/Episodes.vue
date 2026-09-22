<template>
    <div class="roe-page">
        <el-skeleton :loading="playbackInfoLoading" animated>
            <template #template>
                <div class="watch__head">
                    <el-skeleton-item variant="h1" style="width: 40%;" />
                    <el-skeleton-item variant="text" style="width: 90%; margin-top: 16px;" />
                    <el-skeleton-item variant="text" style="width: 80%; margin-top: 8px;" />
                </div>
            </template>
            <div v-if="currentEpisodes" class="watch">
                <!-- 头部：标题 + 主操作。播放是唯一实心按钮，其余退到次级样式 -->
                <header class="watch__head">
                    <div class="watch__titles">
                        <template v-if="currentEpisodes.Type === 'Movie'">
                            <h1 class="watch__title">{{ currentEpisodes.Name }}</h1>
                        </template>
                        <template v-else>
                            <el-link :underline="false" @click="gotoSeries(currentEpisodes.SeriesId)" class="watch__series">
                                {{ currentEpisodes.SeriesName }}
                            </el-link>
                            <h1 class="watch__title">
                                <span class="watch__title-num">{{ 'S' + (currentEpisodes.ParentIndexNumber || '-') + ' E' + (currentEpisodes.IndexNumber || '-') }}</span>
                                {{ currentEpisodes.Name }}
                            </h1>
                        </template>
                        <div class="watch__facts">
                            <span v-if="displayTimeLength" class="mono">{{ displayTimeLength }}</span>
                            <span v-if="mediaStreamResolutionTag && mediaStreamResolutionTag !== 'Unknown'" class="chip">{{ mediaStreamResolutionTag }}</span>
                            <span v-if="mediaSourceSizeTag" class="chip">{{ mediaSourceSizeTag }}</span>
                            <span v-if="mediaSourceBitrateTag" class="chip">{{ mediaSourceBitrateTag }}</span>
                            <span v-if="progressPercent > 0" class="watch__progress">
                                <span class="watch__progress-fill" :style="{ width: progressPercent + '%' }"></span>
                            </span>
                            <span v-if="progressPercent > 0" class="mono watch__percent">
                                {{ currentEpisodes.UserData?.Played ? '已看完' : '已看 ' + progressPercent + '%' }}
                            </span>
                        </div>
                    </div>
                    <div class="loe-logo-img watch__logo">
                        <img v-lazy="useImage().images[embyServerId + ':logo:' + currentEpisodes.Id]" />
                    </div>
                </header>

                <div class="watch__actions">
                    <template v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0 && !currentEpisodes.UserData.Played">
                        <el-button type="primary" size="large" :loading="play_loading" @click="call_player(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks)">
                            <el-icon v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                            <span>继续播放</span>
                        </el-button>
                        <el-button size="large" :loading="play_loading" @click="call_player(currentEpisodes.Id, 0)">从头播放</el-button>
                    </template>
                    <template v-else>
                        <el-button type="primary" size="large" :loading="play_loading" @click="call_player(currentEpisodes.Id, 0)">
                            <el-icon v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                            <span>播放</span>
                        </el-button>
                    </template>
                    <button class="op" :class="{ 'is-done': currentEpisodes.UserData?.Played }" :disabled="playedLoading" @click="played()">
                        <el-icon :size="16" :class="playedLoading ? 'is-loading' : ''">
                            <i-ep-CircleCheckFilled v-if="currentEpisodes.UserData?.Played" />
                            <i-ep-CircleCheck v-else />
                        </el-icon>
                        <span>{{ currentEpisodes.UserData?.Played ? '已播放' : '标记已播放' }}</span>
                    </button>
                    <button class="op" :class="{ 'is-on': currentEpisodes.UserData?.IsFavorite }" :disabled="starLoading" @click="star()">
                        <el-icon :size="16" :class="starLoading ? 'is-loading' : ''">
                            <i-ep-StarFilled v-if="currentEpisodes.UserData?.IsFavorite" />
                            <i-ep-Star v-else />
                        </el-icon>
                        <span>{{ currentEpisodes.UserData?.IsFavorite ? '取消收藏' : '收藏' }}</span>
                    </button>
                    <button class="op" :loading="play_loading" @click="call_player(currentEpisodes.Id, 0, true)">
                        <el-icon :size="16"><i-ep-Download /></el-icon>
                        <span>下载</span>
                    </button>
                </div>

                <!-- 播放配置：按“版本 → 轨道 → 策略”分三行，宽度受控，不再是狼牙棒 -->
                <section class="conf">
                    <div class="conf__row">
                        <span class="conf__key">版本</span>
                        <el-select v-model="versionSelect" @change="playbackVersionChange(versionSelect)" :disabled="versionOptions.length <= 1" class="conf__wide">
                            <el-option v-for="item in versionOptions" :key="item.value" :label="item.label" :value="item.value">
                                {{ item.name }} <el-tag disable-transitions>{{ item.size || "0 KB" }}</el-tag> <el-tag disable-transitions>{{ item.bitrate || "0 Kbps" }}</el-tag> <el-tag disable-transitions>{{ item.resolution || "Unknown" }}</el-tag>
                            </el-option>
                        </el-select>
                    </div>
                    <div class="conf__row">
                        <span class="conf__key">轨道</span>
                        <div class="conf__tracks">
                            <el-select v-model="videoSelect" :disabled="videoOptions.length <= 1" placeholder="视频">
                                <template #prefix><span class="conf__mini">视频</span></template>
                                <el-option v-for="item in videoOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select>
                            <el-select v-model="audioSelect" :disabled="audioOptions.length <= 1" placeholder="音频">
                                <template #prefix><span class="conf__mini">音频</span></template>
                                <el-option v-for="item in audioOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select>
                            <el-select v-model="subtitleSelect" :disabled="subtitleOptions.length <= 1" placeholder="字幕">
                                <template #prefix><span class="conf__mini">字幕</span></template>
                                <el-option v-for="item in subtitleOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select>
                        </div>
                    </div>
                    <div class="conf__row">
                        <span class="conf__key">策略</span>
                        <div class="conf__flags">
                            <button class="flag" :class="{ 'is-on': rememberedManual }" @click="toggleRememberSelect">
                                <el-icon :size="14"><i-ep-Pointer /></el-icon>
                                <span>{{ rememberedManual ? '手动选择媒体' : '自动选择媒体' }}</span>
                            </button>
                            <button v-if="supportDirectLink" class="flag" :class="{ 'is-on': useDirectLink }" @click="useDirectLink = !useDirectLink">
                                <el-icon :size="14"><i-ep-Connection /></el-icon>
                                <span>{{ useDirectLink ? '直链播放' : '直链已禁用' }}</span>
                            </button>
                        </div>
                    </div>
                </section>

                <!-- 章节：可跳转的时间点，横向排列 -->
                <section class="roe-section" v-if="currentEpisodes.Chapters && currentEpisodes.Chapters.length">
                    <div class="roe-section-head">
                        <h2>章节</h2>
                        <span class="roe-section-count">{{ currentEpisodes.Chapters.length }}</span>
                    </div>
                    <div class="chapters">
                        <button v-for="chapter in currentEpisodes.Chapters" :key="chapter.ChapterIndex" class="chapter" @click="call_player(currentEpisodes.Id, chapter.StartPositionTicks)">
                            <span class="chapter__time mono">{{ secondsToHMS2(chapter.StartPositionTicks / 1000_0000) }}</span>
                            <span class="chapter__name">{{ chapter.Name || chapter.MarkerType }}</span>
                        </button>
                    </div>
                </section>

                <section class="roe-section" v-if="overviewText">
                    <div class="roe-section-head"><h2>简介</h2></div>
                    <p class="overview">{{ overviewText }}</p>
                </section>

                <section class="roe-section" v-if="providerEntries.length || (currentEpisodes.ExternalUrls && currentEpisodes.ExternalUrls.length)">
                    <div class="roe-section-head"><h2>外部信息</h2></div>
                    <div class="externals">
                        <button v-for="externalUrl in currentEpisodes.ExternalUrls" :key="externalUrl.Url" class="ext" :title="externalUrl.Url" @click="invokeApi.open_url(externalUrl.Url)">
                            <svg-icon v-if="externalUrl.Url.indexOf('imdb.com') !== -1" name="imdb" size="18" />
                            <svg-icon v-else-if="externalUrl.Url.indexOf('themoviedb.org') !== -1" name="tmdb" size="18" />
                            <svg-icon v-else-if="externalUrl.Url.indexOf('thetvdb.com') !== -1" name="tvdb" size="18" />
                            <svg-icon v-else-if="externalUrl.Url.indexOf('trakt.tv') !== -1" name="trakt" size="18" />
                            <svg-icon v-else-if="externalUrl.Url.indexOf('myanimelist.net') !== -1" name="myanimelist" size="18" />
                            <img v-else-if="externalUrl.Url.indexOf('anidb.net') !== -1" src="../../icons/anidb.png" style="width: 18px; height: 18px;" />
                            <el-icon v-else :size="16"><i-ep-Link /></el-icon>
                            <span>{{ externalUrl.Name }}</span>
                        </button>
                        <span v-for="(value, key) in currentEpisodes.ProviderIds" :key="key" class="chip mono">{{ key }}:{{ value }}</span>
                    </div>
                </section>
            </div>
        </el-skeleton>

        <section class="roe-section" v-if="currentEpisodes?.Type !== 'Movie' && currentEpisodes?.SeriesId">
            <div class="roe-section-head">
                <h2>接下来</h2>
                <span class="roe-section-count">{{ nextUpTotal }}</span>
                <span class="head-ops">
                    <button class="flag" :class="{ 'is-on': nextUpShow && !episodesQueryAll }" @click="handleNextUpPageChange(1)">本季接下来</button>
                    <button class="flag" :class="{ 'is-on': nextUpShow && episodesQueryAll }" @click="handleNextUpPageChange(1, true)">本季全部</button>
                    <button class="flag" @click="nextEpisode()">下一集</button>
                </span>
            </div>
            <el-skeleton :loading="nextUpLoading" animated v-if="nextUpShow">
                <template #template>
                    <div class="grid">
                        <el-card v-for="i in 3" :key="i" style="width: 344px; height: 182px; background: transparent;" />
                    </div>
                </template>
                <div class="grid">
                    <ItemCard v-for="nextUpItem in nextUpList" :key="nextUpItem.Id" :item="nextUpItem" :embyServerId="embyServerId" />
                </div>
            </el-skeleton>
            <el-pagination
                v-if="nextUpShow"
                v-model:current-page="nextUpCurrentPage"
                v-model:page-size="nextUpPageSize"
                layout="total, prev, pager, next, jumper"
                :total="nextUpTotal"
                @current-change="handleNextUpPageChange(nextUpCurrentPage, episodesQueryAll)"
                hide-on-single-page
            />
        </section>
    </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref } from 'vue';
import embyApi, { EmbyPageList, EpisodeItem, MediaSource, UserData } from '../../api/embyApi';
import { formatBytes, formatMbps, secondsToHMS, isInternalUrl, secondsToHMS2 } from '../../util/str_util'
import { getResolutionFromMediaSources, getResolutionLevelFromMediaSources } from '../../util/play_info_util'
import ItemCard from '../../components/ItemCard.vue';
import invokeApi from '../../api/invokeApi';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import { useGlobalConfig } from '../../store/db/globalConfig';
import { useImage } from '../../store/image';
import { useEventBus } from '../../store/eventBus';
import { PlaybackNotifyParam } from '../../store/notifyCenter';

const router = useRouter()
const route = useRoute()

const embyServerId = <string>route.params.embyId

const versionOptions = ref<{label: string, value: number, mediaSourceId: string, name: string, size: string, bitrate: string, resolution: string}[]>([])
const videoOptions = ref<{label: string, value: number}[]>([])
const audioOptions = ref<{label: string, value: number}[]>([])
const subtitleOptions = ref<{label: string, value: number}[]>([])
const versionSelect = ref(-1)
const videoSelect = ref(-1)
const audioSelect = ref(-1)
const subtitleSelect = ref(-1)
const displayTimeLength = ref('')
const runTimeTicks = ref(0)

const rememberSelect = ref(route.query.rememberSelect === 'true' ? true : false)
const playbackInfoLoading = ref(false)
const play_loading = ref(false)

const nextUpShow = ref(false)
const nextUpLoading = ref(false)
const nextUpList = ref<EpisodeItem[]>([])
const nextUpCurrentPage = ref(1)
const nextUpPageSize = ref(6)
const nextUpTotal = ref(0)

const currentEpisodes = ref<EpisodeItem>()

const progressPercent = computed(() => {
    const data = currentEpisodes.value?.UserData
    if (!data) return 0
    if (data.Played) return 100
    return Math.trunc(data.PlayedPercentage || 0)
})

const overviewText = computed(() => currentEpisodes.value?.Overview || '')

const providerEntries = computed(() => Object.entries(currentEpisodes.value?.ProviderIds || {}))

/** 记住手动选择 与 文案保持一致：即时反映在按钮上 */
const rememberedManual = computed({
    get: () => rememberSelect.value,
    set: (value: boolean) => { rememberSelect.value = value },
})

function toggleRememberSelect() {
    rememberSelect.value = !rememberSelect.value
}

function updateCurrentEpisodes(silent: boolean = false) {
    if (!silent) {
        playbackInfoLoading.value = true
    }
    return embyApi.items(embyServerId, <string>route.params.episodeId).then(async response => {
        let json: EpisodeItem = JSON.parse(response);
        currentEpisodes.value = json
        if (!silent) {
            if (json.MediaSources) {
                handleMediaSources(json.MediaSources)
            }
            useImage().loadLogo(embyServerId, json)
        }
    }).catch(e => ElMessage.error(e)).finally(() => playbackInfoLoading.value = false)
}

const episodesQueryAll = ref(false)
const handleNextUpPageChange = (val: number, query_all: boolean = false) => {
    episodesQueryAll.value = query_all
    const start_item_id = query_all ? undefined : currentEpisodes.value?.Id
    nextUpCurrentPage.value = val
    nextUpShow.value = true
    nextUpLoading.value = true
    episodes((val - 1) * nextUpPageSize.value, nextUpPageSize.value, start_item_id).then(json => {
        nextUpList.value = json.Items
        nextUpTotal.value = json.TotalRecordCount
    }).finally(() => nextUpLoading.value = false)
}

function episodes(start_index: number, limit: number, start_item_id?: string) {
    return embyApi.episodes(embyServerId, currentEpisodes.value?.SeriesId!, currentEpisodes.value?.SeasonId!, start_index, limit, start_item_id).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
        return Promise.resolve(json)
    }).catch(e => {
        ElMessage.error(e)
        return Promise.reject(e)
    })
}
function nextEpisode() {
    episodes(1, 1, currentEpisodes.value?.Id).then(json => {
        if (json.Items.length < 1) {
            ElMessage.warning('已经是最后一集了')
            return
        }
        jumpToNextEpisode(json.Items[0].Id)
    })
}
function jumpToNextEpisode(id: string) {
    router.replace({path: '/nav/emby/' + embyServerId + '/episodes/' + id, query: {
        useDirectLink: useDirectLink.value.toString(),
        rememberSelect: rememberSelect.value.toString(),
        videoSelect: videoSelect.value,
        audioSelect: audioSelect.value,
        subtitleSelect: subtitleSelect.value,
        versionSelect: versionSelect.value,
    }})
}

const mediaSourceSizeTag = ref('')
const mediaSourceBitrateTag = ref('')
const mediaStreamResolutionTag = ref('Unknown')
const supportDirectLink = ref(false)
const useDirectLink = ref(route.query.useDirectLink === 'true' ? true : false)
function handleMediaSources(mediaSources: MediaSource[]) {
    if (!mediaSources || mediaSources.length == 0) {
        return
    }
    versionOptions.value = []
    let versionSelectList: {versionId: number, size: number, resolutionLevel: number}[] = [];
    for (let i = 0; i < mediaSources.length; i++) {
        let mediaSource = mediaSources[i]
        versionOptions.value.push({
            label: mediaSource.Name,
            value: i + 1,
            mediaSourceId: mediaSource.Id,
            name: mediaSource.Name,
            size: formatBytes(mediaSource.Size),
            bitrate: formatMbps(mediaSource.Bitrate),
            resolution: getResolutionFromMediaSources(mediaSource),
        })
        versionSelectList.push({
            versionId: i + 1,
            size: mediaSource.Size,
            resolutionLevel: getResolutionLevelFromMediaSources(mediaSource)
        })
    }
    // 已经选择了版本（记住了选择或手动选择），则不更新版本
    if (versionSelect.value > 0) {
        playbackVersionChange(versionSelect.value)
    } else {
        // 自动选择版本
        if (play_version_auto_select_policy.value === 'high-bitrate') {
            versionSelectList.sort((a, b) => b.size - a.size)
        } else if (play_version_auto_select_policy.value === 'high-resolution') {
            versionSelectList.sort((a, b) => {
                if (a.resolutionLevel !== b.resolutionLevel) return b.resolutionLevel - a.resolutionLevel
                else if (a.size !== b.size) return b.size - a.size
                else return 0
            })
        }
        playbackVersionChange(versionSelectList[0].versionId, true)
    }
}

function playbackVersionChange(versionId: number, firstTime: boolean = false) {
    let currentMediaSources = currentEpisodes.value!.MediaSources!.find(mediaSource => mediaSource.Id == versionOptions.value[versionId - 1].mediaSourceId)
    if (!currentMediaSources) {
        return
    }
    mediaSourceSizeTag.value = formatBytes(currentMediaSources.Size)
    mediaSourceBitrateTag.value = formatMbps(currentMediaSources.Bitrate)
    mediaStreamResolutionTag.value = getResolutionFromMediaSources(currentMediaSources)
    if (currentMediaSources.IsRemote && currentMediaSources.Path && currentMediaSources.Path.indexOf('://') !== -1 && !isInternalUrl(currentMediaSources.Path)) {
        supportDirectLink.value = true
    }
    versionSelect.value = versionId
    videoSelect.value = -1
    audioSelect.value = -1
    subtitleSelect.value = -1
    videoOptions.value = []
    audioOptions.value = []
    subtitleOptions.value = []
    runTimeTicks.value = currentMediaSources.RunTimeTicks
    displayTimeLength.value = secondsToHMS(currentMediaSources.RunTimeTicks / 1000_0000)
    let videoIndex = 0
    let audioIndex = 0
    let subtitleIndex = 0
    let subtitleScore = 0
    for (let mediaStream of currentMediaSources.MediaStreams) {
        if (mediaStream.Type == 'Video') {
            videoIndex++
            videoOptions.value.push({
                label: mediaStream.DisplayTitle + (mediaStream.IsExternal ? ' (外置)' : ''),
                value: videoIndex
            })
            if (mediaStream.IsDefault) {
                videoSelect.value = videoIndex
            }
        } else if (mediaStream.Type == 'Audio') {
            audioIndex++
            audioOptions.value.push({
                label: mediaStream.DisplayTitle + (mediaStream.IsExternal ? ' (外置)' : ''),
                value: audioIndex
            })
            if (mediaStream.IsDefault && audioSelect.value === -1) {
                audioSelect.value = audioIndex
            }
        } else if (mediaStream.Type == 'Subtitle') {
            subtitleIndex++
            subtitleOptions.value.push({
                label: mediaStream.DisplayTitle + (mediaStream.DisplayLanguage ? (" / " + mediaStream.DisplayLanguage) : "") + (mediaStream.IsExternal ? ' (外置)' : ''),
                value: subtitleIndex
            })
            let score = 0;
            if (mediaStream.IsDefault) {
                score += 1
            }
            if (mediaStream.IsExternal) {
                score += 2
            }
            if (mediaStream.DisplayLanguage) {
                if (mediaStream.DisplayLanguage.indexOf('Chinese') !== -1) {
                    score += 3;
                    if (mediaStream.DisplayLanguage.indexOf('Simplified') !== -1) {
                        score += 1;
                    }
                }
                if (mediaStream.DisplayLanguage.indexOf("中") !== -1) {
                    score += 3;
                    if (mediaStream.DisplayLanguage.indexOf("简") !== -1) {
                        score += 1;
                    }
                }
            }
            if (score > subtitleScore) {
                subtitleScore = score
                subtitleSelect.value = subtitleIndex
            }
        }
    }
    if (videoOptions.value.length > 0) {
        videoOptions.value.push({
            label: '关闭',
            value: -1
        })
    } else {
        videoOptions.value.push({
            label: '自动',
            value: 0
        })
        videoSelect.value = 0
    }
    if (audioOptions.value.length > 0) {
        audioOptions.value.push({
            label: '关闭',
            value: -1
        })
    } else {
        audioOptions.value.push({
            label: '自动',
            value: 0
        })
        audioSelect.value = 0
    }
    if (subtitleOptions.value.length > 0) {
        subtitleOptions.value.push({
            label: '关闭',
            value: -1
        })
    } else {
        subtitleOptions.value.push({
            label: '自动',
            value: 0
        })
        subtitleSelect.value = 0
    }
    if (videoSelect.value === -1 && videoOptions.value.length > 1) {
        videoSelect.value = videoOptions.value[0].value
    }
    if (audioSelect.value === -1 && audioOptions.value.length > 1) {
        audioSelect.value = audioOptions.value[0].value
    }
    if (subtitleSelect.value === -1 && subtitleOptions.value.length > 1) {
        subtitleSelect.value = subtitleOptions.value[0].value
    }
    if (rememberSelect.value && firstTime) {
        videoSelect.value = Number(<string>route.query.videoSelect)
        audioSelect.value = Number(<string>route.query.audioSelect)
        subtitleSelect.value = Number(<string>route.query.subtitleSelect)
    }
}

const play_version_auto_select_policy = ref<string>('');
function getPlayVersionAutoSelectPolicy() {
    useGlobalConfig().getGlobalConfigValue("play_version_auto_select_policy").then(value => {
        play_version_auto_select_policy.value = value ? value : "high-resolution";
    }).catch(e => ElMessage.error('获取播放版本自动选择策略失败' + e))
}
getPlayVersionAutoSelectPolicy()

function call_player(item_id: string, playbackPositionTicks: number, download: boolean = false) {
    play_loading.value = true
    return invokeApi.call_player({
        emby_server_id: embyServerId,
        series_id: currentEpisodes.value?.SeriesId,
        item_id: item_id,
        playback_position_ticks: playbackPositionTicks,
        use_direct_link: useDirectLink.value,
        select_policy: rememberSelect.value ? 'manual' : 'auto',
        video_select: videoSelect.value,
        audio_select: audioSelect.value,
        subtitle_select: subtitleSelect.value,
        version_select: versionSelect.value,
        download: download,
    }).catch(res => ElMessage.error(res)).finally(() => play_loading.value = false)
}

async function playingNotify(payload: PlaybackNotifyParam) {
    if (embyServerId === payload.emby_server_id) {
        if (payload.item_id === currentEpisodes.value?.Id && payload.event === 'stop') {
            updateCurrentEpisodes(true)
        } else if (payload.series_id && payload.series_id === currentEpisodes.value?.SeriesId && payload.item_id !== currentEpisodes.value?.Id && payload.event === 'start') {
            jumpToNextEpisode(payload.item_id)
        }
    }
}
onMounted(() => useEventBus().on('playingNotify', playingNotify))
onUnmounted(() => useEventBus().remove('playingNotify', playingNotify))

const starLoading = ref<boolean>(false)
function star() {
    if (!currentEpisodes.value?.UserData) {
        return
    }
    starLoading.value = true
    let fun;
    if (currentEpisodes.value?.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServerId, currentEpisodes.value?.Id)
    } else {
        fun = embyApi.star(embyServerId, currentEpisodes.value?.Id)
    }
    return fun.then(async response => {
        let json: UserData = JSON.parse(response);
        currentEpisodes.value!.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => ElMessage.error(e)).finally(() => starLoading.value = false)
}

const playedLoading = ref<boolean>(false)
function played() {
    if (!currentEpisodes.value?.UserData) {
        return
    }
    playedLoading.value = true
    let fun;
    if (currentEpisodes.value?.UserData.Played) {
        fun = embyApi.unplayed(embyServerId, currentEpisodes.value?.Id)
    } else {
        fun = embyApi.played(embyServerId, currentEpisodes.value?.Id)
    }
    return fun.then(async response => {
        let json: UserData = JSON.parse(response);
        currentEpisodes.value!.UserData!.Played = json.Played
    }).catch(e => ElMessage.error(e)).finally(() => playedLoading.value = false)
}

function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServerId + '/series/' + seriesId)
}

updateCurrentEpisodes().then(() => {
    if (rememberSelect.value) {
        versionSelect.value = Number(<string>route.query.versionSelect)
        videoSelect.value = Number(<string>route.query.videoSelect)
        audioSelect.value = Number(<string>route.query.audioSelect)
        subtitleSelect.value = Number(<string>route.query.subtitleSelect)
    }
})
</script>

<style scoped>
.watch__head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
}

.watch__titles {
    flex: auto;
    min-width: 0;
}

.watch__series {
    font-size: var(--text-base);
    color: var(--text-3);
}

.watch__title {
    margin: 2px 0 0;
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: 1.25;
    letter-spacing: 0.01em;
}

.watch__title-num {
    margin-right: 10px;
    font-family: var(--font-mono);
    font-size: var(--text-lg);
    color: var(--text-3);
    font-weight: 400;
}

.watch__facts {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin-top: 14px;
    font-size: var(--text-sm);
    color: var(--text-3);
}

.mono {
    font-family: var(--font-mono);
}

.chip {
    padding: 1px 7px;
    border: 1px solid var(--hairline);
    border-radius: 4px;
    font-size: var(--text-xs);
    color: var(--text-2);
    font-family: var(--font-mono);
}

.watch__progress {
    width: 180px;
    height: 4px;
    border-radius: 999px;
    background: #232A34;
    overflow: hidden;
}

.watch__progress-fill {
    display: block;
    height: 100%;
    background: var(--lamp);
}

.watch__percent {
    color: var(--text-2);
}

.watch__logo img {
    max-height: 120px;
    max-width: 340px;
    object-fit: contain;
}

.watch__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 24px;
}

.op {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 9px 14px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
}

.op:hover:not(:disabled) {
    color: var(--text-1);
    border-color: #333C48;
}

.op.is-on {
    color: var(--lamp);
    border-color: var(--lamp-line);
}

.op.is-done {
    color: #6FBF7C;
    border-color: rgba(79, 163, 94, 0.4);
}

.op:disabled {
    opacity: 0.6;
    cursor: default;
}

/* 播放配置 */
.conf {
    margin-top: 26px;
    border-top: 1px solid var(--hairline);
    padding-top: 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
}

.conf__row {
    display: flex;
    align-items: center;
    gap: 14px;
}

.conf__key {
    flex: none;
    width: 34px;
    font-size: var(--text-xs);
    color: var(--text-3);
}

.conf__wide {
    width: min(760px, 100%);
}

.conf__tracks {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
}

.conf__tracks .el-select {
    width: 224px;
}

.conf__mini {
    font-size: var(--text-xs);
    color: var(--text-3);
    margin-right: 4px;
}

.conf__flags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.flag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 11px;
    border: 1px solid var(--hairline);
    border-radius: 999px;
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-xs);
    cursor: pointer;
}

.flag:hover {
    color: var(--text-1);
    border-color: #333C48;
}

.flag.is-on {
    color: var(--lamp);
    border-color: var(--lamp-line);
    background: var(--lamp-soft);
}

/* 章节 */
.chapters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.chapter {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 11px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
}

.chapter:hover {
    color: var(--text-1);
    border-color: var(--lamp-line);
}

.chapter__time {
    color: var(--lamp);
    font-size: var(--text-xs);
}

.chapter__name {
    max-width: 22ch;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.overview {
    max-width: 76ch;
    margin: 0;
    color: var(--text-2);
    line-height: 1.8;
}

.externals {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
}

.ext {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-3);
    font-family: inherit;
    font-size: var(--text-xs);
    cursor: pointer;
}

.ext:hover {
    color: var(--text-1);
    border-color: #333C48;
}

.head-ops {
    display: flex;
    gap: 6px;
    margin-left: auto;
}

.grid {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
}
</style>
