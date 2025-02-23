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
                        <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div v-for="episodesItem in episodesList" class="note-item">
                    <p>{{ 'S' + episodesItem.ParentIndexNumber + 'E' + episodesItem.IndexNumber + '. ' + episodesItem.Name }}</p>
                    <p>播放进度：<el-progress :percentage="episodesItem.UserData?.PlaybackPositionTicks" status="success" /></p>
                    <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} 最大媒体流：{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</p>
                    <p><el-button type="primary" @click="playbackInfo(episodesItem.Id)" :loading="search_loading">播放信息</el-button></p>
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
                        <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <template v-for="(nextUpItem, index) in nextUpList">
                    <div v-if="index == 0" class="note-item">
                        <p>{{ 'S' + nextUpItem.ParentIndexNumber + 'E' + nextUpItem.IndexNumber + '. ' + nextUpItem.Name }}</p>
                        <p>播放进度：<el-progress :percentage="nextUpItem.UserData?.PlaybackPositionTicks" status="success" /></p>
                        <p>视频：
                            <el-select
                            v-model="videoSelect"
                            @change="playbackVedioChange"
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
                            </el-select>
                        </p>
                        <p>音频：
                            <el-select
                            v-model="audioSelect"
                            placeholder="Select"
                            size="large"
                            style="width: 240px"
                            :disabled="videoOptions.length <= 1"
                            >
                            <el-option
                                v-for="item in audioOptions"
                                :key="item.value"
                                :label="item.label"
                                :value="item.value"
                            />
                            </el-select>
                        </p>
                        <p>字幕：
                            <el-select
                            v-model="subtitleSelect"
                            placeholder="Select"
                            size="large"
                            style="width: 240px"
                            :disabled="videoOptions.length <= 1"
                            >
                            <el-option
                                v-for="item in subtitleOptions"
                                :key="item.value"
                                :label="item.label"
                                :value="item.value"
                            />
                            </el-select>
                        </p>
                        <p><el-button type="primary" @click="playing(nextUpItem)" :loading="search_loading">播放</el-button></p>
                    </div>
                    <div v-else class="note-item">
                        <p>{{ 'S' + nextUpItem.ParentIndexNumber + 'E' + nextUpItem.IndexNumber + '. ' + nextUpItem.Name }}</p>
                        <p>播放进度：<el-progress :percentage="nextUpItem.UserData?.PlaybackPositionTicks" status="success" /></p>
                        <p>{{ nextUpItem.PremiereDate ? nextUpItem.PremiereDate.substring(0, 10) : '' }} 最大媒体流：{{ nextUpItem.MediaSources ? formatBytes(maxMediaSources(nextUpItem.MediaSources)?.Size!) : 0 }}</p>
                        <p><el-button type="primary" @click="playbackInfo(nextUpItem.Id)" :loading="search_loading">播放信息</el-button></p>
                    </div>
                </template>
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
import { formatBytes } from '../util/str_util'
import { maxMediaSources } from '../util/play_info_util'
import invoke from '../api/invoke';

const route = useRoute()

watch(() => route.params.id, (newId, oldId) => {
  // 对路由变化做出响应...
})

const embyServer = useConfig().getEmbyServer(<string>route.params.id)!

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

const videoOptions = ref<{label: string, value: string}[]>([])
const audioOptions = ref<{label: string, value: number}[]>([])
const subtitleOptions = ref<{label: string, value: number}[]>([])
const videoSelect = ref('')
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
    let max = 0;
    let maxMediaSource = mediaSources[0]
    for (let mediaSource of mediaSources) {
        for (let mediaStream of mediaSource.MediaStreams) {
            if (mediaStream.Type == 'Video') {
                videoOptions.value.push({
                    label: mediaStream.DisplayTitle,
                    value: mediaSource.Id
                })
            }
        }
        if (max < mediaSource.Size) {
            max = mediaSource.Size
            maxMediaSource = mediaSource
        }
    }
    playbackVedioChange(maxMediaSource.Id)
}

function playbackVedioChange(val: string) {
    videoSelect.value = val
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
    let currentMediaSources = mediaSources.value.find(mediaSource => mediaSource.Id == videoSelect.value)
    if (currentMediaSources) {
        for (let mediaStream of currentMediaSources.MediaStreams) {
            if (mediaStream.Type == 'Audio') {
                audioOptions.value.push({
                    label: mediaStream.DisplayTitle,
                    value: mediaStream.Index
                })
            } else if (mediaStream.Type == 'Subtitle') {
                subtitleOptions.value.push({
                    label: mediaStream.DisplayTitle,
                    value: mediaStream.Index
                })
            }
        }
    }
    audioSelect.value = audioOptions.value.length > 1 ? audioOptions.value[1].value : audioOptions.value[0].value
    subtitleSelect.value = subtitleOptions.value.length > 1 ? subtitleOptions.value[1].value : subtitleOptions.value[0].value
}

function playing(item: EpisodesItems) {
    let currentMediaSources = mediaSources.value.find(mediaSource => mediaSource.Id == videoSelect.value)
    if (currentMediaSources) {
        let directStreamUrl = embyServer.base_url + currentMediaSources.DirectStreamUrl!
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
</style>