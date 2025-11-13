<template>
    <el-scrollbar style="height: calc(100vh - 30px);">
        <div style="padding: 20px 32px;">
            <el-skeleton :loading="playbackInfoLoading" animated>
                <template #template>
                    <div style="width: 100%;padding: 10px;">
                        <h2><el-skeleton-item variant="h1" style="width: 50%;" /></h2>
                        <p v-for="_i in 15"><el-skeleton-item variant="text" style="width: 97%;" /></p>
                    </div>
                </template>
                <div v-if="currentEpisodes">
                    <div style="width: 100%;">
                        <div style="display: flex; justify-content: space-between;">
                            <div>
                                <h1 v-if="currentEpisodes.Type === 'Movie'">{{ currentEpisodes.Name }}</h1>
                                <template v-else>
                                    <el-link :underline="false" @click="gotoSeries(currentEpisodes.SeriesId)"><h1>{{ currentEpisodes.SeriesName }}</h1></el-link>
                                    <div>{{ 'S' + (currentEpisodes.ParentIndexNumber || '-') + 'E' + (currentEpisodes.IndexNumber || '-') + '. ' + currentEpisodes.Name }}</div>
                                </template>
                                <div style="display: flex;align-items: center;margin: 15px 0;">
                                    <span>时长：{{ displayTimeLength }}</span>
                                    <span style="flex: auto; margin-left: 5px;">
                                        <el-progress style="width: 240px;" :percentage="currentEpisodes.UserData?.Played ? 100 : currentEpisodes.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" />
                                    </span>
                                </div>
                                <div style="display: flex;align-items: center;margin: 15px 0;">
                                    标签：
                                    <span>大小：<el-tag disable-transitions>{{ mediaSourceSizeTag }}</el-tag></span>
                                    <span style="margin-left: 10px;">码率：<el-tag disable-transitions>{{ mediaSourceBitrateTag }}</el-tag></span>
                                    <span style="margin-left: 10px;">分辨率：<el-tag disable-transitions>{{ mediaStreamResolutionTag }}</el-tag></span>
                                </div>
                            </div>
                            <div class="loe-logo-img">
                                <img v-lazy="useImage().images[embyServerId + ':logo:' + currentEpisodes.Id]" style="max-height: 170px; max-width: 400px;" />
                            </div>
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
                        <p style="display: flex; justify-content: center;">
                            <el-button plain @click="rememberSelect = !rememberSelect">
                                <el-icon :size="20" v-if="rememberSelect"><i-ep-Pointer /></el-icon>
                                <el-icon :size="20" v-else><i-ep-Position /></el-icon>
                                <span>{{ rememberSelect ? '手动选择媒体' : '自动选择媒体' }}</span>
                            </el-button>
                            <el-button plain v-if="supportDirectLink" @click="useDirectLink = !useDirectLink">
                                <el-icon :size="20" v-if="useDirectLink"><i-ep-Connection /></el-icon>
                                <el-icon :size="20" v-else><i-ep-CircleClose /></el-icon>
                                <span>{{ useDirectLink ? '直链播放' : '禁用直链' }}</span>
                            </el-button>
                            <template v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0">
                                <el-button plain type="success" :loading="play_loading" @click="play_video(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks)">
                                    <el-icon :size="20" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>继续播放</span>
                                </el-button>
                                <el-button plain type="success" :loading="play_loading" @click="play_video(currentEpisodes.Id, 0)">
                                    <el-icon :size="20" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>从头播放</span>
                                </el-button>
                            </template>
                            <template v-else>
                                <el-button plain type="success" :loading="play_loading" @click="play_video(currentEpisodes.Id, 0)">
                                    <el-icon :size="20" v-if="!play_loading"><i-ep-VideoPlay /></el-icon>
                                    <span>播放</span>
                                </el-button>
                            </template>
                            <el-button plain :disabled="playedLoading" @click="played()">
                                <el-icon color="#67C23A" :size="20" :class="playedLoading ? 'is-loading' : ''" v-if="currentEpisodes.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                                <el-icon :size="20" :class="playedLoading ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                <span>已播放</span>
                            </el-button>
                            <el-button plain :disabled="starLoading" @click="star()">
                                <template v-if="currentEpisodes.UserData?.IsFavorite">
                                    <el-icon color="#E6A23C" :size="20" :class="starLoading ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                                    <span>取消收藏</span>
                                </template>
                                <template v-else>
                                    <el-icon :size="20" :class="starLoading ? 'is-loading' : ''"><i-ep-Star /></el-icon>
                                    <span>收藏</span>
                                </template>
                            </el-button>
                        </p>
                        <p>
                            <span>外部标签：</span>
                            <el-tag v-for="(value, key) in currentEpisodes.ProviderIds" style="margin-right: 10px;" disable-transitions>{{ key + ':' + value }}</el-tag>
                        </p>
                        <p>
                            <span>外部链接：</span>
                            <el-tooltip v-for="externalUrl in currentEpisodes.ExternalUrls" :content="externalUrl.Url" placement="bottom" effect="light">
                                <el-button round @click="invokeApi.open_url(externalUrl.Url)"><i-ep-Link /> {{ externalUrl.Name }}</el-button>
                            </el-tooltip>
                        </p>
                    </div>
                </div>
            </el-skeleton>
            <div v-if="currentEpisodes?.Type !== 'Movie'">
                <h1>接下来</h1>
                <p v-if="currentEpisodes?.SeriesId">
                    <el-button @click="handleNextUpPageChange(1, true)">本季所有</el-button>
                    <el-button @click="handleNextUpPageChange(1)">本季接下来</el-button>
                    <el-button @click="nextEpisode()">下一个</el-button>
                </p>
            </div>
            <el-skeleton :loading="nextUpLoading" animated v-if="nextUpShow">
                <template #template>
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
import { onMounted, onUnmounted, ref } from 'vue';
import embyApi, { EmbyPageList, EpisodeItem, MediaSource, UserData } from '../../api/embyApi';
import { formatBytes, formatMbps, secondsToHMS, isInternalUrl } from '../../util/str_util'
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

const handleNextUpPageChange = (val: number, query_all: boolean = false) => {
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

function play_video(item_id: string, playbackPositionTicks: number) {
    play_loading.value = true
    return invokeApi.play_video({
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
</style>