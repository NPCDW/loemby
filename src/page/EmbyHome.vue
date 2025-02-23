<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
        <div>
            <h1>继续观看</h1>
            <el-skeleton :loading="episodesLoading" animated>
                <template #template>
                    <div class="note-item" v-for="i in 5" :key="i">
                        <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <el-card style="width: 300px; margin: 5px;" v-for="episodesItem in episodesList">
                        <h2>{{ episodesItem.SeriesName }}</h2>
                        <p>{{ 'S' + episodesItem.ParentIndexNumber + 'E' + episodesItem.IndexNumber + '. ' + episodesItem.Name }}</p>
                        <p><el-progress :percentage="episodesItem.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" /></p>
                        <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} 最大媒体流：{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</p>
                        <p><el-button type="primary" @click="playbackInfo(episodesItem.Id);nextUp(episodesItem.SeriesId)" :loading="search_loading">播放信息</el-button></p>
                    </el-card>
                </div>
            </el-skeleton>
            <el-pagination
                v-model:current-page="episodesCurrentPage"
                v-model:page-size="episodesPageSize"
                layout="total, prev, pager, next, jumper"
                :total="episodesTotal"
                @current-change="handleEpisodesPageChange"
                hide-on-single-page
            />
        </div>

        <div>
            <h1>接下来</h1>
            <el-skeleton :loading="nextUpLoading" animated>
                <template #template>
                    <div class="note-item" v-for="i in 5" :key="i">
                        <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <template  v-for="(nextUpItem, index) in nextUpList">
                        <div v-if="index == 0" style="width: 100%;padding: 10px;">
                            <h2>{{ nextUpItem.SeriesName }}</h2>
                            <p>{{ 'S' + nextUpItem.ParentIndexNumber + 'E' + nextUpItem.IndexNumber + '. ' + nextUpItem.Name }}</p>
                            <p><el-progress :percentage="nextUpItem.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" /></p>
                            <p>
                                <span>版本：
                                <el-select
                                v-model="versionSelect"
                                @change="playbackVersionChange"
                                placeholder="Select"
                                size="large"
                                style="width: 850px"
                                :disabled="versionOptions.length <= 1"
                                >
                                <el-option
                                    v-for="item in versionOptions"
                                    :key="item.value"
                                    :label="item.label"
                                    :value="item.value"
                                />
                                </el-select></span>
                            </p>
                            <p>
                                <span>视频：
                                <el-select
                                v-model="videoSelect"
                                placeholder="Select"
                                size="large"
                                style="width: 240px"
                                :disabled="videoOptions.length <= 1"
                                >
                                <el-option
                                    v-for="item in videoOptions"
                                    :key="item.value"
                                    :label="item.label"
                                    :value="item.value"
                                />
                                </el-select></span>
                                <span style="margin-left: 15px;">音频：
                                <el-select
                                v-model="audioSelect"
                                placeholder="Select"
                                size="large"
                                style="width: 240px"
                                :disabled="audioOptions.length <= 1"
                                >
                                <el-option
                                    v-for="item in audioOptions"
                                    :key="item.value"
                                    :label="item.label"
                                    :value="item.value"
                                />
                                </el-select></span>
                                <span style="margin-left: 15px;">字幕：
                                <el-select
                                v-model="subtitleSelect"
                                placeholder="Select"
                                size="large"
                                style="width: 240px"
                                :disabled="subtitleOptions.length <= 1"
                                >
                                <el-option
                                    v-for="item in subtitleOptions"
                                    :key="item.value"
                                    :label="item.label"
                                    :value="item.value"
                                />
                                </el-select></span>
                            </p>
                            <p><el-button type="primary" @click="playing(nextUpItem)" :loading="search_loading">播放</el-button></p>
                        </div>
                        <el-card style="width: 300px; margin: 5px;" v-if="index != 0">
                            <p>{{ 'S' + nextUpItem.ParentIndexNumber + 'E' + nextUpItem.IndexNumber + '. ' + nextUpItem.Name }}</p>
                            <p>{{ nextUpItem.PremiereDate ? nextUpItem.PremiereDate.substring(0, 10) : '' }} 最大媒体流：{{ nextUpItem.MediaSources ? formatBytes(maxMediaSources(nextUpItem.MediaSources)?.Size!) : 0 }}</p>
                            <p><el-button type="primary" @click="playbackInfo(nextUpItem.Id);nextUp(nextUpItem.SeriesId)" :loading="search_loading">播放信息</el-button></p>
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
import { ref } from 'vue';
import { watch } from 'vue'
import { useRoute } from 'vue-router'
import embyApi, { EmbyPageList, EpisodesItems, MediaSources, PlaybackInfo } from '../api/embyApi';
import { useConfig } from '../store/config';
import { ElMessage } from 'element-plus';
import { formatBytes, formatMbps } from '../util/str_util'
import { maxMediaSources } from '../util/play_info_util'
import invoke from '../api/invoke';

const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.id)!

watch(() => route.params.id, (newId, _oldId) => {
    embyServer = useConfig().getEmbyServer(<string>newId)!

    episodesList.value = []
    episodesCurrentPage.value = 1
    episodesPageSize.value = 6
    episodesTotal.value = 0
    continuePlay(episodesCurrentPage.value, episodesPageSize.value)

    nextUpList.value = []
    nextUpCurrentPage.value = 1
    nextUpPageSize.value = 4
    nextUpTotal.value = 0
})

