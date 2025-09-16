<template>
    <el-scrollbar style="height: calc(100vh - 30px);">
        <div style="padding: 20px 32px;">
            <el-skeleton :loading="playbackInfoLoading" animated>
                <template #template>
                    <div style="width: 100%;padding: 10px;">
                        <h2><el-skeleton-item variant="h1" style="width: 50%;" /></h2>
                        <p><el-skeleton-item variant="text" style="width: 40%; margin: 5px 0;" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%; margin: 5px 0;" /></p>
                        <p><el-skeleton-item variant="button" style="width: 100%; margin: 5px 0;" /></p>
                        <p>
                            <el-skeleton-item variant="button" style="width: 30%; margin: 5px; margin-left: 0;" />
                            <el-skeleton-item variant="button" style="width: 30%; margin: 5px;" />
                            <el-skeleton-item variant="button" style="width: 30%; margin: 5px;" />
                        </p>
                        <p>
                            <el-skeleton-item variant="button" style="width: 15%; margin: 5px; margin-left: 0;" />
                            <el-skeleton-item variant="button" style="width: 15%; margin: 5px;" />
                            <el-skeleton-item variant="button" style="width: 15%; margin: 5px;" />
                        </p>
                    </div>
                </template>
                <div v-if="currentEpisodes">
                    <div style="width: 100%;">
                        <div style="display: flex; justify-content: space-between;">
                            <div>
                                <h1 v-if="currentEpisodes.Type === 'Movie'">{{ currentEpisodes.Name }}</h1>
                                <template v-else>
                                    <el-link :underline="false" @click="gotoSeries(currentEpisodes.SeriesId)"><h1>{{ currentEpisodes.SeriesName }}</h1></el-link>
                                    <p>{{ 'S' + (currentEpisodes.ParentIndexNumber || -1) + 'E' + (currentEpisodes.IndexNumber || -1) + '. ' + currentEpisodes.Name }}</p>
                                </template>
                                <p>
                                    <span>外部链接：</span>
                                    <el-tooltip v-for="externalUrl in currentEpisodes.ExternalUrls" :content="externalUrl.Url" placement="bottom" effect="light">
                                        <el-button round @click="invokeApi.open_url(externalUrl.Url)"><i-ep-Link /> {{ externalUrl.Name }}</el-button>
                                    </el-tooltip>
                                </p>
                            </div>
                            <div class="loe-logo-img">
                                <img v-lazy="useImage().images[embyServerId + ':logo:' + currentEpisodes.Id]" style="max-height: 115px; max-width: 515px;" />
                            </div>
                        </div>
                        <div style="display: flex;align-items: center;">
                            <span>总时长: {{ displayTimeLength }}</span>
                            <span style="flex: auto; margin-left: 20px;">
                                <el-progress style="width: 300px;" :percentage="currentEpisodes.UserData?.Played ? 100 : currentEpisodes.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" />
                            </span>
                        </div>
                        <div style="display: flex;align-items: center;">
                            标签：
                            <span>大小：<el-tag disable-transitions>{{ mediaSourceSizeTag }}</el-tag></span>
                            <span style="margin: 20px;">码率：<el-tag disable-transitions>{{ mediaSourceBitrateTag }}</el-tag></span>
                            <span style="margin: 20px;">分辨率：<el-tag disable-transitions>{{ mediaStreamResolutionTag }}</el-tag></span>
                            <el-button style="margin: 20px;" plain type="info" :loading="playback_info_loading" @click="getPlaybackInfo(currentEpisodes.Id)" v-if="videoOptions.length <= 1">
                                <el-icon :size="24" v-if="!playback_info_loading"><i-ep-PriceTag /></el-icon>
                                <span>获取播放信息</span>
                            </el-button>
                        </div>
                        <div>
                            版本：
                            <el-select v-model="versionSelect" @change="playbackVersionChange" size="large" style="width: 840px" :disabled="versionOptions.length <= 1">
                                <template #label="{ label }">
                                    {{ label }}
                                </template>
                                <el-option v-for="item in versionOptions" :key="item.value" :label="item.label" :value="item.value">
                                    {{ item.name }} <el-tag disable-transitions>{{ item.size || "0 KB" }}</el-tag> <el-tag disable-transitions>{{ item.bitrate || "0 Kbps" }}</el-tag> <el-tag disable-transitions>{{ item.resolution || "Unknown" }}</el-tag>
                                </el-option>
                            </el-select>
                        </div>
                        <div style="margin: 15px 0;">
                            <span>视频：
                            <el-select v-model="videoSelect" size="large" style="width: 235px" :disabled="videoOptions.length <= 1">
                                <el-option v-for="item in videoOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select></span>
                            <span style="margin-left: 15px;">音频：
                            <el-select v-model="audioSelect" size="large" style="width: 235px" :disabled="audioOptions.length <= 1">
                                <el-option v-for="item in audioOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select></span>
                            <span style="margin-left: 15px;">字幕：
                            <el-select v-model="subtitleSelect" size="large" style="width: 235px" :disabled="subtitleOptions.length <= 1">
                                <el-option v-for="item in subtitleOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select></span>
                        </div>
                        <p v-if="currentEpisodes?.SeriesId" style="display: flex; justify-content: center;">
                            <el-button plain @click="autoplay = !autoplay">
                                <span>{{ autoplay ? '连续播放' : '单集播放' }}</span>
                            </el-button>
                            <el-button plain @click="rememberSelect = !rememberSelect">
                                <span>{{ rememberSelect ? '记住媒体选项' : '自动选择媒体' }}</span>
                            </el-button>
                            <el-button v-if="supportDirectLink" plain @click="useDirectLink = (useDirectLink + 1) % 2">
                                <span>{{ useDirectLink == 2 ? '直链播放？' : useDirectLink == 1 ? '使用直链' : '不使用直链' }}</span>
                            </el-button>
                            <el-button @click="nextUp(1)">接下来</el-button>
                            <el-button @click="nextEpisode()">下一个</el-button>
                        </p>
                        <p style="display: flex; justify-content: center;">
                            <template v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0">
                                <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks, false)">
                                    <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>继续播放</span>
                                </el-button>
                                <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, 0, false)">
                                    <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>从头播放</span>
                                </el-button>
                            </template>
                            <template v-else>
                                <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, 0, false)">
                                    <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>播放</span>
                                </el-button>
                            </template>
                            <el-button plain :disabled="playedLoading" @click="played()">
                                <el-icon color="#67C23A" :size="24" :class="playedLoading ? 'is-loading' : ''" v-if="currentEpisodes.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                                <el-icon :size="24" :class="playedLoading ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                <span>已播放</span>
                            </el-button>
                            <el-button plain :disabled="starLoading" @click="star()">
                                <template v-if="currentEpisodes.UserData?.IsFavorite">
                                    <el-icon color="#E6A23C" :size="24" :class="starLoading ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                                    <span>取消收藏</span>
                                </template>
                                <template v-else>
                                    <el-icon :size="24" :class="starLoading ? 'is-loading' : ''"><i-ep-Star /></el-icon>
                                    <span>收藏</span>
                                </template>
                            </el-button>
                            <template v-if="supportDirectLink">
                                <template v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0">
                                    <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks, true)">
                                        <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                        <span>直链继续播放</span>
                                    </el-button>
                                    <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, 0, true)">
                                        <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                        <span>直链从头播放</span>
                                    </el-button>
                                </template>
                                <template v-else>
                                    <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, 0, true)">
                                        <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                        <span>直链播放</span>
                                    </el-button>
                                </template>
                            </template>
                        </p>
                    </div>
                </div>
            </el-skeleton>
            <el-skeleton :loading="nextUpLoading" animated v-if="nextUpShow">
                <template #template>
                    <h1>接下来</h1>
                    <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <el-card style="width: 300px; margin: 5px;" v-for="i in 5" :key="i">
                            <p><el-skeleton-item variant="text" style="width: 90%" /></p>
                            <p><el-skeleton-item variant="text" style="width: 60%" /></p>
                        </el-card>
                    </div>
                </template>
                <h2 v-if="nextUpList.length == 0 || (nextUpList.length == 1 && nextUpList[0].Id === currentEpisodes?.Id)">已经是最后一集了</h2>
                <div v-else style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <ItemCard v-for="nextUpItem in nextUpList" :key="nextUpItem.Id" :item="nextUpItem" :embyServerId="embyServerId" />
                </div>
            </el-skeleton>
            <el-pagination
                v-model:current-page="nextUpCurrentPage"
                v-model:page-size="nextUpPageSize"
                layout="total, prev, pager, next, jumper"
                :total="nextUpTotal"
                @current-change="handleNextUpPageChange"
                hide-on-single-page
            />
        </div>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { nextTick, ref, watchEffect } from 'vue';
