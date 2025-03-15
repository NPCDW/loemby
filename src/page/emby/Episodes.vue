<template>
    <el-scrollbar style="height: 100vh;">
        <div style="padding: 20px 40px;">
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
                        <div style="display: flex;">
                            <span>总时长: {{ timeLength }}</span>
                            <span style="flex: auto; margin-left: 20px;">
                                <el-progress :percentage="currentEpisodes.UserData?.Played ? 100 : currentEpisodes.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" />
                            </span>
                        </div>
                        <p>
                            版本：
                            <el-select v-model="versionSelect" @change="playbackVersionChange" size="large" style="width: 840px" :disabled="versionOptions.length <= 1">
                                <template #label="{ label }">
                                    {{ label.split("$|$")[0] }} <el-tag disable-transitions>{{ label.split("$|$")[1] }}</el-tag> <el-tag disable-transitions>{{ label.split("$|$")[2] }}</el-tag>
                                </template>
                                <el-option v-for="item in versionOptions" :key="item.value" :label="item.label" :value="item.value">
                                    {{ item.name }} <el-tag disable-transitions>{{ item.size }}</el-tag> <el-tag disable-transitions>{{ item.bitrate }}</el-tag>
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
                            <el-button plain type="success" :loading="play_loading" @click="playing(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks)">继续播放</el-button>
                            <el-button plain type="success" @click="playing(currentEpisodes.Id, 0)" :loading="play_loading">从头播放</el-button>
                        </template>
                        <template v-else><el-button plain type="success" @click="playing(currentEpisodes.Id, 0)" :loading="play_loading">播放</el-button></template>
                        <el-button plain @click="continuousPlay = !continuousPlay">
                            <span>{{ continuousPlay ? '连续播放' : '单集播放' }}</span>
                        </el-button>
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
                    </div>
                </div>
            </el-skeleton>
            <el-skeleton :loading="nextUpLoading" animated>
                <template #template>
                    <div class="note-item" v-for="i in 3" :key="i">
                        <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <h1 v-if="nextUpList && nextUpList.length == 1 && nextUpCurrentPage == 1">已经是最后一集了</h1>
                <h1 v-if="nextUpList && nextUpList.length > 1">接下来</h1>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row; justify-content: space-between;">
                    <template  v-for="(nextUpItem, index) in nextUpList">
                        <el-card style="width: 300px; margin-bottom: 7px;" v-if="index != 0 || nextUpCurrentPage != 1">
                            <el-link :underline="false" @click="gotoEpisodes(nextUpItem.Id)">
                                <p>{{ 'S' + nextUpItem.ParentIndexNumber + 'E' + nextUpItem.IndexNumber + '. ' + nextUpItem.Name }}</p>
                            </el-link>
                            <p>{{ nextUpItem.PremiereDate ? nextUpItem.PremiereDate.substring(0, 10) : '' }}
                                <el-tag disable-transitions>{{ nextUpItem.MediaSources ? formatBytes(maxMediaSources(nextUpItem.MediaSources)?.Size!) : 0 }}</el-tag>
                            </p>
                        </el-card>
                    </template>
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
import { nextTick, ref, watch } from 'vue';
import embyApi, { EmbyPageList, EpisodesItems, MediaSources, PlaybackInfo, UserData } from '../../api/embyApi';
import { formatBytes, formatMbps, secondsToHMS } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'
import invoke from '../../api/invoke';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import { usePlayback } from '../../store/playback';
import { EmbyServer, useEmbyServer } from '../../store/db/embyServer';
import { useProxyServer } from '../../store/db/proxyServer';

const router = useRouter()
const route = useRoute()

let embyServer = ref<EmbyServer>({})
useEmbyServer().getEmbyServer(<string>route.params.embyId).then(value => {
    embyServer.value = value!;
}).catch(e => ElMessage.error('获取Emby服务器失败' + e))

const versionOptions = ref<{label: string, value: string, name: string, size: string, bitrate: string}[]>([])
const videoOptions = ref<{label: string, value: number}[]>([])
const audioOptions = ref<{label: string, value: number}[]>([])
const subtitleOptions = ref<{label: string, value: number}[]>([])
const versionSelect = ref('')
const videoSelect = ref(-1)
const audioSelect = ref(-1)
const subtitleSelect = ref(-1)
const timeLength = ref('')

const continuousPlay = ref(true)
const playbackInfoLoading = ref(false)
const play_loading = ref(false)

