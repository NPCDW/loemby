<template>
    <el-scrollbar>
        <el-skeleton :loading="serieInfoLoading" animated>
            <template #template>
                <div class="note-item">
                    <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                </div>
            </template>
            <div style="display: flex; padding: 20px;" v-if="currentSeries">
                <img v-if="embyServer" :src="embyServer.base_url + '/Items/' + route.params.serieId + '/Images/Primary'" style="height: 416px; width: 300px;" />
                <div style="padding: 20px;">
                    <h1>{{ currentSeries.Name }}</h1>
                    <p>{{ currentSeries.ProductionYear }}</p>
                    <p>{{ currentSeries.Overview }}</p>
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
        <div v-if="episodesList && episodesList.length > 0" style="display: flex; flex-wrap: wrap; flex-direction: row; padding: 20px;">
            <el-card style="width: 300px; margin: 5px;" v-for="episodesItem in episodesList">
                <el-link :underline="false" @click="gotoEpisodes(episodesItem.Id)"><p>{{ 'S' + episodesItem.ParentIndexNumber + 'E' + episodesItem.IndexNumber + '. ' + episodesItem.Name }}</p></el-link>
                <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} <el-tag disable-transitions>{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</el-tag></p>
            </el-card>
            <el-pagination
                v-model:current-page="episodesCurrentPage"
                v-model:page-size="episodesPageSize"
                layout="total, prev, pager, next, jumper"
                :total="episodesTotal"
                @current-change="handleEpisodesPageChange"
                hide-on-single-page
            />
        </div>
        <el-skeleton :loading="seasonsLoading" animated>
            <template #template>
                <div class="note-item">
                    <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                </div>
            </template>
            <div style="display: flex; flex-wrap: wrap; flex-direction: row; padding: 20px;" v-if="currentSeries && seasonsList && seasonsList.length > 0">
                <div v-for="season in seasonsList" @click="showSeasons(season)" style="display: flex; flex-direction: column; align-items: center; padding-right: 30px;">
                    <img v-if="embyServer" :src="embyServer.base_url + '/Items/' + season.Id + '/Images/Primary'" style="height: 160px; width: 115px;" />
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
        <el-scrollbar>
            <h1>{{ dialogSeasons?.Name }}</h1>
            <p>{{ dialogSeasons?.Overview }}</p>
            <el-skeleton :loading="dialogEpisodesLoading" animated>
                <template #template>
                    <div class="note-item" v-for="i in 5" :key="i">
                        <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div v-for="episodesItem in dialogEpisodesList" class="note-item" style="display: flex;justify-content: space-between; align-items: center;">
                    <div>
                        <p>{{ episodesItem.IndexNumber + '. ' + episodesItem.Name }}</p>
                        <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} <el-tag disable-transitions>{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</el-tag></p>
                    </div>
                    <el-button @click="gotoEpisodes(episodesItem.Id)" type="success" plain circle><el-icon><i-ep-ArrowRightBold /></el-icon></el-button>
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
import { useConfig } from '../../store/config';
import { ref } from 'vue';
import embyApi, { EmbyPageList, EpisodesItems, SeasonsItems, UserData } from '../../api/embyApi';
import { ElMessage } from 'element-plus';
import { formatBytes } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.embyId)!

const serieInfoLoading = ref(false)
const currentSeries = ref<EpisodesItems>()
function updateCurrentEpisodes() {
    serieInfoLoading.value = true
    return embyApi.items(embyServer, <string>route.params.serieId).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EpisodesItems = JSON.parse(response.body);
        currentSeries.value = json
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => serieInfoLoading.value = false)
}
updateCurrentEpisodes().then(() => {
    getSeasons()
    getEpisodes()
})

const starLoading = ref<boolean>(false)
function star() {
    if (!currentSeries.value?.UserData) {
        return
    }
    starLoading.value = true
    let fun;
    if (currentSeries.value?.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServer, currentSeries.value?.Id)
    } else {
        fun = embyApi.star(embyServer, currentSeries.value?.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: UserData = JSON.parse(response.body);
        currentSeries.value!.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => {
        ElMessage.error({
            message: e
        })
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
        fun = embyApi.unplayed(embyServer, currentSeries.value?.Id)
    } else {
        fun = embyApi.played(embyServer, currentSeries.value?.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: UserData = JSON.parse(response.body);
        currentSeries.value!.UserData!.Played = json.Played
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => playedLoading.value = false)
}

const seasonsLoading = ref<boolean>(false)
const seasonsList = ref<SeasonsItems[]>([])
async function getSeasons() {
    seasonsLoading.value = true
    return embyApi.seasons(embyServer, currentSeries.value?.Id!).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<SeasonsItems> = JSON.parse(response.body);
        seasonsList.value = json.Items
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => seasonsLoading.value = false)
}

const episodesList = ref<EpisodesItems[]>([])
const episodesCurrentPage = ref<number>(1)
const episodesPageSize = ref<number>(6)
const episodesTotal = ref<number>(0)
async function getEpisodes() {
    return embyApi.episodes(embyServer, currentSeries.value?.Id!, '', (episodesCurrentPage.value - 1) * episodesPageSize.value, episodesPageSize.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
function handleEpisodesPageChange(page: number) {
    episodesCurrentPage.value = page
    getEpisodes()
}
function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
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
    return embyApi.episodes(embyServer, currentSeries.value?.Id!, dialogSeasons.value?.Id!, (dialogEpisodesCurrentPage.value - 1) * dialogEpisodesPageSize.value, dialogEpisodesPageSize.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        dialogEpisodesList.value = json.Items
        dialogEpisodesTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => dialogEpisodesLoading.value = false)
}
function handleDialogEpisodesPageChange(page: number) {
    dialogEpisodesCurrentPage.value = page
    getDialogEpisodes()
}
</script>

<style scoped>
</style>