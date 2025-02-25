<template>
    <el-scrollbar style="height: 100vh; padding: 0 20px;">
        <div>
            <el-skeleton :loading="playbackInfoLoading" animated>
                <template #template>
                    <div class="note-item" v-for="i in 3" :key="i">
                        <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div v-if="currentEpisodes" style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <div style="width: 100%;padding: 10px;">
                        <h2 v-if="currentEpisodes.Type === 'Movie'">{{ currentEpisodes.Name }}</h2>
                        <template v-else>
                            <h2>{{ currentEpisodes.SeriesName }}</h2>
                            <p>{{ 'S' + currentEpisodes.ParentIndexNumber + 'E' + currentEpisodes.IndexNumber + '. ' + currentEpisodes.Name }}</p>
                        </template>
                        <p><el-progress :percentage="currentEpisodes.UserData?.Played ? 100 : currentEpisodes.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" /></p>
                        <p>
                            版本：
                            <el-select v-model="versionSelect" @change="playbackVersionChange" size="large" style="width: 850px" :disabled="versionOptions.length <= 1">
                                <el-option v-for="item in versionOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select>
                        </p>
                        <p>
                            <span>视频：
                            <el-select v-model="videoSelect" size="large" style="width: 240px" :disabled="videoOptions.length <= 1">
                                <el-option v-for="item in videoOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select></span>
                            <span style="margin-left: 15px;">音频：
                            <el-select v-model="audioSelect" size="large" style="width: 240px" :disabled="audioOptions.length <= 1">
                                <el-option v-for="item in audioOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select></span>
                            <span style="margin-left: 15px;">字幕：
                            <el-select v-model="subtitleSelect" size="large" style="width: 240px" :disabled="subtitleOptions.length <= 1">
                                <el-option v-for="item in subtitleOptions" :key="item.value" :label="item.label" :value="item.value" />
                            </el-select></span>
                        </p>
                        <p v-if="currentEpisodes.UserData && currentEpisodes.UserData.PlaybackPositionTicks > 0">
                            <el-button type="primary" :loading="play_loading" @click="playing(currentEpisodes.Id, currentEpisodes.UserData.PlaybackPositionTicks)">继续播放</el-button>
                            <el-button type="primary" @click="playing(currentEpisodes.Id, 0)" :loading="play_loading">从头播放</el-button>
                        </p>
                        <p v-else><el-button type="primary" @click="playing(currentEpisodes.Id, 0)" :loading="play_loading">播放</el-button></p>
                        <p><el-button type="primary" :loading="play_loading">连播</el-button></p>
                        <p><el-button type="primary" :loading="play_loading">已播放</el-button></p>
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
                <h1>接下来</h1>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <template  v-for="(nextUpItem, index) in nextUpList">
                        <el-card style="width: 300px; margin: 5px;" v-if="index != 0">
                            <p>{{ 'S' + nextUpItem.ParentIndexNumber + 'E' + nextUpItem.IndexNumber + '. ' + nextUpItem.Name }}</p>
                            <p>{{ nextUpItem.PremiereDate ? nextUpItem.PremiereDate.substring(0, 10) : '' }}
                                 最大媒体流：{{ nextUpItem.MediaSources ? formatBytes(maxMediaSources(nextUpItem.MediaSources)?.Size!) : 0 }}</p>
                            <p><el-button type="primary" @click="gotoEpisodes(nextUpItem.Id)" :loading="playbackInfoLoading">详情</el-button></p>
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
import embyApi, { EmbyPageList, EpisodesItems, MediaSources, PlaybackInfo } from '../../api/embyApi';
import { formatBytes, formatMbps } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'
import invoke from '../../api/invoke';
import { useConfig } from '../../store/config';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import { usePlayback } from '../../store/playback';

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.embyId)!

const versionOptions = ref<{label: string, value: string}[]>([])
const videoOptions = ref<{label: string, value: number}[]>([])
const audioOptions = ref<{label: string, value: number}[]>([])
const subtitleOptions = ref<{label: string, value: number}[]>([])
const versionSelect = ref('')
const videoSelect = ref(-1)
const audioSelect = ref(-1)
const subtitleSelect = ref(-1)

const playbackInfoLoading = ref(false)
const play_loading = ref(false)

const nextUpLoading = ref(false)
const nextUpList = ref<EpisodesItems[]>([])
const nextUpCurrentPage = ref(1)
const nextUpPageSize = ref(4)
const nextUpTotal = ref(0)

