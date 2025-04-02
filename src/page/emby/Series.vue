<template>
    <el-scrollbar>
        <el-skeleton :loading="serieInfoLoading" animated>
            <template #template>
                <div style="display: flex; padding: 20px;">
                    <el-skeleton-item variant="image" style="height: 416px; width: 300px;" />
                    <div style="flex: 1;padding: 20px;">
                        <h1><el-skeleton-item variant="h1" style="width: 50%; margin-top: 10px;" /></h1>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                        <p>
                            <el-skeleton-item variant="button" style="width: 15%;margin: 5px;margin-left: 0;" />
                            <el-skeleton-item variant="button" style="width: 15%;margin: 5px;margin-left: 0;" />
                        </p>
                    </div>
                </div>
            </template>
            <div style="display: flex; padding: 20px;" v-if="currentSeries">
                <div style="min-height: 416px; min-width: 300px;">
                    <img v-lazy="images[<string>route.params.serieId]" style="max-height: 416px; max-width: 300px;" />
                </div>
                <div style="padding: 20px;">
                    <h1>{{ currentSeries.Name }}</h1>
                    <p>{{ currentSeries.ProductionYear }}</p>
                    <p><el-scrollbar style="height: 180px;">{{ currentSeries.Overview }}</el-scrollbar></p>
                    <el-button plain :disabled="playedLoading" @click="played()">
                        <el-icon color="#67C23A" :size="24" :class="playedLoading ? 'is-loading' : ''" v-if="currentSeries.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                        <el-icon :size="24" :class="playedLoading ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                        <span>已播放</span>
                    </el-button>
                    <el-button plain :disabled="starLoading" @click="star()">
                        <template v-if="currentSeries.UserData?.IsFavorite">
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
        <el-skeleton :loading="episodesLoading" animated>
            <template #template>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;padding: 20px;padding-top: 0;">
                    <el-card style="width: 300px; margin: 5px;" v-for="i in 5" :key="i">
                        <p><el-skeleton-item variant="text" style="width: 90%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 60%" /></p>
                    </el-card>
                </div>
            </template>
            <div v-if="episodesList && episodesList.length > 0" style="display: flex; flex-wrap: wrap; flex-direction: row; padding: 20px;">
                <ItemCard v-for="episodesItem in episodesList" :key="episodesItem.Id" :item="episodesItem" :embyServer="embyServer" />
                <el-pagination
                    v-model:current-page="episodesCurrentPage"
                    v-model:page-size="episodesPageSize"
                    layout="total, prev, pager, next, jumper"
                    :total="episodesTotal"
                    @current-change="handleEpisodesPageChange"
                    hide-on-single-page
                />
            </div>
        </el-skeleton>
        <el-skeleton :loading="seasonsLoading" animated>
            <template #template>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row; padding: 20px;">
                    <div v-for="i in 5" :key="i" style="display: flex; flex-direction: column; align-items: center; padding-right: 30px;">
                        <el-skeleton-item variant="image" style="height: 160px; width: 115px;" />
                        <p><el-skeleton-item variant="text" style="width: 60px" /></p>
                    </div>
                </div>
            </template>
            <div style="display: flex; flex-wrap: wrap; flex-direction: row; padding: 20px;" v-if="currentSeries && seasonsList && seasonsList.length > 0">
                <div v-for="season in seasonsList" @click="showSeasons(season)" style="display: flex; flex-direction: column; align-items: center; padding-right: 30px;">
                    <div style="min-height: 160px; min-width: 115px;">
                        <img v-lazy="images[season.Id]" style="max-height: 160px; max-width: 115px;" />
                    </div>
                    <span>{{ season.Name }}</span>
                </div>
            </div>
        </el-skeleton>
    </el-scrollbar>
    
    <el-dialog
        v-model="dialogSeasonsVisible"
        :title="dialogSeasons?.Name"
        width="800"
    >
        <el-scrollbar style="padding: 0 20px;">
            <el-skeleton :loading="dialogEpisodesLoading" animated>
                <template #template>
                    <div class="box-item" v-for="i in 5" :key="i">
                        <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div v-for="episodesItem in dialogEpisodesList" class="box-item">
                    <p><el-link :underline="false" @click="gotoEpisodes(episodesItem.Id)">{{ episodesItem.IndexNumber + '. ' + episodesItem.Name }}</el-link></p>
                    <p>
                        {{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }}
                        <el-tag disable-transitions>{{ mediaSourceSizeTag[episodesItem.Id] }}</el-tag>
                        <el-tag disable-transitions style="margin-left: 5px;">{{ mediaSourceBitrateTag[episodesItem.Id] }}</el-tag>
                        <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[episodesItem.Id] || 'Unknown' }}</el-tag>
                    </p>
                </div>
            </el-skeleton>
            <el-pagination
                v-model:current-page="dialogEpisodesCurrentPage"
                v-model:page-size="dialogEpisodesPageSize"
                layout="total, prev, pager, next, jumper"
                :total="dialogEpisodesTotal"
                @current-change="handleDialogEpisodesPageChange"
                hide-on-single-page
            />
        </el-scrollbar>
    </el-dialog>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from 'vue-router';
