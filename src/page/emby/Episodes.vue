<template>
    <el-scrollbar class="episodes-scrollbar">
        <div class="episodes-page">
            <el-skeleton :loading="playbackInfoLoading" animated>
                <template #template>
                    <!-- 骨架与真实渲染保持同一结构：左信息 + 右侧 logo，避免加载完成时跳动 -->
                    <div class="episodes-skeleton">
                        <div class="eps-hero eps-skel-hero">
                            <div class="eps-main">
                                <div class="eps-title-row">
                                    <el-skeleton-item variant="h1" class="eps-title" />
                                    <div class="eps-episode-no">
                                        <el-skeleton-item variant="text" class="eps-chip-skel" />
                                        <el-skeleton-item variant="text" class="eps-episode-name-skel" />
                                    </div>
                                </div>
                                <div class="eps-meta-row">
                                    <div class="eps-meta-item">
                                        <el-skeleton-item variant="text" class="eps-meta-key-skel" />
                                        <el-skeleton-item variant="text" class="eps-time" />
                                    </div>
                                    <div class="eps-meta-item">
                                        <el-skeleton-item variant="text" class="eps-meta-key-skel" />
                                        <el-skeleton-item variant="text" class="eps-progress" />
                                    </div>
                                </div>
                                <div class="eps-tag-row">
                                    <div class="eps-tag-item">
                                        <el-skeleton-item variant="text" class="eps-meta-key-skel" />
                                        <el-skeleton-item variant="text" class="eps-tag" />
                                    </div>
                                    <div class="eps-tag-item">
                                        <el-skeleton-item variant="text" class="eps-meta-key-skel" />
                                        <el-skeleton-item variant="text" class="eps-tag" />
                                    </div>
                                    <div class="eps-tag-item">
                                        <el-skeleton-item variant="text" class="eps-meta-key-skel" />
                                        <el-skeleton-item variant="text" class="eps-tag" />
                                    </div>
                                </div>
                            </div>
                            <div class="eps-logo-wrap">
                                <el-skeleton-item variant="image" class="eps-logo" />
                            </div>
                        </div>
                        <!-- 参数卡片：一行版本 + 一行三列流选择 -->
                        <div class="eps-card eps-skel-card">
                            <div class="eps-field eps-field-version">
                                <el-skeleton-item variant="text" class="eps-label" />
                                <el-skeleton-item variant="text" class="eps-select-wide" />
                            </div>
                            <div class="eps-field eps-field-streams">
                                <el-skeleton-item variant="text" class="eps-select" />
                                <el-skeleton-item variant="text" class="eps-select" />
                                <el-skeleton-item variant="text" class="eps-select" />
                            </div>
                        </div>
                        <!-- 操作区卡片 -->
                        <div class="eps-card eps-actions-card eps-skel-card">
                            <div class="eps-switch-group">
                                <el-skeleton-item variant="button" class="eps-action" />
                                <el-skeleton-item variant="button" class="eps-action" />
                            </div>
                            <div class="eps-play-group">
                                <el-skeleton-item variant="button" class="eps-action-wide" />
                                <el-skeleton-item variant="button" class="eps-action" />
                                <el-skeleton-item variant="button" class="eps-action" />
                                <el-skeleton-item variant="button" class="eps-action" />
                            </div>
                        </div>
                        <!-- 章节卡片 -->
                        <div class="eps-card eps-tags-card eps-skel-card">
                            <el-skeleton-item variant="text" class="eps-card-title-skel" />
                            <div class="eps-tags-wrap">
                                <el-skeleton-item variant="text" class="eps-chapter-skel" />
                                <el-skeleton-item variant="text" class="eps-chapter-skel" />
                                <el-skeleton-item variant="text" class="eps-chapter-skel" />
                            </div>
                        </div>
                        <!-- 外部标签卡片 -->
                        <div class="eps-card eps-tags-card eps-skel-card">
                            <el-skeleton-item variant="text" class="eps-card-title-skel" />
                            <div class="eps-tags-wrap">
                                <el-skeleton-item variant="text" class="eps-provider-skel" />
                                <el-skeleton-item variant="text" class="eps-provider-skel" />
                            </div>
                        </div>
                        <!-- 外部链接卡片：真实渲染存在该卡片，骨架缺失会导致加载完成时整页下移 -->
                        <div class="eps-card eps-tags-card eps-skel-card">
                            <el-skeleton-item variant="text" class="eps-card-title-skel" />
                            <div class="eps-external-list">
                                <el-skeleton-item variant="image" class="eps-external-skel" />
                                <el-skeleton-item variant="image" class="eps-external-skel" />
                            </div>
                        </div>
                    </div>
                </template>
                <div v-if="currentEpisodes" class="episodes-detail">
                    <!-- 头部：剧名 / 集号 / 时长进度 / 规格标签 + 右侧 logo -->
                    <div class="eps-hero">
                        <div class="eps-main">
                            <div class="eps-title-row">
                                <h1 v-if="currentEpisodes.Type === 'Movie'" class="eps-title-text">{{ currentEpisodes.Name }}</h1>
                                <template v-else>
                                    <el-link :underline="false" @click="gotoSeries(currentEpisodes.SeriesId)" class="eps-series-link">
                                        <h1 class="eps-title-text">{{ currentEpisodes.SeriesName }}</h1>
                                    </el-link>
                                    <div class="eps-episode-no">
                                        <span class="eps-chip">{{ 'S' + (currentEpisodes.ParentIndexNumber || '-') + 'E' + (currentEpisodes.IndexNumber || '-') }}</span>
                                        <span class="eps-episode-name">{{ currentEpisodes.Name }}</span>
                                    </div>
                                </template>
                            </div>

                            <div class="eps-meta-row">
                                <div class="eps-meta-item">
                                    <span class="eps-meta-key">时长</span>
                                    <span class="eps-meta-value">{{ displayTimeLength }}</span>
                                </div>
                                <div class="eps-meta-item eps-meta-progress">
                                    <span class="eps-meta-key">进度</span>
                                    <el-progress
                                        class="eps-progress-bar"
                                        :percentage="currentEpisodes.UserData?.Played ? 100 : (currentEpisodes.UserData?.PlayedPercentage || 0)"
                                        :stroke-width="8"
                                        :format="(percentage: number) => Math.trunc(percentage) + '%'" />
                                </div>
                            </div>

                            <div class="eps-tag-row">
                                <div class="eps-tag-item">
                                    <span class="eps-meta-key">大小</span>
                                    <el-tag disable-transitions round>{{ mediaSourceSizeTag }}</el-tag>
                                </div>
                                <div class="eps-tag-item">
                                    <span class="eps-meta-key">码率</span>
                                    <el-tag disable-transitions round>{{ mediaSourceBitrateTag }}</el-tag>
                                </div>
                                <div class="eps-tag-item">
                                    <span class="eps-meta-key">分辨率</span>
                                    <el-tag disable-transitions round>{{ mediaStreamResolutionTag }}</el-tag>
                                </div>
                            </div>
                        </div>

                        <div class="eps-logo-wrap loe-logo-img">
                            <img class="eps-logo-img" v-lazy="useImage().images[embyServerId + ':logo:' + currentEpisodes.Id]" />
                        </div>
                    </div>

                    <!-- 播放参数：版本 + 视频/音频/字幕 -->
                    <div class="eps-card eps-select-card">
                        <div class="eps-field eps-field-version">
                            <span class="eps-label">版本</span>
                            <el-select v-model="versionSelect" @change="playbackVersionChange" size="large" class="eps-select-version" :disabled="versionOptions.length <= 1">
                                <template #label="{ label }">
                                    {{ label }}
                                </template>
                                <el-option v-for="item in versionOptions" :key="item.value" :label="item.label" :value="item.value">
                                    {{ item.name }} <el-tag disable-transitions>{{ item.size || "0 KB" }}</el-tag> <el-tag disable-transitions>{{ item.bitrate || "0 Kbps" }}</el-tag> <el-tag disable-transitions>{{ item.resolution || "Unknown" }}</el-tag>
                                </el-option>
                            </el-select>
                        </div>
                        <div class="eps-field eps-field-streams">
                            <div class="eps-field-item">
                                <span class="eps-label">视频</span>
                                <el-select v-model="videoSelect" size="large" class="eps-select-stream" :disabled="videoOptions.length <= 1">
                                    <el-option v-for="item in videoOptions" :key="item.value" :label="item.label" :value="item.value" />
                                </el-select>
                            </div>
                            <div class="eps-field-item">
                                <span class="eps-label">音频</span>
                                <el-select v-model="audioSelect" size="large" class="eps-select-stream" :disabled="audioOptions.length <= 1">
                                    <el-option v-for="item in audioOptions" :key="item.value" :label="item.label" :value="item.value" />
                                </el-select>
                            </div>
                            <div class="eps-field-item">
                                <span class="eps-label">字幕</span>
                                <el-select v-model="subtitleSelect" size="large" class="eps-select-stream" :disabled="subtitleOptions.length <= 1">
                                    <el-option v-for="item in subtitleOptions" :key="item.value" :label="item.label" :value="item.value" />
                                </el-select>
                            </div>
                        </div>
                    </div>

                    <!-- 操作区：开关 + 播放 + 标记 + 下载 -->
                    <div class="eps-card eps-actions-card">
                        <div class="eps-switch-group">
                            <el-button class="eps-switch-btn" :class="{ 'is-on': rememberSelect }" plain @click="rememberSelect = !rememberSelect">
                                <el-icon :size="18" v-if="rememberSelect"><i-ep-Pointer /></el-icon>
                                <el-icon :size="18" v-else><i-ep-Position /></el-icon>
                                <span>{{ rememberSelect ? '手动选择媒体' : '自动选择媒体' }}</span>
                            </el-button>
                            <el-button v-if="supportDirectLink" class="eps-switch-btn" :class="{ 'is-on': useDirectLink }" plain @click="useDirectLink = !useDirectLink">
                                <el-icon :size="18" v-if="useDirectLink"><i-ep-Connection /></el-icon>
                                <el-icon :size="18" v-else><i-ep-CircleClose /></el-icon>
                                <span>{{ useDirectLink ? '直链播放' : '禁用直链' }}</span>
                            </el-button>
                        </div>
                        <div class="eps-play-group">
                            <template v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0">
                                <el-button class="eps-primary-btn" type="primary" :loading="play_loading" @click="call_player(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks)">
                                    <el-icon :size="20" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>继续播放</span>
                                </el-button>
                                <el-button class="eps-secondary-btn" @click="call_player(currentEpisodes.Id, 0)">
                                    <el-icon :size="18"><i-ep-RefreshLeft /></el-icon>
                                    <span>从头播放</span>
                                </el-button>
                            </template>
                            <template v-else>
                                <el-button class="eps-primary-btn" type="primary" :loading="play_loading" @click="call_player(currentEpisodes.Id, 0)">
                                    <el-icon :size="20" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>播放</span>
                                </el-button>
                            </template>
                            <el-button class="eps-secondary-btn" :disabled="playedLoading" @click="played()">
                                <el-icon color="#67C23A" :size="18" :class="playedLoading ? 'is-loading' : ''" v-if="currentEpisodes.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                                <el-icon :size="18" :class="playedLoading ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                <span>已播放</span>
                            </el-button>
                            <el-button class="eps-secondary-btn" :disabled="starLoading" @click="star()">
                                <template v-if="currentEpisodes.UserData?.IsFavorite">
                                    <el-icon color="#E6A23C" :size="18" :class="starLoading ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                                    <span>取消收藏</span>
                                </template>
                                <template v-else>
                                    <el-icon :size="18" :class="starLoading ? 'is-loading' : ''"><i-ep-Star /></el-icon>
                                    <span>收藏</span>
                                </template>
                            </el-button>
                            <el-button class="eps-secondary-btn" :loading="play_loading" @click="call_player(currentEpisodes.Id, 0, true)">
                                <el-icon :size="18" v-if="!play_loading"><i-ep-Download /></el-icon>
                                <span>下载</span>
                            </el-button>
                        </div>
                    </div>

                    <!-- 章节 -->
                    <div class="eps-card eps-tags-card" v-if="currentEpisodes.Chapters && currentEpisodes.Chapters.length > 0">
                        <div class="eps-card-title">
                            <span class="eps-card-title-mark"></span>
                            <span>章节</span>
                            <span class="eps-card-title-count">{{ currentEpisodes.Chapters.length }}</span>
                        </div>
                        <div class="eps-tags-wrap">
                            <el-tag v-for="chapter in currentEpisodes.Chapters" @click="call_player(currentEpisodes.Id, chapter.StartPositionTicks)" class="eps-chapter-tag" disable-transitions>
                                <span class="eps-chapter-index">{{ chapter.ChapterIndex }}</span>
                                <span class="eps-chapter-type">{{ chapter.MarkerType }}</span>
                                <span class="eps-chapter-time">{{ secondsToHMS2(chapter.StartPositionTicks / 1000_0000) }}</span>
                                <span class="eps-chapter-name">{{ chapter.Name }}</span>
                            </el-tag>
                        </div>
                    </div>

                    <!-- 外部标签 -->
                    <div class="eps-card eps-tags-card" v-if="currentEpisodes.ProviderIds && Object.keys(currentEpisodes.ProviderIds).length > 0">
                        <div class="eps-card-title">
                            <span class="eps-card-title-mark"></span>
                            <span>外部标签</span>
                        </div>
                        <div class="eps-tags-wrap">
                            <el-tag v-for="(value, key) in currentEpisodes.ProviderIds" class="eps-provider-tag" disable-transitions>
                                <span class="eps-provider-key">{{ key }}</span>
                                <span class="eps-provider-sep">:</span>
                                <span class="eps-provider-value">{{ value }}</span>
                            </el-tag>
                        </div>
                    </div>

                    <!-- 外部链接 -->
                    <div class="eps-card eps-tags-card" v-if="currentEpisodes.ExternalUrls && currentEpisodes.ExternalUrls.length > 0">
                        <div class="eps-card-title">
                            <span class="eps-card-title-mark"></span>
                            <span>外部链接</span>
                        </div>
                        <div class="eps-external-list">
                            <el-tooltip v-for="externalUrl in currentEpisodes.ExternalUrls" :content="externalUrl.Url" placement="bottom" effect="light">
                                <button class="eps-external-btn" @click="invokeApi.open_url(externalUrl.Url)">
                                    <svg-icon v-if="externalUrl.Url.indexOf('imdb.com') !== -1" name="imdb" class="eps-external-icon" />
                                    <svg-icon v-else-if="externalUrl.Url.indexOf('themoviedb.org') !== -1" name="tmdb" class="eps-external-icon" />
                                    <svg-icon v-else-if="externalUrl.Url.indexOf('thetvdb.com') !== -1" name="tvdb" class="eps-external-icon" />
                                    <svg-icon v-else-if="externalUrl.Url.indexOf('trakt.tv') !== -1" name="trakt" class="eps-external-icon" />
                                    <svg-icon v-else-if="externalUrl.Url.indexOf('myanimelist.net') !== -1" name="myanimelist" class="eps-external-icon" />
                                    <img v-else-if="externalUrl.Url.indexOf('anidb.net') !== -1" src="../../icons/anidb.png" class="eps-external-icon" />
                                    <i-ep-Link v-else class="eps-external-icon" />
                                    <span class="eps-external-name">{{ externalUrl.Name }}</span>
                                </button>
                            </el-tooltip>
                        </div>
                    </div>
                </div>
            </el-skeleton>
        </div>
    </el-scrollbar>
