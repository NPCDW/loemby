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
            <div style="display: flex;" v-if="currentSeries">
                <img v-if="embyServer" :src="embyServer.base_url + '/Items/' + route.params.serieId + '/Images/Primary'" />
                <div>
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
        <div v-if="episodesList && episodesList.length > 0">
            <el-card style="width: 300px; margin: 5px;" v-for="episodesItem in episodesList">
                <p><el-link @click="gotoEpisodes(episodesItem.Id)">{{ 'S' + episodesItem.ParentIndexNumber + 'E' + episodesItem.IndexNumber + '. ' + episodesItem.Name }}</el-link></p>
                <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }}</p>
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
            <div style="display: flex;" v-if="currentSeries && seasonsList && seasonsList.length > 0">
                <div v-for="season in seasonsList">
                    <img v-if="embyServer" :src="embyServer.base_url + '/Items/' + season.Id + '/Images/Primary'" />
                    <span>{{ season.Name }}</span>
                </div>
            </div>
        </el-skeleton>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from 'vue-router';
import { useConfig } from '../../store/config';
import { ref } from 'vue';
import embyApi, { EmbyPageList, EpisodesItems, SeasonsItems, UserData } from '../../api/embyApi';
import { ElMessage } from 'element-plus';

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.embyId)!

const serieInfoLoading = ref(false)
const currentSeries = ref<EpisodesItems>()
function updateCurrentEpisodes() {
    serieInfoLoading.value = true
    return embyApi.items(embyServer, <string>route.params.serieId).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EpisodesItems = await response.json();
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
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: UserData = await response.json();
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
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: UserData = await response.json();
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
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EmbyPageList<SeasonsItems> = await response.json();
        seasonsList.value = json.Items
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => seasonsLoading.value = false)
}

const episodesList = ref<EpisodesItems[]>([])
const episodesCurrentPage = ref<number>(1)
const episodesPageSize = ref<number>(10)
const episodesTotal = ref<number>(0)
async function getEpisodes() {
    return embyApi.episodes(embyServer, currentSeries.value?.Id!, '', episodesCurrentPage.value, episodesPageSize.value).then(async response => {
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
    })
}
function handleEpisodesPageChange(page: number) {
    episodesCurrentPage.value = page
    getEpisodes()
}
function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
</script>

<style scoped>
</style>