import { onUnmounted, ref } from 'vue';
import embyApi, { EmbyPageList, EpisodesItems, MediaSources, SeasonsItems, UserData } from '../../api/embyApi';
import ItemCard from '../../components/ItemCard.vue';
import { ElMessage } from 'element-plus';
import { formatBytes, formatMbps } from '../../util/str_util'
import { getResolutionFromMediaSources, maxMediaSources } from '../../util/play_info_util'
import invokeApi from '../../api/invokeApi';
import { EmbyServer, useEmbyServer } from '../../store/db/embyServer';
import { useProxyServer } from '../../store/db/proxyServer';
import { useEventBus } from '../../store/eventBus';

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

getEmbyServer().then(() => {
    updateCurrentSerie()
    getSeasons()
    getEpisodes()
    loadImage(<string>route.params.serieId)
})

const mediaSourceSizeTag = ref<{[key: string]: string}>({})
const mediaSourceBitrateTag = ref<{[key: string]: string}>({})
const mediaStreamResolutionTag = ref<{[key: string]: string}>({})
function getTag(itemId: string, mediaSources?: MediaSources[]) {
    let maxMediaSource = maxMediaSources(mediaSources);
    if (maxMediaSource) {
        mediaSourceSizeTag.value[itemId] = formatBytes(maxMediaSource.Size)
        mediaSourceBitrateTag.value[itemId] = formatMbps(maxMediaSource.Bitrate)
        if (maxMediaSource.MediaStreams && maxMediaSource.MediaStreams.length > 0) {
            mediaStreamResolutionTag.value[itemId] = getResolutionFromMediaSources(maxMediaSource)
        }
    }
}

const serieInfoLoading = ref(false)
const currentSeries = ref<EpisodesItems>()
function updateCurrentSerie() {
    serieInfoLoading.value = true
    return embyApi.items(embyServer.value, <string>route.params.serieId).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EpisodesItems = JSON.parse(response.body);
        currentSeries.value = json
    }).catch(e => {
        ElMessage.error('更新当前剧集信息失败' + e)
    }).finally(() => serieInfoLoading.value = false)
}

const starLoading = ref<boolean>(false)
function star() {
    if (!currentSeries.value?.UserData) {
        return
    }
    starLoading.value = true
    let fun;
    if (currentSeries.value?.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServer.value, currentSeries.value?.Id)
    } else {
        fun = embyApi.star(embyServer.value, currentSeries.value?.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        currentSeries.value!.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => {
        ElMessage.error('标记收藏信息失败' + e)
    }).finally(() => starLoading.value = false)
}