const currentEpisodes = ref<EpisodesItems>()
function updateCurrentEpisodes(silent: boolean = false) {
    if (!silent) {
        playbackInfoLoading.value = true
    }
    return embyApi.items(embyServer, <string>route.params.episodeId).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EpisodesItems = await response.json();
        currentEpisodes.value = json
        if (!silent && json.MediaSources) {
            handleMediaSources(json.MediaSources)
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => playbackInfoLoading.value = false)
}
updateCurrentEpisodes()

const handleNextUpPageChange = (val: number) => {
    nextUpCurrentPage.value = val
    nextUp(val)
}

function nextUp(pageNumber: number) {
    nextUpLoading.value = true
    return embyApi.nextUp(embyServer, currentEpisodes.value?.SeriesId!, (pageNumber - 1) * nextUpPageSize.value, nextUpPageSize.value).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = await response.json();
        nextUpList.value = json.Items
        nextUpTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
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
            label: mediaSource.Name + '    大小: ' + formatBytes(mediaSource.Size) + '     码率: ' + formatMbps(mediaSource.Bitrate) + '',
            value: mediaSource.Id
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
    audioOptions.value.push({
        label: '关闭',
        value: -1
    })
    subtitleOptions.value.push({
        label: '关闭',
        value: -1
    })
    let currentMediaSources = currentEpisodes.value!.MediaSources!.find(mediaSource => mediaSource.Id == versionSelect.value)
    if (currentMediaSources) {
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
                videoSelect.value = videoIndex
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
        if (audioSelect.value === -1 && audioOptions.value.length > 1) {
            audioSelect.value = audioOptions.value[1].value
        }
        if (subtitleSelect.value === -1 && subtitleOptions.value.length > 1) {
            subtitleSelect.value = subtitleOptions.value[1].value
        }
    }
}

function playing(item_id: string, playbackPositionTicks: number) {
    play_loading.value = true
    return embyApi.playbackInfo(embyServer, item_id).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let playbackInfo: PlaybackInfo = await response.json();
        let currentMediaSources = playbackInfo.MediaSources!.find(mediaSource => mediaSource.Id == versionSelect.value)
        if (currentMediaSources) {
            let directStreamUrl = embyServer.base_url + currentMediaSources.DirectStreamUrl!
            let externalAudio = []
            let externalSubtitle = []
            for (let mediaStream of currentMediaSources.MediaStreams) {
                if (mediaStream.Type == 'Audio' && mediaStream.IsExternal) {
                    externalAudio.push(embyApi.getAudioStreamUrl(embyServer, currentMediaSources, mediaStream)!)
                } else if (mediaStream.Type == 'Subtitle' && mediaStream.IsExternal) {
                    externalSubtitle.push(embyApi.getSubtitleStreamUrl(embyServer, currentMediaSources, mediaStream)!)
                }
            }
            return invoke.playback({
                path: directStreamUrl,
                serverId: embyServer!.id!,
                itemId: item_id,
                mediaSourceId: currentMediaSources.Id,
                playSessionId: playbackInfo.PlaySessionId,
                playbackPositionTicks: playbackPositionTicks,
                aid: audioSelect.value,
                sid: subtitleSelect.value,
                externalAudio: externalAudio,
                externalSubtitle: externalSubtitle,
            }).then(async () => {
                embyApi.playing(embyServer!, item_id, currentMediaSources.Id, playbackInfo.PlaySessionId, playbackPositionTicks).then(() => {
                    ElMessage.success({
                        message: '开始播放，请稍候'
                    })
                })
            }).catch(res => {
                ElMessage.error({
                    message: res
                })
            })
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => play_loading.value = false)
}

const playbackStore = usePlayback()
watch(() => playbackStore.playingStopped, (newValue, _oldValue) => {
    console.log('listen store playingStopped', playbackStore.playingStopped, newValue, _oldValue);
    if (embyServer.id === newValue.server_id && newValue.item_id === currentEpisodes.value?.Id) {
        updateCurrentEpisodes(true).then(() => {
            if (currentEpisodes.value?.UserData?.Played && currentEpisodes.value.Type !== 'Movie') {
                ElMessage.success({
                    message: '播放完成，即将播放下一集'
                })
                nextUp(nextUpCurrentPage.value).then(() => {
                    if (nextUpList.value.length > 0) {
                        router.replace({path: '/nav/emby/' + embyServer.id + '/episodes/' + nextUpList.value[0].Id, query: {autoplay: 'true'}})
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

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
</script>

<style scoped>
</style>