const nextUpLoading = ref(false)
const nextUpList = ref<EpisodesItems[]>([])
const nextUpCurrentPage = ref(1)
const nextUpPageSize = ref(7)
const nextUpTotal = ref(0)

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
        if (!silent && json.MediaSources) {
            handleMediaSources(json.MediaSources)
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => playbackInfoLoading.value = false)
}
updateCurrentEpisodes()

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

function handleMediaSources(mediaSources: MediaSources[]) {
    if (!mediaSources || mediaSources.length == 0) {
        return
    }
    versionOptions.value = []
    let max = 0;
    let maxMediaSource = mediaSources[0]
    for (let mediaSource of mediaSources) {
        versionOptions.value.push({
            label: mediaSource.Name + '$|$' + formatBytes(mediaSource.Size) + '$|$' + formatMbps(mediaSource.Bitrate),
            value: mediaSource.Id,
            name: mediaSource.Name,
            size: formatBytes(mediaSource.Size),
            bitrate: formatMbps(mediaSource.Bitrate),
        })
        if (max < mediaSource.Size) {
            max = mediaSource.Size
            maxMediaSource = mediaSource
        }
    }
    playbackVersionChange(maxMediaSource.Id)
}

function playbackVersionChange(val: string) {
    versionSelect.value = val
    videoSelect.value = -1
    audioSelect.value = -1
    subtitleSelect.value = -1
    videoOptions.value = []
    audioOptions.value = []
    subtitleOptions.value = []
    let currentMediaSources = currentEpisodes.value!.MediaSources!.find(mediaSource => mediaSource.Id == versionSelect.value)
    if (!currentMediaSources) {
        return
    }
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
}

const playProxyUrl = ref<string | undefined>()
useProxyServer().getPlayProxyUrl(embyServer.value.play_proxy_id).then(response => {
    playProxyUrl.value = response
})

// const playingProgressTask = ref<NodeJS.Timeout>()
function playing(item_id: string, playbackPositionTicks: number) {
    play_loading.value = true
    return embyApi.playbackInfo(embyServer.value, item_id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let playbackInfo: PlaybackInfo = JSON.parse(response.body);
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
            return invoke.playback({
                path: directStreamUrl,
                proxy: playProxyUrl.value,
                title: episodesName + " | " + currentEpisodes.value?.SeriesName + " | " + embyServer.value.server_name,
                user_agent: embyServer.value!.user_agent!,
                server_id: embyServer.value!.id!,
                item_id: item_id,
                media_source_id: currentMediaSources.Id,
                play_session_id: playbackInfo.PlaySessionId,
                playback_position_ticks: playbackPositionTicks,
                vid: videoSelect.value,
                aid: audioSelect.value,
                sid: subtitleSelect.value,
                external_audio: externalAudio,
                external_subtitle: externalSubtitle,
            }).then(async () => {
                embyApi.playing(embyServer.value!, item_id, currentMediaSources.Id, playbackInfo.PlaySessionId, playbackPositionTicks).then(() => {
                    ElMessage.success({
                        message: '开始播放，请稍候'
                    })
                    // playingProgressTask.value = setInterval(() => {
                    //     embyApi.playingProgress(embyServer.value!, item_id, currentMediaSources.Id, playbackInfo.PlaySessionId, playbackPositionTicks)
                    // }, 30000)
                })
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

const playbackStore = usePlayback()
watch(() => playbackStore.playingStopped, (newValue, _oldValue) => {
    if (embyServer.value.id === newValue.server_id && newValue.item_id === currentEpisodes.value?.Id) {
        // if (playingProgressTask.value) {
        //     clearInterval(playingProgressTask.value)
        // }
        updateCurrentEpisodes(true).then(() => {
            if (currentEpisodes.value?.UserData?.Played && currentEpisodes.value.Type !== 'Movie' && continuousPlay.value) {
                // todo 播放完成后，展示窗口，跳到下一集，继承当前音频和字幕选项
                ElMessage.success({
                    message: '播放完成，即将播放下一集'
                })
                nextUpCurrentPage.value = 1
                nextUp(1).then(() => {
                    if (nextUpList.value.length > 0) {
                        router.replace({path: '/nav/emby/' + embyServer.value.id + '/episodes/' + nextUpList.value[0].Id, query: {autoplay: 'true'}})
                    } else {
                        ElMessage.warning({
                            message: '已经是最后一集了'
                        })
                    }
                })
            }
        })
    }
});

watch(() => route.params.episodeId, (_newId, _oldId) => {
    updateCurrentEpisodes().then(() => {
        if (route.query.autoplay === 'true') {
            nextTick(() => {
                playing(<string>route.params.episodeId, 0)
            })
        }
    })
})

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

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/series/' + seriesId)
}
</script>

<style scoped>
</style>