import embyApi, { EmbyPageList, EpisodeItem, MediaSource, PlaybackInfo, SeriesItem, UserData } from '../../api/embyApi';
import { formatBytes, formatMbps, secondsToHMS, isInternalUrl } from '../../util/str_util'
import { getResolutionFromMediaSources } from '../../util/play_info_util'
import ItemCard from '../../components/ItemCard.vue';
import invokeApi from '../../api/invokeApi';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import { useGlobalConfig } from '../../store/db/globalConfig';
import { useImage } from '../../store/image';
import { listen } from '@tauri-apps/api/event';

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

const autoplay = ref(true)
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
            if (json.SeriesId && !currentSeries.value) {
                await getCurrentSeries()
            }
        }
    }).catch(e => ElMessage.error(e)).finally(() => playbackInfoLoading.value = false)
}

const currentSeries = ref<SeriesItem>()
async function getCurrentSeries() {
    if (!currentEpisodes.value || !currentEpisodes.value.SeriesId) {
        return
    }
    const storage = await sessionStorage.getItem(embyServerId + ':emby:' + currentEpisodes.value.SeriesId)
    if (storage) {
        currentSeries.value = JSON.parse(storage)
        return Promise.resolve()
    }
    return embyApi.items(embyServerId, currentEpisodes.value.SeriesId).then(async response => {
        let json: SeriesItem = JSON.parse(response);
        currentSeries.value = json
        sessionStorage.setItem(embyServerId + ':emby:' + currentEpisodes.value!.SeriesId, JSON.stringify(json))
        return Promise.resolve()
    }).catch(e => ElMessage.error(e))
}