const playedLoading = ref<boolean>(false)
function played() {
    if (!currentSeries.value?.UserData) {
        return
    }
    playedLoading.value = true
    let fun;
    if (currentSeries.value?.UserData.Played) {
        fun = embyApi.unplayed(embyServer.value, currentSeries.value?.Id)
    } else {
        fun = embyApi.played(embyServer.value, currentSeries.value?.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        currentSeries.value!.UserData!.Played = json.Played
    }).catch(e => {
        ElMessage.error('标记播放信息失败' + e)
    }).finally(() => playedLoading.value = false)
}

const seasonsLoading = ref<boolean>(false)
const seasonsList = ref<SeasonsItems[]>([])
async function getSeasons() {
    seasonsLoading.value = true
    return embyApi.seasons(embyServer.value, <string>route.params.serieId).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<SeasonsItems> = JSON.parse(response.body);
        seasonsList.value = json.Items
        json.Items.forEach(item => {
            loadImage(item.Id)
        })
    }).catch(e => {
        ElMessage.error('获取季失败' + e)
    }).finally(() => seasonsLoading.value = false)
}

const episodesLoading = ref<boolean>(false)
const episodesList = ref<EpisodesItems[]>([])
const episodesCurrentPage = ref<number>(1)
const episodesPageSize = ref<number>(6)
const episodesTotal = ref<number>(0)
async function getEpisodes() {
    episodesLoading.value = true
    return embyApi.episodes(embyServer.value, <string>route.params.serieId, '', (episodesCurrentPage.value - 1) * episodesPageSize.value, episodesPageSize.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error('获取剧集失败' + e)
    }).finally(() => episodesLoading.value = false)
}
function handleEpisodesPageChange(page: number) {
    episodesCurrentPage.value = page
    getEpisodes()
}
function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/episodes/' + episodesId)
}

const dialogSeasonsVisible = ref<boolean>(false)
const dialogSeasons = ref<SeasonsItems>()
const dialogEpisodesLoading = ref<boolean>(false)
const dialogEpisodesList = ref<EpisodesItems[]>([])
const dialogEpisodesCurrentPage = ref<number>(1)
const dialogEpisodesPageSize = ref<number>(6)
const dialogEpisodesTotal = ref<number>(0)
function showSeasons(season: SeasonsItems) {
    dialogSeasonsVisible.value = true
    dialogSeasons.value = season
    dialogEpisodesCurrentPage.value = 1
    dialogEpisodesPageSize.value = 6
    dialogEpisodesTotal.value = 0
    dialogEpisodesList.value = []
    getDialogEpisodes()
}
function getDialogEpisodes() {
    dialogEpisodesLoading.value = true
    return embyApi.episodes(embyServer.value, currentSeries.value?.Id!, dialogSeasons.value?.Id!, (dialogEpisodesCurrentPage.value - 1) * dialogEpisodesPageSize.value, dialogEpisodesPageSize.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        dialogEpisodesList.value = json.Items
        dialogEpisodesTotal.value = json.TotalRecordCount
        for (const item of json.Items) {
            getTag(item.Id, item.MediaSources)
        }
    }).catch(e => ElMessage.error('获取剧集失败' + e)).finally(() => dialogEpisodesLoading.value = false)
}
function handleDialogEpisodesPageChange(page: number) {
    dialogEpisodesCurrentPage.value = page
    getDialogEpisodes()
}

const images = ref<{[key: string]: string}>({})
async function loadImage(itemId: string) {
  images.value[itemId] = invokeApi.loadImage({
    image_url: embyApi.getImageUrl(embyServer.value, itemId)!,
    proxy_url: await useProxyServer().getBrowseProxyUrl(embyServer.value.browse_proxy_id),
    user_agent: embyServer.value.user_agent!,
    cache_prefix: ['image', embyServer.value.id!],
  })
}
</script>

<style scoped>
.box-item {
    padding: 5px 20px;
}

.box-item:hover {
  background-color: #18222C;
}

.box-item.active {
  color: #409EFF;
}

</style>