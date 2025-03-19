<template>
    <el-card style="width: 300px; margin: 5px;">
        <template #header>
            <el-link v-if="item.Type == 'Series'" :underline="false" @click="gotoSeries(item.Id)">{{ item.Name }}</el-link>
            <el-link v-else :underline="false" @click="gotoEpisodes(item.Id)">{{ item.Name }}</el-link>
        </template>
        <div style="margin-bottom: 10px;">
            <span v-if="item.Type == 'Series'">
                {{ item.ProductionYear + (item.EndDate && item.EndDate.substring(0, 4) != item.ProductionYear + '' ? '-' + item.EndDate.substring(0, 4) : '') }}
            </span>
            <span v-else style="display: flex;justify-content: space-between;align-items: center;">
                <span>{{ item.ProductionYear }}</span>
                <el-tag disable-transitions>{{ mediaSourceSizeTag[item.Id] }}</el-tag>
                <el-tag disable-transitions style="margin-left: 5px;">{{ mediaSourceBitrateTag[item.Id] }}</el-tag>
                <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[item.Id] }}</el-tag>
            </span>
        </div>
        <div style="display: flex;justify-content: space-between;">
            <span>
                <el-link :underline="false" v-if="item.UserData" :disabled="starLoading[item.Id]" @click="star(item)">
                    <el-icon color="#E6A23C" :size="24" :class="starLoading[item.Id] ? 'is-loading' : ''" v-if="item.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                    <el-icon :size="24" :class="starLoading[item.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                </el-link>
                <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[item.Id]" v-if="item.UserData" @click="played(item)">
                    <el-icon color="#67C23A" :size="24" :class="playedLoading[item.Id] ? 'is-loading' : ''" v-if="item.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                    <el-icon :size="24" :class="playedLoading[item.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                </el-link>
            </span>
            <span>
                <el-badge :value="item.UserData?.UnplayedItemCount" :max="999" :show-zero="false" type="primary">
                    <el-button v-if="item.Type == 'Series'" @click="getSeasons(item)" type="primary" plain>剧集</el-button>
                </el-badge>
            </span>
        </div>
    </el-card>
    
    <el-dialog
        v-model="dialogSeriesVisible"
        :title="dialogSeries?.Name"
        width="800"
    >
        <div class="note-container">
            <div class="note-sidebar">
                <el-scrollbar>
                    <el-skeleton :loading="dialogSeasonsLoading" animated>
                        <template #template>
                            <div class="note-item" v-for="i in 5" :key="i">
                                <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                                <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                            </div>
                        </template>
                        <div
                            v-for="seasonsItem in dialogSeasonsList"
                            :key="seasonsItem.Id"
                            class="note-item"
                            :class="{ active: dialogSeasons?.Id === seasonsItem.Id }"
                            @click="getEpisodes(dialogEmbyServer!, dialogSeries!.Id, seasonsItem, 1, 10)"
                        >
                            <h3>{{ 'S' + seasonsItem.IndexNumber + '. ' + seasonsItem.Name }}</h3>
                            <div style="display: flex;justify-content: space-between;">
                                <span>
                                    <span style="margin-right: 10px;">{{ seasonsItem.ProductionYear }}</span>
                                    <el-tag type="primary" effect="dark" round disable-transitions>{{ seasonsItem.UserData?.UnplayedItemCount }}</el-tag>
                                </span>
                                <span>
                                    <el-link :underline="false" v-if="seasonsItem.UserData" :disabled="starLoading[seasonsItem.Id]" @click="star(seasonsItem)">
                                        <el-icon color="#E6A23C" :size="24" :class="starLoading[seasonsItem.Id] ? 'is-loading' : ''" v-if="seasonsItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                        <el-icon :size="24" :class="starLoading[seasonsItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                                    </el-link>
                                    <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[seasonsItem.Id]" v-if="seasonsItem.UserData" @click="played(seasonsItem)">
                                        <el-icon color="#67C23A" :size="24" :class="playedLoading[seasonsItem.Id] ? 'is-loading' : ''" v-if="seasonsItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                        <el-icon :size="24" :class="playedLoading[seasonsItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                    </el-link>
                                </span>
                            </div>
                        </div>
                    </el-skeleton>
                </el-scrollbar>
            </div>
            <div class="note-content">
                <el-scrollbar>
                    <el-skeleton :loading="dialogEpisodesLoading" animated>
                        <template #template>
                            <div class="note-item" v-for="i in 5" :key="i">
                                <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                                <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                            </div>
                        </template>
                        <div v-for="episodesItem in dialogEpisodesList" class="note-item">
                            <p>
                                <el-link :underline="false" @click="gotoEpisodes(episodesItem.Id)">
                                    {{ episodesItem.IndexNumber + '. ' + episodesItem.Name }}
                                </el-link>
                            </p>
                            <div style="display: flex;justify-content: space-between;">
                                <span>
                                    <span>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }}</span>
                                    <el-tag disable-transitions style="margin-left: 10px;">{{ mediaSourceSizeTag[episodesItem.Id] }}</el-tag>
                                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaSourceBitrateTag[episodesItem.Id] }}</el-tag>
                                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[episodesItem.Id] }}</el-tag>
                                </span>
                                <span>
                                    <el-link :underline="false" v-if="episodesItem.UserData" :disabled="starLoading[episodesItem.Id]" @click="star(episodesItem)">
                                        <el-icon color="#E6A23C" :size="24" :class="starLoading[episodesItem.Id] ? 'is-loading' : ''" v-if="episodesItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                        <el-icon :size="24" :class="starLoading[episodesItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                                    </el-link>
                                    <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[episodesItem.Id]" v-if="episodesItem.UserData" @click="played(episodesItem)">
                                        <el-icon color="#67C23A" :size="24" :class="playedLoading[episodesItem.Id] ? 'is-loading' : ''" v-if="episodesItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                        <el-icon :size="24" :class="playedLoading[episodesItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                    </el-link>
                                </span>
                            </div>
                        </div>
                    </el-skeleton>
                    <el-pagination
                        v-if="episodes_result[dialogSeries!.Id + '|' + dialogSeasons?.Id]"
                        v-model:current-page="dialogEpisodesCurrentPage"
                        v-model:page-size="dialogEpisodesPageSize"
                        layout="total, prev, pager, next, jumper"
                        :total="episodes_result[dialogSeries!.Id + '|' + dialogSeasons!.Id].total"
                        @current-change="handleEpisodesPageChange(dialogEpisodesCurrentPage, dialogEmbyServer!, dialogSeries!.Id, dialogSeasons!)"
                        hide-on-single-page
                    />
                </el-scrollbar>
            </div>
        </div>
    </el-dialog>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import embyApi, { EmbyPageList, EpisodesItems, MediaSources, SearchItems, SeasonsItems, UserData } from '../api/embyApi';
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus';
import { formatBytes, formatMbps } from '../util/str_util'
import { guessResolution, maxMediaSources } from '../util/play_info_util'
import { EmbyServer } from '../store/db/embyServer';

const router = useRouter()

const {item, embyServer} = defineProps<{
  item: SearchItems,
  embyServer: EmbyServer
}>()

const mediaSourceSizeTag = ref<{[key: string]: string}>({})
const mediaSourceBitrateTag = ref<{[key: string]: string}>({})
const mediaStreamResolutionTag = ref<{[key: string]: string}>({})
function getTag(itemId: string, mediaSources?: MediaSources[]) {
    let maxMediaSource = maxMediaSources(mediaSources);
    if (maxMediaSource) {
        mediaSourceSizeTag.value[itemId] = formatBytes(maxMediaSource.Size)
        mediaSourceBitrateTag.value[itemId] = formatMbps(maxMediaSource.Bitrate)
        if (maxMediaSource.MediaStreams && maxMediaSource.MediaStreams.length > 0) {
            mediaStreamResolutionTag.value[itemId] = guessResolution(maxMediaSource.MediaStreams[0].Width, maxMediaSource.MediaStreams[0].Height)
        }
    }
}
getTag(item.Id, item.MediaSources)

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/series/' + seriesId)
}

const starLoading = ref<{[key: string]: boolean}>({})
function star(item: SearchItems | SeasonsItems | EpisodesItems) {
    if (!item.UserData) {
        return
    }
    starLoading.value[item.Id] = true
    let fun;
    if (item.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServer, item.Id)
    } else {
        fun = embyApi.star(embyServer, item.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        item.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => starLoading.value[item.Id] = false)
}

const playedLoading = ref<{[key: string]: boolean}>({})
function played(item: SearchItems | SeasonsItems | EpisodesItems) {
    if (!item.UserData) {
        return
    }
    playedLoading.value[item.Id] = true
    let fun;
    if (item.UserData.Played) {
        fun = embyApi.unplayed(embyServer, item.Id)
    } else {
        fun = embyApi.played(embyServer, item.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        item.UserData!.Played = json.Played
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => playedLoading.value[item.Id] = false)
}

const seasons_result = ref<{[key: string]: EmbyPageList<SeasonsItems>}>({})
const episodes_result = ref<{[key: string]: {total: number, [key: number]: EpisodesItems[]}}>({})

const dialogSeriesVisible = ref(false)
const dialogEmbyServer = ref<EmbyServer>()
const dialogSeries = ref<SearchItems>()
const dialogSeasons = ref<SeasonsItems>()
const dialogSeasonsList = ref<SeasonsItems[]>([])
const dialogEpisodesList = ref<EpisodesItems[]>([])
const dialogEpisodesCurrentPage = ref(1)
const dialogEpisodesPageSize = ref(10)
const dialogSeasonsLoading = ref(false)
const dialogEpisodesLoading = ref(false)

async function getSeasons(series: SearchItems) {
    dialogSeasons.value = undefined
    dialogSeasonsLoading.value = true
    dialogSeasonsList.value = []
    dialogEpisodesList.value = []
    dialogEmbyServer.value = embyServer
    dialogSeries.value = series
    dialogSeriesVisible.value = true
    if (seasons_result.value[series.Id]) {
        dialogSeasonsList.value = seasons_result.value[series.Id].Items
        dialogSeasonsLoading.value = false
        return
    }
    return embyApi.seasons(embyServer, series.Id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<SeasonsItems> = JSON.parse(response.body);
        seasons_result.value[series.Id] = json
        dialogSeasonsList.value = json.Items
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => dialogSeasonsLoading.value = false)
}
async function getEpisodes(embyServer: EmbyServer, series_id: string, seasons: SeasonsItems, currentPage: number, pageSize: number) {
    dialogEpisodesLoading.value = true
    dialogEpisodesList.value = []
    dialogEpisodesCurrentPage.value = currentPage
    dialogEpisodesPageSize.value = pageSize
    dialogSeasons.value = seasons
    if (!episodes_result.value[series_id + '|' + seasons.Id]) {
        episodes_result.value[series_id + '|' + seasons.Id] = {total: 0}
    }
    if (episodes_result.value[series_id + '|' + seasons.Id][currentPage]) {
        dialogEpisodesList.value = episodes_result.value[series_id + '|' + seasons.Id][currentPage]
        dialogEpisodesLoading.value = false
        return
    }
    return embyApi.episodes(embyServer, series_id, seasons.Id, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        episodes_result.value[series_id + '|' + seasons.Id].total = json.TotalRecordCount
        episodes_result.value[series_id + '|' + seasons.Id][currentPage]= json.Items
        dialogEpisodesList.value = json.Items
        for (let item of json.Items) {
            getTag(seasons.Id, item.MediaSources)
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => dialogEpisodesLoading.value = false)
}
async function handleEpisodesPageChange(val: number, embyServer: EmbyServer, series_id: string, seasons: SeasonsItems) {
    await getEpisodes(embyServer, series_id, seasons, val, dialogEpisodesPageSize.value)
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
  margin-bottom: 0;
}

.el-scrollbar {
  height: 100%;
}
</style>