const handleNextUpPageChange = (val: number) => {
    nextUpCurrentPage.value = val
    nextUp(val)
}

function nextUp(pageNumber: number, pageSize: number = nextUpPageSize.value) {
    nextUpShow.value = true
    nextUpLoading.value = true
    return embyApi.nextUp(embyServerId, currentEpisodes.value?.SeriesId!, (pageNumber - 1) * nextUpPageSize.value, pageSize).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
        nextUpList.value = json.Items
        nextUpTotal.value = json.TotalRecordCount
    }).catch(e => ElMessage.error(e)).finally(() => nextUpLoading.value = false)
}
function nextEpisode() {
    nextUp(1, 1).then(() => {
        if (nextUpList.value.length > 0) {
            router.replace({path: '/nav/emby/' + embyServerId + '/episodes/' + nextUpList.value[0].Id, query: {
                autoplay: autoplay.value ? 'true' : 'false',
                directLink: useDirectLink.value.toString(),
                rememberSelect: rememberSelect.value.toString(),
                videoSelect: videoSelect.value,
                audioSelect: audioSelect.value,
                subtitleSelect: subtitleSelect.value,
                versionSelect: versionSelect.value,
            }})
        } else {
            ElMessage.warning('已经是最后一集了')
        }
    })
}

const mediaSourceSizeTag = ref('')
const mediaSourceBitrateTag = ref('')
const mediaStreamResolutionTag = ref('Unknown')
const supportDirectLink = ref(false)
const useDirectLink = ref(2)
function handleMediaSources(mediaSources: MediaSource[]) {
    if (!mediaSources || mediaSources.length == 0) {
        return
    }
    versionOptions.value = []
    let maxMediaSourceSize = 0;
    let maxVersionId = 1
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
        if (maxMediaSourceSize < mediaSource.Size) {
            maxMediaSourceSize = mediaSource.Size
            maxVersionId = i + 1
        }
    }
    if (rememberSelect.value) {
        playbackVersionChange(Number(<string>route.query.versionSelect))
    } else if (versionSelect.value > 0) {
        playbackVersionChange(versionSelect.value)
    } else {
        playbackVersionChange(maxVersionId)
    }
}