</template>
<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from 'vue';
import embyApi, { EpisodeItem, MediaSource, UserData } from '../../api/embyApi';
import { formatBytes, formatMbps, secondsToHMS, isInternalUrl, secondsToHMS2 } from '../../util/str_util'
import { getResolutionFromMediaSources, getResolutionLevelFromMediaSources } from '../../util/play_info_util'
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

const currentEpisodes = ref<EpisodeItem>()
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
            router.replace({path: '/nav/emby/' + embyServerId + '/episodes/' + payload.item_id, query: {
                useDirectLink: useDirectLink.value.toString(),
                rememberSelect: rememberSelect.value.toString(),
                videoSelect: videoSelect.value,
                audioSelect: audioSelect.value,
                subtitleSelect: subtitleSelect.value,
                versionSelect: versionSelect.value,
            }})
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
/* ============================================================
 * 剧集详情播放页
 * 结构：头部信息（左信息 + 右 logo）→ 播放参数卡片 → 操作区卡片
 *       → 章节 / 外部标签 / 外部链接卡片 → 接下来列表
 * 配色沿用全局变量：卡片 --el-bg-color-overlay，边框 --el-border-color-lighter，
 * 内部分隔用 --el-border-color-extra-light，圆角统一 10px。
 * ============================================================ */