const search_str = ref('')
const search_loading = ref(false)
const search = async () => {
    search_loading.value = true
    search_loading.value = false
}

const episodesLoading = ref(false)
const episodesList = ref<EpisodesItems[]>([])
const episodesCurrentPage = ref(1)
const episodesPageSize = ref(6)
const episodesTotal = ref(0)
const handleEpisodesPageChange = (val: number) => {
    episodesCurrentPage.value = val
    continuePlay(val, episodesPageSize.value)
}

function continuePlay(currentPage: number, pageSize: number) {
    episodesLoading.value = true
    episodesCurrentPage.value = currentPage
    episodesPageSize.value = pageSize
    return embyApi.continuePlay(embyServer, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = await response.json();
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => episodesLoading.value = false)
}
continuePlay(episodesCurrentPage.value, episodesPageSize.value)

const versionOptions = ref<{label: string, value: string}[]>([])
const videoOptions = ref<{label: string, value: number}[]>([])
const audioOptions = ref<{label: string, value: number}[]>([])
const subtitleOptions = ref<{label: string, value: number}[]>([])
const versionSelect = ref('')
const videoSelect = ref(-1)
const audioSelect = ref(-1)
const subtitleSelect = ref(-1)
const mediaSources = ref<MediaSources[]>([])
const playSessionId = ref('')

const nextUpLoading = ref(false)
const nextUpList = ref<EpisodesItems[]>([])
const nextUpCurrentPage = ref(1)
const nextUpPageSize = ref(4)
const nextUpTotal = ref(0)
const handleNextUpPageChange = (val: number) => {
    nextUpCurrentPage.value = val
    continuePlay(val, nextUpPageSize.value)
}

function nextUp(seriesId: string) {
    nextUpLoading.value = true
    return embyApi.nextUp(embyServer, seriesId, (nextUpCurrentPage.value - 1) * nextUpPageSize.value, nextUpPageSize.value).then(async response => {
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

function playbackInfo(itemId: string) {
    nextUpLoading.value = true
    return embyApi.playbackInfo(embyServer, itemId).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: PlaybackInfo = await response.json();
        mediaSources.value = json.MediaSources
        playSessionId.value = json.PlaySessionId
        handleMediaSources(json.MediaSources)
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
    let currentMediaSources = mediaSources.value.find(mediaSource => mediaSource.Id == versionSelect.value)
    if (currentMediaSources) {
        for (let mediaStream of currentMediaSources.MediaStreams) {
            if (mediaStream.Type == 'Video') {
                videoOptions.value.push({
                    label: mediaStream.DisplayTitle,
                    value: mediaStream.Index
                })
                videoSelect.value = mediaStream.Index
            } else if (mediaStream.Type == 'Audio') {
                audioOptions.value.push({
                    label: mediaStream.DisplayTitle,
                    value: mediaStream.Index
                })
            } else if (mediaStream.Type == 'Subtitle') {
                subtitleOptions.value.push({
                    label: mediaStream.DisplayTitle,
                    value: mediaStream.Index
                })
                if (mediaStream.Language === 'zh-CN') {
                    subtitleSelect.value = mediaStream.Index
                }
            }
        }
    }
    audioSelect.value = audioOptions.value.length > 1 ? audioOptions.value[1].value : audioOptions.value[0].value
}

function playing(item: EpisodesItems) {
    let currentMediaSources = mediaSources.value.find(mediaSource => mediaSource.Id == versionSelect.value)
    if (currentMediaSources) {
        let directStreamUrl = embyServer.base_url + currentMediaSources.DirectStreamUrl!
        if (!audioSelect.value) {
            directStreamUrl += '&AudioStreamIndex=' + audioSelect.value
        }
        if (!videoSelect.value) {
            directStreamUrl += '&VideoStreamIndex=' + videoSelect.value
        }
        if (!subtitleSelect.value) {
            directStreamUrl += '&SubtitleStreamIndexs=-1'
        }
        invoke.playback(directStreamUrl, embyServer!.id!, item.Id, currentMediaSources.Id, playSessionId.value).then(async () => {
            await embyApi.playing(embyServer!, item.Id, currentMediaSources.Id, playSessionId.value, item.UserData ? item.UserData.PlaybackPositionTicks : 0)
        }).catch(res => {
            ElMessage.error({
                message: res
            })
        })
    }
}
</script>

<style scoped>
.note-container {
  display: flex;
  height: 500px;
}

.note-sidebar {
  width: 30%;
  border-right: 1px solid #18222C;
  padding-right: 20px;
  overflow-y: auto;
}

.note-item {
  padding: 3px 10px;
  cursor: pointer;
  border-bottom: 1px solid #18222C;
}

.note-item:hover {
  background-color: #18222C;
}

.note-item.active {
  color: #409EFF;
}

.note-content {
  width: 70%;
  padding-left: 20px;
}

h2 {
  margin-top: 0;
}

.el-scrollbar {
  height: 100%;
}
</style>