function playbackVersionChange(versionId: number) {
    let currentMediaSources = currentEpisodes.value!.MediaSources!.find(mediaSource => mediaSource.Id == versionOptions.value[versionId - 1].mediaSourceId)
    if (!currentMediaSources) {
        return
    }
    mediaSourceSizeTag.value = formatBytes(currentMediaSources.Size)
    mediaSourceBitrateTag.value = formatMbps(currentMediaSources.Bitrate)
    if (currentMediaSources.MediaStreams && currentMediaSources.MediaStreams.length > 0) {
        mediaStreamResolutionTag.value = getResolutionFromMediaSources(currentMediaSources)
    } else {
        mediaStreamResolutionTag.value = 'Unknown'
    }
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
                label: mediaStream.DisplayTitle,
                value: videoIndex
            })
            if (mediaStream.IsDefault) {
                videoSelect.value = videoIndex
            }
        } else if (mediaStream.Type == 'Audio') {
            audioIndex++
            audioOptions.value.push({
                label: mediaStream.DisplayTitle,
                value: audioIndex
            })
            if (mediaStream.IsDefault && audioSelect.value === -1) {
                audioSelect.value = audioIndex
            }
        } else if (mediaStream.Type == 'Subtitle') {
            subtitleIndex++
            subtitleOptions.value.push({
                label: mediaStream.DisplayTitle + (mediaStream.DisplayLanguage ? (" / " + mediaStream.DisplayLanguage) : ""),
                value: subtitleIndex
            })
            let score = 0;
            if (mediaStream.IsDefault) {
                score += 1
            }
            if (mediaStream.IsExternal) {
                score += 2
            }
            if (mediaStream.DisplayLanguage && mediaStream.DisplayLanguage.indexOf('Chinese Simplified') !== -1) {
                score += 3
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
    if (rememberSelect.value) {
        videoSelect.value = Number(<string>route.query.videoSelect)
        audioSelect.value = Number(<string>route.query.audioSelect)
        subtitleSelect.value = Number(<string>route.query.subtitleSelect)
    }
}

const trakt_sync_switch = ref("on")
function getTraktSyncSwitch() {
    return useGlobalConfig().getGlobalConfigValue("trakt_sync_switch").then(value => {
        trakt_sync_switch.value = value ? value : "on";
    }).catch(e => ElMessage.error('获取Trakt同步开关失败' + e))
}
getTraktSyncSwitch()
const trakt_username = ref("")
function getTraktUsername() {
    return useGlobalConfig().getGlobalConfigValue("trakt_username").then(value => {
        trakt_username.value = value;
    }).catch(e => ElMessage.error('获取Trakt同步开关失败' + e))
}
getTraktUsername()

function getScrobbleTraktParam(playbackPositionTicks: number) {
    if (trakt_sync_switch.value != 'on' || !trakt_username.value) {
        return
    }
    const type = currentEpisodes.value!.Type == 'Movie' ? 'movie' : 'episode'
    const progress = Number((playbackPositionTicks / (runTimeTicks.value / 100)).toFixed(2))
    let param: any = {[type]: {}, progress}
    let ids = getScrobbleTraktIdsParam(currentEpisodes.value!)
    if (ids.imdb || ids.tmdb || ids.tvdb || ids.trakt) {
        param[type].ids = ids
        return param
    }
    if (currentSeries.value && currentEpisodes.value?.IndexNumber != undefined && currentEpisodes.value?.ParentIndexNumber != undefined) {
        ids = getScrobbleTraktIdsParam(currentSeries.value!)
        if (ids.imdb || ids.tmdb || ids.tvdb || ids.trakt) {
            param.show = {ids}
            param[type].season = currentEpisodes.value?.ParentIndexNumber
            param[type].number = currentEpisodes.value?.IndexNumber
            return param
        }
    }
}
function getScrobbleTraktIdsParam(item: EpisodeItem | SeriesItem) {
    let ids: {[key in 'imdb' | 'tmdb' | 'tvdb' | 'trakt']?: string} = {}
    for (const [key, value] of Object.entries(item.ProviderIds)) {
        let provider = key.toLowerCase()
        switch (provider) {
            case 'imdb': ids.imdb = value; break;
            case 'tmdb': ids.tmdb = value; break;
            case 'tvdb': ids.tvdb = value; break;
            case 'trakt': ids.trakt = value; break;
        }
    }
    for (let externalUrl of item.ExternalUrls) {
        if (externalUrl.Url.startsWith("https://www.imdb.com")) {
            let url = new URL(externalUrl.Url)
            if (!url.pathname.endsWith("/") && !ids.imdb) {
                ids.imdb = url.pathname.substring(url.pathname.lastIndexOf("/") + 1)
            }
        } else if (externalUrl.Url.startsWith("https://www.themoviedb.org")) {
            let url = new URL(externalUrl.Url)
            if (!url.pathname.endsWith("/") && !ids.tmdb) {
                ids.tmdb = url.pathname.substring(url.pathname.lastIndexOf("/") + 1)
            }
        } else if (externalUrl.Url.startsWith("https://thetvdb.com")) {
            let url = new URL(externalUrl.Url)
            if (url.searchParams.get("id") && !ids.tvdb) {
                ids.tvdb = url.searchParams.get("id")!
            }
        } else if (externalUrl.Url.startsWith("https://trakt.tv/search/")) {
            let url = new URL(externalUrl.Url)
            const path = url.pathname.split('/')
            if (path.length === 4 && ['imdb', 'tmdb', 'tvdb', 'trakt'].indexOf(path[2]) !== -1 && !ids[path[2] as 'imdb' | 'tmdb' | 'tvdb' | 'trakt']) {
                ids[path[2] as 'imdb' | 'tmdb' | 'tvdb' | 'trakt'] = path[3]
            }
        }
    }
    return ids
}

const playback_info_loading = ref(false)
function getPlaybackInfo(item_id: string) {
    playback_info_loading.value = true
    return embyApi.playbackInfo(embyServerId, item_id).then(async response => {
        let playbackInfo: PlaybackInfo = JSON.parse(response);
        if (playbackInfo.ErrorCode) {
            return Promise.reject(playbackInfo.ErrorCode)
        }
        if (videoSelect.value === 0) {
            currentEpisodes.value!.MediaSources = playbackInfo.MediaSources;
            handleMediaSources(playbackInfo.MediaSources)
        }
        return playbackInfo
    }).catch(e => {
        ElMessage.error('获取播放信息失败' + e)
        return Promise.reject(e)
    }).finally(() => playback_info_loading.value = false)
}

// const playingProgressTask = ref<NodeJS.Timeout>()
function playing(item_id: string, playbackPositionTicks: number, directLink: boolean) {
    play_loading.value = true
    useDirectLink.value = directLink ? 1 : 0
    return getPlaybackInfo(item_id).then(async playbackInfo => {
        let currentMediaSources = playbackInfo.MediaSources!.find(mediaSource => mediaSource.Id == versionOptions.value[versionSelect.value - 1].mediaSourceId)
        if (!currentMediaSources) {
            ElMessage.error('未获取到播放信息')
            return
        }
        let playUrl
        if (directLink && supportDirectLink.value) {
            playUrl = currentMediaSources.Path
        } else {
            playUrl = embyApi.getDirectStreamUrl(currentMediaSources.DirectStreamUrl!)!
        }
        let externalAudio = []
        let externalSubtitle = []
        for (let mediaStream of currentMediaSources.MediaStreams) {
            if (mediaStream.Type == 'Audio' && mediaStream.IsExternal) {
                externalAudio.push(embyApi.getAudioStreamUrl(currentEpisodes.value!, currentMediaSources, mediaStream)!)
            } else if (mediaStream.Type == 'Subtitle' && mediaStream.IsExternal) {
                externalSubtitle.push(embyApi.getSubtitleStreamUrl(currentEpisodes.value!, currentMediaSources, mediaStream)!)
            }
        }
        let episodesName = currentEpisodes.value?.Type === 'Movie' ? currentEpisodes.value?.Name
                : 'S' + (currentEpisodes.value?.ParentIndexNumber || -1) + 'E' + (currentEpisodes.value?.IndexNumber || -1) + '. ' + currentEpisodes.value?.Name
        const scrobbleTraktParam = getScrobbleTraktParam(playbackPositionTicks)
        return invokeApi.playback({
            path: playUrl,
            title: episodesName + " | " + (currentEpisodes.value?.SeriesName || "🎬电影") + " | ",
            item_id: item_id,
            item_type: currentEpisodes.value!.Type || 'Movie',
            item_name: episodesName,
            emby_server_id: embyServerId,
            emby_server_name: '',
            series_id: currentEpisodes.value!.SeriesId,
            series_name: currentEpisodes.value!.SeriesName,
            media_source_id: currentMediaSources.Id,
            play_session_id: playbackInfo.PlaySessionId,
            playback_position_ticks: playbackPositionTicks,
            run_time_ticks: runTimeTicks.value ? runTimeTicks.value : 0,
            bitrate: currentMediaSources.Bitrate,
            vid: videoSelect.value,
            aid: audioSelect.value,
            sid: subtitleSelect.value,
            external_audio: externalAudio,
            external_subtitle: externalSubtitle,
            scrobble_trakt_param: JSON.stringify(scrobbleTraktParam),
            start_time: Math.round(new Date().getTime() / 1000),
        }).catch(res => ElMessage.error(res))
    }).finally(() => play_loading.value = false)
}

interface PlaybackStoppedParam {
    emby_server_id: string;
    item_id: string;
}
listen<PlaybackStoppedParam>('playingStopped', (event) => {
    console.log("tauri playingStopped event", event)
    if (embyServerId === event.payload.emby_server_id && event.payload.item_id === currentEpisodes.value?.Id) {
        updateCurrentEpisodes(true).then(async () => {
            if (currentEpisodes.value?.UserData?.Played && currentEpisodes.value.Type !== 'Movie') {
                if (autoplay.value) {
                    ElMessage.success('即将播放下一集')
                }
                nextEpisode()
            }
        })
    }
});

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

watchEffect(async () => {
    updateCurrentEpisodes().then(() => {
        if (route.query.autoplay === 'true') {
            nextTick(() => {
                playing(<string>route.params.episodeId, 0, Boolean(JSON.parse(<string>route.query.directLink)))
            })
        }
    })
})
</script>

<style scoped>
</style>