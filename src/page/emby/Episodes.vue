<template>
    <el-scrollbar style="height: 100vh;">
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
                        <h2 v-if="currentEpisodes.Type === 'Movie'">{{ currentEpisodes.Name }}</h2>
                        <template v-else>
                            <el-link :underline="false" @click="gotoSeries(currentEpisodes.SeriesId)"><h2>{{ currentEpisodes.SeriesName }}</h2></el-link>
                            <p>{{ 'S' + currentEpisodes.ParentIndexNumber + 'E' + currentEpisodes.IndexNumber + '. ' + currentEpisodes.Name }}</p>
                        </template>
                        <p>
                            <span>外部链接：</span>
                            <el-tooltip v-for="externalUrl in currentEpisodes.ExternalUrls" :content="externalUrl.Url" placement="bottom" effect="light">
                                <el-button round @click="invokeApi.open_url(externalUrl.Url)"><i-ep-Link /> {{ externalUrl.Name }}</el-button>
                            </el-tooltip>
                        </p>
                        <div style="display: flex;align-items: center;">
                            <span>总时长: {{ timeLength }}</span>
                            <span style="flex: auto; margin-left: 20px;">
                                <el-progress :percentage="currentEpisodes.UserData?.Played ? 100 : currentEpisodes.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" />
                            </span>
                        </div>
                        <p>
                            标签：
                            <span>大小：<el-tag disable-transitions>{{ mediaSourceSizeTag }}</el-tag></span>
                            <span style="margin: 20px;">码率：<el-tag disable-transitions>{{ mediaSourceBitrateTag }}</el-tag></span>
                            <span style="margin: 20px;">分辨率：<el-tag disable-transitions>{{ mediaStreamResolutionTag }}</el-tag></span>
                        </p>
                        <p>
                            版本：
                            <el-select v-model="versionSelect" @change="playbackVersionChange" size="large" style="width: 840px" :disabled="versionOptions.length <= 1">
                                <template #label="{ label }">
                                    {{ label }}
                                </template>
                                <el-option v-for="item in versionOptions" :key="item.value" :label="item.label" :value="item.value">
                                    {{ item.name }} <el-tag disable-transitions>{{ item.size }}</el-tag> <el-tag disable-transitions>{{ item.bitrate }}</el-tag> <el-tag disable-transitions>{{ item.resolution }}</el-tag>
                                </el-option>
                            </el-select>
                        </p>
                        <p>
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
                        </p>
                        <template v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0">
                            <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks)">
                                <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                <span>继续播放</span>
                            </el-button>
                            <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, 0)">
                                <el-icon :size="24" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                <span>从头播放</span>
                            </el-button>
                        </template>
                        <template v-else>
                            <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, 0)">
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
                        <el-button plain @click="continuousPlay = !continuousPlay">
                            <span>{{ continuousPlay ? '连续播放' : '单集播放' }}</span>
                        </el-button>
                        <el-button v-if="continuousPlay" plain @click="rememberSelect = !rememberSelect">
                            <span>{{ rememberSelect ? '记住选择' : '自动选择' }}</span>
                        </el-button>
                        <el-button @click="nextUp(1)">接下来</el-button>
                    </div>
                </div>
            </el-skeleton>
            <h1 v-if="nextUpLoading">接下来</h1>
            <el-skeleton :loading="nextUpLoading" animated>
                <template #template>
                    <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <el-card style="width: 300px; margin: 5px;" v-for="i in 5" :key="i">
                            <p><el-skeleton-item variant="text" style="width: 90%" /></p>
                            <p><el-skeleton-item variant="text" style="width: 60%" /></p>
                        </el-card>
                    </div>
                </template>
                <h1 v-if="nextUpList && nextUpList.length >= 1">接下来</h1>
                <h1 v-if="nextUpList && nextUpList.length < 1">已经是最后一集了</h1>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <ItemCard v-for="nextUpItem in nextUpList" :key="nextUpItem.Id" :item="nextUpItem" :embyServer="embyServer" />
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
import { h, nextTick, onUnmounted, ref, VNode, watchEffect } from 'vue';
import embyApi, { EmbyPageList, EpisodesItems, MediaSources, PlaybackInfo, UserData } from '../../api/embyApi';
import { formatBytes, formatMbps, secondsToHMS } from '../../util/str_util'
import { getResolutionFromMediaSources } from '../../util/play_info_util'
import ItemCard from '../../components/ItemCard.vue';
import invokeApi from '../../api/invokeApi';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage, ElNotification } from 'element-plus';
import { PlaybackProgress } from '../../store/playback';
import { EmbyServer, useEmbyServer } from '../../store/db/embyServer';
import { useProxyServer } from '../../store/db/proxyServer';
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'
import { useGlobalConfig } from '../../store/db/globalConfig';
import { useEventBus } from '../../store/eventBus';
import traktApi from '../../api/traktApi';

