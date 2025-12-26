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
                <div style="min-height: 416px; min-width: 300px;" class="loe-cover-img">
                    <img v-lazy="useImage().images[embyServerId + ':cover:' + currentSeries.Id]" style="max-height: 416px; max-width: 300px;" />
                </div>
                <div style="padding: 20px;">
                    <h1>{{ currentSeries.Name }}</h1>
                    <p>{{ currentSeries.ProductionYear }}</p>
                    <p><el-scrollbar style="height: 180px;">{{ currentSeries.Overview }}</el-scrollbar></p>
                    <p>
                        <span>外部链接：</span>
                        <el-tooltip v-for="externalUrl in currentSeries.ExternalUrls" :content="externalUrl.Url" placement="bottom" effect="light">
                            <el-button round @click="invokeApi.open_url(externalUrl.Url)"><i-ep-Link /> {{ externalUrl.Name }}</el-button>
                        </el-tooltip>
                    </p>
                    <el-button plain :disabled="playedLoading[currentSeries.Id]" @click="played(currentSeries)">
                        <el-icon color="#67C23A" :size="20" :class="playedLoading[currentSeries.Id] ? 'is-loading' : ''" v-if="currentSeries.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                        <el-icon :size="20" :class="playedLoading[currentSeries.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                        <span>已播放</span>
                    </el-button>
                    <el-button plain :disabled="starLoading[currentSeries.Id]" @click="star(currentSeries)">
                        <template v-if="currentSeries.UserData?.IsFavorite">
                            <el-icon color="#E6A23C" :size="20" :class="starLoading[currentSeries.Id] ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                            <span>取消收藏</span>
                        </template>
                        <template v-else>
                            <el-icon :size="20" :class="starLoading[currentSeries.Id] ? 'is-loading' : ''"><i-ep-Star /></el-icon>
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
                <ItemCard v-for="episodeItem in episodesList" :key="episodeItem.Id" :item="episodeItem" :embyServerId="embyServerId" />
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
                    <div style="min-height: 160px; min-width: 115px;" class="loe-cover-img">
                        <img v-lazy="useImage().images[embyServerId + ':cover:' + season.Id]" style="max-height: 160px; max-width: 115px; cursor: pointer;" />
                    </div>
                    <el-text truncated style="max-width: 115px;">{{ season.Name }}</el-text>
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
            <p>简介：{{ dialogSeasons?.Overview }}</p>
            <p>
                <el-button plain :disabled="playedLoading[dialogSeasons!.Id]" @click="played(dialogSeasons!)">
                    <el-icon color="#67C23A" :size="20" :class="playedLoading[dialogSeasons!.Id] ? 'is-loading' : ''" v-if="dialogSeasons?.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                    <el-icon :size="20" :class="playedLoading[dialogSeasons!.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                    <span>已播放</span>
                </el-button>
                <el-button plain :disabled="starLoading[dialogSeasons!.Id]" @click="star(dialogSeasons!)">
                    <template v-if="dialogSeasons?.UserData?.IsFavorite">
                        <el-icon color="#E6A23C" :size="20" :class="starLoading[dialogSeasons!.Id] ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                        <span>取消收藏</span>
                    </template>
                    <template v-else>
                        <el-icon :size="20" :class="starLoading[dialogSeasons!.Id] ? 'is-loading' : ''"><i-ep-Star /></el-icon>
                        <span>收藏</span>
                    </template>
                </el-button>
            </p>
            <el-skeleton :loading="dialogEpisodesLoading" animated>
                <template #template>
                    <div class="box-item" v-for="i in 5" :key="i">
                        <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div v-for="episodeItem in dialogEpisodesList" class="box-item">
                    <p>
                        <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)" style="display: block;">
                            {{ episodeItem.IndexNumber + '. ' + episodeItem.Name }}
                        </el-link>
                    </p>
                    <div style="display: flex;justify-content: space-between;align-items: end;">
                        <span style="display: flex; flex-direction: column;">
                            <el-tag disable-transitions style="margin-left: 10px;" v-for="value in mediaSourceTag[episodeItem.Id]">{{ value }}</el-tag>
                        </span>
                        <span style="display: flex; justify-content: center; align-items: center;">
                            <span>{{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}</span>
                            <el-link style="margin-left: 7px;" :underline="false" v-if="episodeItem.UserData" :disabled="starLoading[episodeItem.Id]" @click="star(episodeItem)">
                                <el-icon color="#E6A23C" :size="24" :class="starLoading[episodeItem.Id] ? 'is-loading' : ''" v-if="episodeItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                <el-icon :size="24" :class="starLoading[episodeItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                            </el-link>
                            <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[episodeItem.Id]" v-if="episodeItem.UserData" @click="played(episodeItem)">
                                <el-icon color="#67C23A" :size="24" :class="playedLoading[episodeItem.Id] ? 'is-loading' : ''" v-if="episodeItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                <el-icon :size="24" :class="playedLoading[episodeItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                            </el-link>
                        </span>
                    </div>
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
import { ref } from 'vue';
import embyApi, { EmbyPageList, EpisodeItem, MediaSource, SeasonItem, SeriesItem, UserData } from '../../api/embyApi';
import ItemCard from '../../components/ItemCard.vue';
import { ElMessage } from 'element-plus';
import { formatBytes } from '../../util/str_util'
import { getResolutionFromMediaSources } from '../../util/play_info_util'
import invokeApi from '../../api/invokeApi';
import { useImage } from '../../store/image';

const router = useRouter()
const route = useRoute()

const embyServerId = <string>route.params.embyId

const mediaSourceTag = ref<{[key: string]: string[]}>({})
function getTag(itemId: string, mediaSources?: MediaSource[]) {
    mediaSourceTag.value[itemId] = []
    if (mediaSources) {
        for (let mediaSource of mediaSources) {
            const size = formatBytes(mediaSource.Size) || '0 KB'
            // const bitrate = formatMbps(mediaSource.Bitrate) || '0 Kbps'
            let resolution = getResolutionFromMediaSources(mediaSource)
            mediaSourceTag.value[itemId].push(size + " | " + resolution)
        }
    }
}

const serieInfoLoading = ref(false)
const currentSeries = ref<SeriesItem>()
function updateCurrentSerie() {
    serieInfoLoading.value = true
    return embyApi.items(embyServerId, <string>route.params.serieId).then(async response => {
        let json: SeriesItem = JSON.parse(response);
        currentSeries.value = json
        useImage().loadCover(embyServerId, json)
    }).catch(e => ElMessage.error('更新当前剧集信息失败' + e)).finally(() => serieInfoLoading.value = false)
}
updateCurrentSerie()

const starLoading = ref<{[key: string]: boolean}>({})
function star(item: SeriesItem | SeasonItem | EpisodeItem) {
    if (!item.UserData) {
        return
    }
    starLoading.value[item.Id] = true
    let fun;
    if (item.UserData!.IsFavorite) {
        fun = embyApi.unstar(embyServerId, item.Id)
    } else {
        fun = embyApi.star(embyServerId, item.Id)
    }
    return fun.then(async response => {
        let json: UserData = JSON.parse(response);
        item.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => ElMessage.error('标记收藏信息失败' + e)).finally(() => starLoading.value[item.Id] = false)
}

const playedLoading = ref<{[key: string]: boolean}>({})
function played(item: SeriesItem | SeasonItem | EpisodeItem) {
    if (!currentSeries.value?.UserData) {
        return
    }
    playedLoading.value[item.Id] = true
    let fun;
    if (item.UserData!.Played) {
        fun = embyApi.unplayed(embyServerId, item.Id)
    } else {
        fun = embyApi.played(embyServerId, item.Id)
    }
    return fun.then(async response => {
        let json: UserData = JSON.parse(response);
        item.UserData!.Played = json.Played
    }).catch(e => ElMessage.error('标记播放信息失败' + e)).finally(() => playedLoading.value[item.Id] = false)
}

const seasonsLoading = ref<boolean>(false)
const seasonsList = ref<SeasonItem[]>([])
async function getSeasons() {
    seasonsLoading.value = true
    return embyApi.seasons(embyServerId, <string>route.params.serieId).then(async response => {
        let json: EmbyPageList<SeasonItem> = JSON.parse(response);
        seasonsList.value = json.Items
        json.Items.forEach(item => {
            useImage().loadCover(embyServerId, item)
        })
    }).catch(e => ElMessage.error('获取季失败' + e)).finally(() => seasonsLoading.value = false)
}
getSeasons()

const episodesLoading = ref<boolean>(false)
const episodesList = ref<EpisodeItem[]>([])
const episodesCurrentPage = ref<number>(1)
const episodesPageSize = ref<number>(6)
const episodesTotal = ref<number>(0)
async function getEpisodes() {
    episodesLoading.value = true
    return embyApi.episodes(embyServerId, <string>route.params.serieId, '', (episodesCurrentPage.value - 1) * episodesPageSize.value, episodesPageSize.value).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => ElMessage.error('获取剧集失败' + e)).finally(() => episodesLoading.value = false)
}
getEpisodes()

function handleEpisodesPageChange(page: number) {
    episodesCurrentPage.value = page
    getEpisodes()
}
function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
}

const dialogSeasonsVisible = ref<boolean>(false)
const dialogSeasons = ref<SeasonItem>()
const dialogEpisodesLoading = ref<boolean>(false)
const dialogEpisodesList = ref<EpisodeItem[]>([])
const dialogEpisodesCurrentPage = ref<number>(1)
const dialogEpisodesPageSize = ref<number>(6)
const dialogEpisodesTotal = ref<number>(0)
function showSeasons(season: SeasonItem) {
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
    return embyApi.episodes(embyServerId, currentSeries.value?.Id!, dialogSeasons.value?.Id!, (dialogEpisodesCurrentPage.value - 1) * dialogEpisodesPageSize.value, dialogEpisodesPageSize.value).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
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