.episodes-scrollbar {
    height: calc(100vh - 30px);
}

.episodes-page {
    padding: 20px 32px 32px;
}

.episodes-detail {
    display: flex;
    flex-direction: column;
    gap: 18px;
}

/* ===== 头部 ===== */
.eps-hero {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 24px;
}

.eps-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
}

.eps-title-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
}

.eps-title-text {
    margin: 0;
    font-size: 24px;
    line-height: 1.3;
    font-weight: 600;
    color: var(--el-text-color-primary, #dcdfe6);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.eps-series-link {
    max-width: 100%;
    /* el-link 默认 justify-content: center，会把标题居中，这里改回左对齐 */
    justify-content: flex-start;
}

.eps-series-link :deep(.el-link__inner) {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.eps-series-link:hover :deep(.el-link__inner) {
    color: var(--el-color-primary, #409eff);
}

.eps-episode-no {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
}

.eps-chip {
    flex: none;
    padding: 2px 10px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--el-color-primary, #409eff);
    background-color: var(--el-color-primary-light-9, #18222c);
    border: 1px solid var(--el-color-primary-light-7, #2b3d52);
}

.eps-episode-name {
    font-size: 15px;
    color: var(--el-text-color-regular, #cfd3dc);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 元信息行：时长 / 进度 */
.eps-meta-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 28px;
}

.eps-meta-item {
    display: flex;
    align-items: center;
    gap: 10px;
}

.eps-meta-key {
    flex: none;
    font-size: 12px;
    letter-spacing: 0.5px;
    color: var(--el-text-color-secondary, #909399);
}

.eps-meta-value {
    font-size: 13px;
    color: var(--el-text-color-regular, #cfd3dc);
}

.eps-progress-bar {
    width: 240px;
}

.eps-progress-bar :deep(.el-progress__text) {
    font-size: 12px !important;
    color: var(--el-text-color-secondary, #909399);
}

.eps-progress-bar :deep(.el-progress-bar__outer) {
    background-color: var(--el-fill-color-light, #262727);
}

/* 规格标签行 */
.eps-tag-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px 24px;
}

.eps-tag-item {
    display: flex;
    align-items: center;
    gap: 8px;
}

.eps-tag-item :deep(.el-tag) {
    background-color: var(--el-fill-color-light, #262727);
    border-color: var(--el-border-color-extra-light, #2b2b2c);
    color: var(--el-text-color-regular, #cfd3dc);
}

/* 右侧 logo */
.eps-logo-wrap {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    /* 与骨架占位一致，避免加载完成时布局跳动 */
    min-height: 100px;
    max-width: 40%;
}

.eps-logo-img {
    max-height: 170px;
    max-width: 400px;
    object-fit: contain;
}

/* ===== 卡片 ===== */
.eps-card {
    background-color: var(--el-bg-color-overlay, #1c1d1f);
    border: 1px solid var(--el-border-color-lighter, #2e3034);
    border-radius: 10px;
    padding: 18px 24px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.eps-card-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 14px;
    font-size: 14px;
    font-weight: 600;
    color: var(--el-text-color-primary, #dcdfe6);
}

.eps-card-title-mark {
    width: 3px;
    height: 14px;
    border-radius: 2px;
    background: var(--el-color-primary, #409eff);
}

.eps-card-title-count {
    padding: 0 8px;
    border-radius: 9px;
    font-size: 12px;
    font-weight: 500;
    color: var(--el-text-color-secondary, #909399);
    background-color: var(--el-fill-color-light, #262727);
}

/* ===== 播放参数 ===== */
.eps-select-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.eps-field {
    display: flex;
    /* 与 .eps-field-item 的 10px 保持一致，避免版本行标签与下拉之间出现大空白 */
    gap: 24px;
}

.eps-field-version {
    align-items: center;
    gap: 10px;
}

.eps-field-streams {
    flex-wrap: wrap;
}

.eps-field-item {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
}

.eps-label {
    flex: none;
    width: 34px;
    font-size: 13px;
    color: var(--el-text-color-secondary, #909399);
}

.eps-select-version {
    width: 100%;
}

.eps-select-stream {
    width: 235px;
}

/* 下拉框：背景透明跟随卡片底色，仅保留描边 */
.eps-card :deep(.el-select__wrapper) {
    background-color: transparent;
    box-shadow: 0 0 0 1px var(--el-border-color-extra-light, #2b2b2c) inset;
    transition: box-shadow 0.2s ease;
}

.eps-card :deep(.el-select__wrapper:hover) {
    box-shadow: 0 0 0 1px var(--el-border-color, #4c4d4f) inset;
}

.eps-card :deep(.el-select__wrapper.is-focused) {
    box-shadow: 0 0 0 1px var(--el-color-primary, #409eff) inset;
}

/* 禁用态：不要亮灰块，只比正常态略压一点对比度，保持可辨别 */
.eps-card :deep(.el-select.is-disabled .el-select__wrapper) {
    background-color: var(--el-fill-color, #242424);
    box-shadow: 0 0 0 1px var(--el-border-color, #4c4d4f) inset;
}

/* 下拉浮层保持不透明，保证选项可读 */
:deep(.el-select__popper.el-popper) {
    background-color: var(--el-bg-color-overlay, #1c1d1f);
}

.eps-select-version :deep(.el-tag) {
    background-color: var(--el-fill-color-light, #262727);
    border-color: var(--el-border-color-extra-light, #2b2b2c);
    color: var(--el-text-color-secondary, #909399);
}

/* ===== 操作区 ===== */
.eps-actions-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px 24px;
    flex-wrap: wrap;
}

.eps-switch-group,
.eps-play-group {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
}

.eps-switch-btn {
    background-color: transparent;
    border-color: var(--el-border-color-extra-light, #2b2b2c);
    color: var(--el-text-color-secondary, #909399);
    transition: all 0.2s ease;
}

.eps-switch-btn:hover {
    border-color: var(--el-border-color, #4c4d4f);
    color: var(--el-text-color-regular, #cfd3dc);
}

/* 开关打开：仅用描边与文字色表达状态，避免满屏实色块 */
.eps-switch-btn.is-on {
    color: var(--el-color-primary, #409eff);
    border-color: var(--el-color-primary-light-5, #3375b9);
    background-color: var(--el-color-primary-light-9, #18222c);
}

.eps-primary-btn {
    min-width: 118px;
    font-weight: 600;
    box-shadow: 0 2px 10px rgba(64, 158, 255, 0.25);
}

.eps-secondary-btn {
    background-color: transparent;
    border-color: var(--el-border-color-extra-light, #2b2b2c);
    color: var(--el-text-color-regular, #cfd3dc);
    transition: all 0.2s ease;
}

.eps-secondary-btn:hover {
    border-color: var(--el-color-primary-light-5, #3375b9);
    color: var(--el-color-primary, #409eff);
    background-color: var(--el-color-primary-light-9, #18222c);
}

/* ===== 章节 / 外部标签 / 外部链接 ===== */
.eps-tags-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.eps-chapter-tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    cursor: pointer;
    background-color: var(--el-fill-color-light, #262727);
    border-color: var(--el-border-color-extra-light, #2b2b2c);
    color: var(--el-text-color-regular, #cfd3dc);
    transition: all 0.2s ease;
}

.eps-chapter-tag:hover {
    border-color: var(--el-color-primary-light-5, #3375b9);
    color: var(--el-color-primary, #409eff);
    background-color: var(--el-color-primary-light-9, #18222c);
}

.eps-chapter-index {
    font-size: 11px;
    font-weight: 600;
    color: var(--el-text-color-secondary, #909399);
}

.eps-chapter-type {
    font-size: 11px;
    padding: 0 5px;
    border-radius: 4px;
    background-color: var(--el-fill-color, #303030);
    color: var(--el-text-color-secondary, #909399);
}

.eps-chapter-time {
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    color: var(--el-color-primary, #409eff);
}

.eps-chapter-name {
    font-size: 12px;
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.eps-provider-tag {
    display: inline-flex;
    align-items: center;
    /* 冒号紧贴 key，value 前留一个空格宽度，形成「key: value」 */
    gap: 0;
    height: 28px;
    padding: 0 10px;
    background-color: var(--el-fill-color-light, #262727);
    border-color: var(--el-border-color-extra-light, #2b2b2c);
}

.eps-provider-key {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--el-text-color-secondary, #909399);
}

.eps-provider-sep {
    font-size: 11px;
    color: var(--el-text-color-secondary, #909399);
}

.eps-provider-sep + .eps-provider-value {
    margin-left: 4px;
}

.eps-provider-value {
    font-size: 12px;
    color: var(--el-text-color-regular, #cfd3dc);
}

.eps-external-list {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
}

.eps-external-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 92px;
    height: 92px;
    border-radius: 10px;
    cursor: pointer;
    background-color: var(--el-fill-color-light, #262727);
    border: 1px solid var(--el-border-color-extra-light, #2b2b2c);
    transition: all 0.2s ease;
}

.eps-external-btn:hover {
    border-color: var(--el-color-primary-light-5, #3375b9);
    transform: translateY(-2px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.25);
}

.eps-external-icon {
    width: 42px;
    height: 42px;
    color: var(--el-text-color-regular, #cfd3dc);
}

.eps-external-name {
    font-size: 11px;
    color: var(--el-text-color-secondary, #909399);
}

/* ===== 接下来 ===== */
.eps-nextup-card {
    margin-top: 4px;
}

.eps-section-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--el-text-color-primary, #dcdfe6);
}

.eps-section-desc {
    font-size: 12px;
    color: var(--el-text-color-secondary, #909399);
    padding-left: 11px;
}

.eps-nextup-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.eps-nextup-list {
    display: flex;
    flex-wrap: wrap;
    flex-direction: row;
}

.eps-nextup-pagination {
    margin-top: 8px;
    justify-content: flex-end;
}

/* ===== 骨架屏：与真实结构 1:1 对齐，加载完成只替换内容不位移 =====
 * 关键点：骨架行不额外加 margin/height 去「凑」真实间距，
 * 而是复用真实渲染的 .eps-title-row / .eps-meta-row / .eps-tag-row / .eps-hero
 * （后者只补 min-height），让内部 gap 自然对上。
 * 另外骨架行本身不写死 min-height，否则会撑得比内容还高（例如集号行）。 */
.episodes-skeleton {
    display: flex;
    flex-direction: column;
    gap: 18px;
}

.eps-skel-hero {
    min-height: 170px;
}

/* 标题：对齐 .eps-title-text 的 24px/1.3 ≈ 31px 行高 */
.eps-title {
    width: 50%;
    height: 31px;
    max-width: 420px;
}

/* 集号行：对齐 .eps-chip 的 2px 内边距 + 13px 行高 ≈ 21px */
.eps-chip-skel {
    width: 74px;
    height: 21px;
    border-radius: 6px;
}

.eps-episode-name-skel {
    width: 220px;
    height: 20px;
}

/* meta / tag 行的 key 占位：与 .eps-meta-key 的 12px 文字同宽同高 */
.eps-meta-key-skel {
    width: 30px;
    height: 16px;
    flex: none;
}

.eps-time {
    width: 60px;
    height: 16px;
}

.eps-progress {
    width: 240px;
    height: 16px;
}

/* 与 .eps-tag-item :deep(.el-tag) 的 24px 高度 + 4px 圆角一致 */
.eps-tag {
    width: 120px;
    height: 24px;
    border-radius: 4px;
}

.eps-logo {
    width: 400px;
    max-width: 100%;
    height: 170px;
    border-radius: 8px;
}

/* 卡片内占位：padding 与真实 .eps-card 一致（18px 24px），
 * 内部间距与真实 gap 一致，使卡片高度逐张对齐。 */
.eps-skel-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.eps-skel-card.eps-actions-card {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    gap: 16px 24px;
    flex-wrap: wrap;
    min-height: 70px;
}

.eps-skel-card.eps-tags-card {
    gap: 0;
}

/*
 * 骨架参数卡片：这些类名与真实卡片共用（.eps-field/.eps-label/.eps-select*），
 * 必须限定在 .eps-skel-card 内，否则会盖掉真实卡片的 gap/尺寸（之前的版本行大空白就是这个泄漏导致）。
 */
.eps-skel-card .eps-field {
    display: flex;
    align-items: center;
    gap: 10px;
}

.eps-skel-card .eps-label {
    width: 34px;
    height: 16px;
    flex: none;
}

.eps-skel-card .eps-select-wide {
    width: 100%;
    height: 40px;
    border-radius: 4px;
}

.eps-skel-card .eps-select {
    width: 235px;
    height: 40px;
    border-radius: 4px;
}

.eps-action {
    width: 100px;
    height: 32px;
}

.eps-action-wide {
    width: 118px;
    height: 32px;
}

.eps-card-title-skel {
    width: 72px;
    height: 16px;
    margin-bottom: 14px;
}

.eps-chapter-skel {
    width: 150px;
    height: 28px;
    border-radius: 6px;
}

.eps-provider-skel {
    width: 110px;
    height: 28px;
    border-radius: 6px;
}

/* 占位与 .eps-external-btn 同为 92x92、10px 圆角 */
.eps-external-skel {
    width: 92px;
    height: 92px;
    border-radius: 10px;
}
</style>