const router = useRouter()
const route = useRoute()

let embyServer = ref<EmbyServer>({})
async function getEmbyServer() {
    return useEmbyServer().getEmbyServer(<string>route.params.embyId).then(value => {
        embyServer.value = value!;
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
function embyServerChanged(payload?: {event?: string, id?: string}) {
    if (payload?.id === route.params.embyId) {
        getEmbyServer()
    }
}
useEventBus().on('EmbyServerChanged', embyServerChanged)
onUnmounted(() => useEventBus().remove('EmbyServerChanged', embyServerChanged))

const versionOptions = ref<{label: string, value: string, name: string, size: string, bitrate: string, resolution: string}[]>([])
const videoOptions = ref<{label: string, value: number}[]>([])
const audioOptions = ref<{label: string, value: number}[]>([])
const subtitleOptions = ref<{label: string, value: number}[]>([])
const versionSelect = ref('')
const videoSelect = ref(-1)
const audioSelect = ref(-1)
const subtitleSelect = ref(-1)
const timeLength = ref('')
const runTimeTicks = ref(0)
const mpv_path = ref('')

const continuousPlay = ref(true)
const rememberSelect = ref(route.query.rememberSelect === 'true' ? true : false)
const playbackInfoLoading = ref(false)
const play_loading = ref(false)

const nextUpLoading = ref(false)
const nextUpList = ref<EpisodesItems[]>([])
const nextUpCurrentPage = ref(1)
const nextUpPageSize = ref(6)
const nextUpTotal = ref(0)

watchEffect(async () => {
    await getEmbyServer()
    updateCurrentEpisodes().then(() => {
        if (route.query.autoplay === 'true') {
            nextTick(() => {
                playing(<string>route.params.episodeId, 0)
            })
        }
    })
})

useGlobalConfig().getGlobalConfigValue("mpv_path").then(value => {
    mpv_path.value = value;
}).catch(e => ElMessage.error('获取全局播放代理失败' + e))

const currentEpisodes = ref<EpisodesItems>()
function updateCurrentEpisodes(silent: boolean = false) {
    if (!silent) {
        playbackInfoLoading.value = true
    }
    return embyApi.items(embyServer.value, <string>route.params.episodeId).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EpisodesItems = JSON.parse(response.body);
        currentEpisodes.value = json
        if (json.SeriesId && !currentSeries.value) {
            getCurrentSeries()
        }
        if (!silent && json.MediaSources) {
            handleMediaSources(json.MediaSources)
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => playbackInfoLoading.value = false)
}

const currentSeries = ref<EpisodesItems>()
async function getCurrentSeries() {
    if (!currentEpisodes.value || !currentEpisodes.value.SeriesId) {
        return
    }
    return embyApi.items(embyServer.value, currentEpisodes.value.SeriesId).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EpisodesItems = JSON.parse(response.body);
        currentSeries.value = json
    }).catch(e => {
        ElMessage.error(e)
    })
}

const handleNextUpPageChange = (val: number) => {
    nextUpCurrentPage.value = val
    nextUp(val)
}

function nextUp(pageNumber: number) {
    nextUpLoading.value = true
    return embyApi.nextUp(embyServer.value, currentEpisodes.value?.SeriesId!, (pageNumber - 1) * nextUpPageSize.value, nextUpPageSize.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        nextUpList.value = json.Items
        nextUpTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => nextUpLoading.value = false)
}

const mediaSourceSizeTag = ref('')
const mediaSourceBitrateTag = ref('')
const mediaStreamResolutionTag = ref('Unknown')
function handleMediaSources(mediaSources: MediaSources[]) {
    if (!mediaSources || mediaSources.length == 0) {
        return
    }
    versionOptions.value = []
    let max = 0;
    let maxMediaSource = mediaSources[0]
    for (let mediaSource of mediaSources) {
        versionOptions.value.push({
            label: mediaSource.Name,
            value: mediaSource.Id,
            name: mediaSource.Name,
            size: formatBytes(mediaSource.Size),
            bitrate: formatMbps(mediaSource.Bitrate),
            resolution: getResolutionFromMediaSources(mediaSource),
        })
        if (max < mediaSource.Size) {
            max = mediaSource.Size
            maxMediaSource = mediaSource
        }
    }
    playbackVersionChange(maxMediaSource.Id)
}

function playbackVersionChange(mediaSourceId: string) {
    let currentMediaSources = currentEpisodes.value!.MediaSources!.find(mediaSource => mediaSource.Id == mediaSourceId)
    if (!currentMediaSources) {
        return
    }
    mediaSourceSizeTag.value = formatBytes(currentMediaSources.Size)
    mediaSourceBitrateTag.value = formatMbps(currentMediaSources.Bitrate)
    if (currentMediaSources.MediaStreams && currentMediaSources.MediaStreams.length > 0) {
        mediaStreamResolutionTag.value = getResolutionFromMediaSources(currentMediaSources)
    }
    versionSelect.value = mediaSourceId
    videoSelect.value = -1
    audioSelect.value = -1
    subtitleSelect.value = -1
    videoOptions.value = []
    audioOptions.value = []
    subtitleOptions.value = []
    runTimeTicks.value = currentMediaSources.RunTimeTicks
    timeLength.value = secondsToHMS(currentMediaSources.RunTimeTicks / 1000_0000)
    let videoIndex = 0
    let audioIndex = 0
    let subtitleIndex = 0
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
            if (mediaStream.IsDefault) {
                audioSelect.value = audioIndex
            }
        } else if (mediaStream.Type == 'Subtitle') {
            subtitleIndex++
            subtitleOptions.value.push({
                label: mediaStream.DisplayTitle,
                value: subtitleIndex
            })
            if (mediaStream.DisplayLanguage && mediaStream.DisplayLanguage.indexOf('Chinese Simplified') !== -1) {
                subtitleSelect.value = subtitleIndex
            } else if (mediaStream.IsDefault && subtitleSelect.value === -1) {
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

function getScrobbleTraktParam(playbackPositionTicks: number) {
    if (trakt_sync_switch.value == 'off') {
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
function getScrobbleTraktIdsParam(item: EpisodesItems) {
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

// const playingProgressTask = ref<NodeJS.Timeout>()
function playing(item_id: string, playbackPositionTicks: number) {
    if (!mpv_path) {
        ElMessage.error('未设置mpv路径')
        return
    }
    play_loading.value = true
    return embyApi.playbackInfo(embyServer.value, item_id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let playbackInfo: PlaybackInfo = JSON.parse(response.body);
        for (const mediaSource of currentEpisodes.value!.MediaSources!) {
            if (!mediaSource.MediaStreams || mediaSource.MediaStreams.length == 0 || !mediaSource.MediaStreams.find(mediaStream => mediaStream.Type == 'Video')) {
                currentEpisodes.value!.MediaSources = playbackInfo.MediaSources;
                handleMediaSources(playbackInfo.MediaSources)
                break
            }
        }
        let currentMediaSources = playbackInfo.MediaSources!.find(mediaSource => mediaSource.Id == versionSelect.value)
        if (currentMediaSources) {
            let directStreamUrl = embyApi.getDirectStreamUrl(embyServer.value, currentMediaSources.DirectStreamUrl!)!
            let externalAudio = []
            let externalSubtitle = []
            for (let mediaStream of currentMediaSources.MediaStreams) {
                if (mediaStream.Type == 'Audio' && mediaStream.IsExternal) {
                    externalAudio.push(embyApi.getAudioStreamUrl(embyServer.value, currentMediaSources, mediaStream)!)
                } else if (mediaStream.Type == 'Subtitle' && mediaStream.IsExternal) {
                    externalSubtitle.push(embyApi.getSubtitleStreamUrl(embyServer.value, currentMediaSources, mediaStream)!)
                }
            }
            let episodesName = currentEpisodes.value?.Type === 'Movie' ? currentEpisodes.value?.Name
                 : 'S' + currentEpisodes.value?.ParentIndexNumber + 'E' + currentEpisodes.value?.IndexNumber + '. ' + currentEpisodes.value?.Name
            const scrobbleTraktParam = getScrobbleTraktParam(playbackPositionTicks)
            return invokeApi.playback({
                mpv_path: mpv_path.value,
                path: directStreamUrl,
                proxy: await useProxyServer().getPlayProxyUrl(embyServer.value.play_proxy_id),
                title: episodesName + " | " + currentEpisodes.value?.SeriesName + " | " + embyServer.value.server_name,
                user_agent: embyServer.value!.user_agent!,
                server_id: embyServer.value!.id!,
                item_id: item_id,
                media_source_id: currentMediaSources.Id,
                play_session_id: playbackInfo.PlaySessionId,
                playback_position_ticks: playbackPositionTicks,
                run_time_ticks: runTimeTicks.value,
                vid: videoSelect.value,
                aid: audioSelect.value,
                sid: subtitleSelect.value,
                external_audio: externalAudio,
                external_subtitle: externalSubtitle,
                scrobble_trakt_param: JSON.stringify(scrobbleTraktParam),
            }).then(async () => {
                embyApi.playing(embyServer.value!, item_id, currentMediaSources.Id, playbackInfo.PlaySessionId, playbackPositionTicks).then(() => {
                    ElMessage.success({
                        message: '开始播放，请稍候'
                    })
                    useEmbyServer().updateEmbyServer({id: embyServer.value!.id!, last_playback_time: dayjs().locale('zh-cn').format('YYYY-MM-DD HH:mm:ss')})
                        .then(() => useEventBus().emit('EmbyServerChanged', {event: 'update', id: embyServer.value!.id!}))
                })
                if (scrobbleTraktParam) {
                    traktApi.start(scrobbleTraktParam).then(response => {
                        if (response.status_code != 201) {
                            ElMessage.error('Trakt 同步失败：' + response.status_code + ' ' + response.status_text)
                            return
                        }
                        const json: {progress: number, movie?: {title: string, year: number}, episode?: {title: string, season: number, number: number}, show?: {title: string, year: number}} = JSON.parse(response.body);
                        let message: VNode[] = []
                        if (json.movie) {
                            message = [h('p', null, `${json.movie.title} (${json.movie.year})`)]
                        } else if (json.episode) {
                            message = [h('p', null, `${json.show?.title} (${json.show?.year})`), h('p', null, `S${json.episode.season}E${json.episode.number} ${json.episode.title}`)]
                        }
                        ElNotification.success({
                            title: 'Trakt 同步播放',
                            message: h('div', null, message),
                            position: 'bottom-right',
                        })
                    }).catch(e => ElMessage.error("Trakt 同步失败：" + e))
                }
            }).catch(res => {
                ElMessage.error({
                    message: res
                })
            })
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => play_loading.value = false)
}

function playingStopped(payload: PlaybackProgress) {
    if (embyServer.value.id === payload.server_id && payload.item_id === currentEpisodes.value?.Id) {
        updateCurrentEpisodes(true).then(() => {
            if (currentEpisodes.value?.UserData?.Played && currentEpisodes.value.Type !== 'Movie' && continuousPlay.value) {
                ElMessage.success({
                    message: '播放完成，即将播放下一集'
                })
                nextUpCurrentPage.value = 1
                nextUp(1).then(() => {
                    if (nextUpList.value.length > 0) {
                        router.replace({path: '/nav/emby/' + embyServer.value.id + '/episodes/' + nextUpList.value[0].Id, query: {
                            autoplay: 'true',
                            rememberSelect: rememberSelect.value.toString(),
                            videoSelect: videoSelect.value,
                            audioSelect: audioSelect.value,
                            subtitleSelect: subtitleSelect.value,
                            versionSelect: versionSelect.value,
                        }})
                    } else {
                        ElMessage.warning({
                            message: '已经是最后一集了'
                        })
                    }
                })
            }
        })
    }
}
useEventBus().on('playingStopped', playingStopped)
onUnmounted(() => useEventBus().remove('playingStopped', playingStopped))

const starLoading = ref<boolean>(false)
function star() {
    if (!currentEpisodes.value?.UserData) {
        return
    }
    starLoading.value = true
    let fun;
    if (currentEpisodes.value?.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServer.value, currentEpisodes.value?.Id)
    } else {
        fun = embyApi.star(embyServer.value, currentEpisodes.value?.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        currentEpisodes.value!.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => starLoading.value = false)
}

const playedLoading = ref<boolean>(false)
function played() {
    if (!currentEpisodes.value?.UserData) {
        return
    }
    playedLoading.value = true
    let fun;
    if (currentEpisodes.value?.UserData.Played) {
        fun = embyApi.unplayed(embyServer.value, currentEpisodes.value?.Id)
    } else {
        fun = embyApi.played(embyServer.value, currentEpisodes.value?.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        currentEpisodes.value!.UserData!.Played = json.Played
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => playedLoading.value = false)
}

function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/series/' + seriesId)
}
</script>

<style scoped>
</style>