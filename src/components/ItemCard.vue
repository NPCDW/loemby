<template>
    <el-card style="width: 300px; margin: 5px;">
        <div v-if="showSeriesName">
            <el-link v-if="item.Type == 'Episode' || item.Type == 'Season'" :underline="false" @click="gotoSeries((item as EpisodeItem).SeriesId)">
                <h2>{{ (item as EpisodeItem).SeriesName }}</h2>
            </el-link>
        </div>
        <div>
            <el-link v-if="item.Type == 'Series'" :underline="false" @click="gotoSeries(item.Id)">{{ item.Name }}</el-link>
            <el-link v-else-if="item.Type == 'Episode'" :underline="false" @click="gotoEpisodes(item.Id)">
                {{ 'S' + ((item as EpisodeItem).ParentIndexNumber || -1) + 'E' + ((item as EpisodeItem).IndexNumber || -1) + '. ' + item.Name }}
            </el-link>
            <el-link v-else-if="item.Type == 'Season'" :underline="false" @click="showSeason(item as SeasonItem)">
                {{ 'S' + ((item as SeasonItem).IndexNumber || -1) + '. ' + item.Name }}
            </el-link>
            <el-link v-else-if="item.Type == 'Movie'" :underline="false" @click="gotoEpisodes(item.Id)">{{ item.Name }}</el-link>
        </div>
        <div style="margin: 10px 0;">
            <span v-if="item.Type == 'Series'">
                {{ item.ProductionYear + ((item as SeriesItem).EndDate && (item as SeriesItem).EndDate.substring(0, 4) != item.ProductionYear + '' ? '-' + (item as SeriesItem).EndDate.substring(0, 4) : '') }}
            </span>
            <span v-else-if="item.Type == 'Episode'" style="display: flex;justify-content: space-between;align-items: center;">
                <span>{{ (item as EpisodeItem).PremiereDate ? (item as EpisodeItem).PremiereDate.substring(0, 10) : '' }}</span>
                <span>
                    <el-tag disable-transitions>{{ mediaSourceSizeTag[item.Id] || "0 KB" }}</el-tag>
                    <!-- <el-tag disable-transitions>{{ mediaSourceBitrateTag[item.Id] || "0 Kbps" }}</el-tag> -->
                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[item.Id] || 'Unknown' }}</el-tag>
                </span>
            </span>
            <span v-else-if="item.Type == 'Movie'" style="display: flex;justify-content: space-between;align-items: center;">
                <span>{{ item.ProductionYear }}</span>
                <span>
                    <el-tag disable-transitions>{{ mediaSourceSizeTag[item.Id] || "0 KB" }}</el-tag>
                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaSourceBitrateTag[item.Id] || "0 Kbps" }}</el-tag>
                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[item.Id] || 'Unknown' }}</el-tag>
                </span>
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
            <span v-if="item.Type == 'Series'">
                <el-badge :value="item.UserData?.UnplayedItemCount" :max="999" :show-zero="false" type="primary">
                    <el-button @click="showSeries(item as SeriesItem)" type="primary" plain>剧集</el-button>
                </el-badge>
            </span>
            <span v-if="item.Type == 'Season'">
                <el-badge :value="item.UserData?.UnplayedItemCount" :max="999" :show-zero="false" type="primary">
                    <el-button @click="showSeason(item as SeasonItem)" type="primary" plain>剧集</el-button>
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
                            <div class="box-item" v-for="i in 5" :key="i">
                                <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                                <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                            </div>
                        </template>
                        <div
                            v-for="seasonItem in dialogSeasonsList"
                            :key="seasonItem.Id"
                            class="box-item"
                            :class="{ active: dialogSeasons?.Id === seasonItem.Id }"
                            @click="getEpisodes(dialogEmbyServer!, dialogSeries!.Id, seasonItem, 1, 10)"
                        >
                            <h3>{{ 'S' + seasonItem.IndexNumber + '. ' + seasonItem.Name }}</h3>
                            <div style="display: flex;justify-content: space-between;">
                                <span>
                                    <span style="margin-right: 10px;">{{ seasonItem.ProductionYear }}</span>
                                    <el-tag type="primary" effect="dark" round disable-transitions>{{ seasonItem.UserData?.UnplayedItemCount }}</el-tag>
                                </span>
                                <span>
                                    <el-link :underline="false" v-if="seasonItem.UserData" :disabled="starLoading[seasonItem.Id]" @click="star(seasonItem)">
                                        <el-icon color="#E6A23C" :size="24" :class="starLoading[seasonItem.Id] ? 'is-loading' : ''" v-if="seasonItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                        <el-icon :size="24" :class="starLoading[seasonItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                                    </el-link>
                                    <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[seasonItem.Id]" v-if="seasonItem.UserData" @click="played(seasonItem)">
                                        <el-icon color="#67C23A" :size="24" :class="playedLoading[seasonItem.Id] ? 'is-loading' : ''" v-if="seasonItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                        <el-icon :size="24" :class="playedLoading[seasonItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
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
                            <div class="box-item" v-for="i in 5" :key="i">
                                <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                                <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                            </div>
                        </template>
                        <div v-for="episodeItem in dialogEpisodesList" class="box-item">
                            <p>
                                <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)">
                                    {{ episodeItem.IndexNumber + '. ' + episodeItem.Name }}
                                </el-link>
                            </p>
                            <div style="display: flex;justify-content: space-between;">
                                <span>
                                    <span>{{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}</span>
                                    <el-tag disable-transitions style="margin-left: 10px;">{{ mediaSourceSizeTag[episodeItem.Id] || "0 KB" }}</el-tag>
                                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaSourceBitrateTag[episodeItem.Id] || "0 Kbps" }}</el-tag>
                                    <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[episodeItem.Id] || 'Unknown' }}</el-tag>
                                </span>
                                <span>
                                    <el-link :underline="false" v-if="episodeItem.UserData" :disabled="starLoading[episodeItem.Id]" @click="star(episodeItem)">
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
    <el-dialog
        v-model="dialogSeasonsVisible"
        :title="dialogSeasons?.Name"
        width="800"
    >
        <el-scrollbar style="padding: 0 20px;">
            <p>简介：{{ dialogSeasons?.Overview }}</p>
            <p>
                <el-button plain :disabled="playedLoading[dialogSeasons!.Id]" @click="played(dialogSeasons!)">
                    <el-icon color="#67C23A" :size="24" :class="playedLoading[dialogSeasons!.Id] ? 'is-loading' : ''" v-if="dialogSeasons?.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                    <el-icon :size="24" :class="playedLoading[dialogSeasons!.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                    <span>已播放</span>
                </el-button>
                <el-button plain :disabled="starLoading[dialogSeasons!.Id]" @click="star(dialogSeasons!)">
                    <template v-if="dialogSeasons?.UserData?.IsFavorite">
                        <el-icon color="#E6A23C" :size="24" :class="starLoading[dialogSeasons!.Id] ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                        <span>取消收藏</span>
                    </template>
                    <template v-else>
                        <el-icon :size="24" :class="starLoading[dialogSeasons!.Id] ? 'is-loading' : ''"><i-ep-Star /></el-icon>
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
                        <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)">
                            {{ episodeItem.IndexNumber + '. ' + episodeItem.Name }}
                        </el-link>
                    </p>
                    <div style="display: flex;justify-content: space-between;">
                        <span>
                            <span>{{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}</span>
                            <el-tag disable-transitions style="margin-left: 10px;">{{ mediaSourceSizeTag[episodeItem.Id] || "0 KB" }}</el-tag>
                            <el-tag disable-transitions style="margin-left: 5px;">{{ mediaSourceBitrateTag[episodeItem.Id] || "0 Kbps" }}</el-tag>
                            <el-tag disable-transitions style="margin-left: 5px;">{{ mediaStreamResolutionTag[episodeItem.Id] || 'Unknown' }}</el-tag>
                        </span>
                        <span>
                            <el-link :underline="false" v-if="episodeItem.UserData" :disabled="starLoading[episodeItem.Id]" @click="star(episodeItem)">
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
                v-if="episodes_result[dialogSeasons!.SeriesId + '|' + dialogSeasons?.Id]"
                v-model:current-page="dialogEpisodesCurrentPage"
                v-model:page-size="dialogEpisodesPageSize"
                layout="total, prev, pager, next, jumper"
                :total="episodes_result[dialogSeasons!.SeriesId + '|' + dialogSeasons!.Id].total"
                @current-change="handleEpisodesPageChange(dialogEpisodesCurrentPage, dialogEmbyServer!, dialogSeasons!.SeriesId, dialogSeasons!)"
                hide-on-single-page
            />
        </el-scrollbar>
    </el-dialog>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import embyApi, { EmbyPageList, EpisodeItem, MediaSource, SearchItem, SeasonItem, SeriesItem, UserData } from '../api/embyApi';
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus';
import { formatBytes, formatMbps } from '../util/str_util'
import { getResolutionFromMediaSources, maxMediaSources } from '../util/play_info_util'
import { EmbyServer } from '../store/db/embyServer';

const router = useRouter()

const {item, embyServer} = defineProps<{
  item: SearchItem,
  embyServer: EmbyServer,
  showSeriesName?: boolean,
}>()

const mediaSourceSizeTag = ref<{[key: string]: string}>({})
const mediaSourceBitrateTag = ref<{[key: string]: string}>({})
const mediaStreamResolutionTag = ref<{[key: string]: string}>({})
function getTag(itemId: string, mediaSources?: MediaSource[]) {
    let maxMediaSource = maxMediaSources(mediaSources);
    if (maxMediaSource) {
        mediaSourceSizeTag.value[itemId] = formatBytes(maxMediaSource.Size)
        mediaSourceBitrateTag.value[itemId] = formatMbps(maxMediaSource.Bitrate)
        if (maxMediaSource.MediaStreams && maxMediaSource.MediaStreams.length > 0) {
            mediaStreamResolutionTag.value[itemId] = getResolutionFromMediaSources(maxMediaSource)
        }
    }
}
getTag(item.Id, (item as EpisodeItem).MediaSources)

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/series/' + seriesId)
}

const starLoading = ref<{[key: string]: boolean}>({})
function star(item: SearchItem | SeasonItem | EpisodeItem) {
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
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        item.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => starLoading.value[item.Id] = false)
}

const playedLoading = ref<{[key: string]: boolean}>({})
function played(item: SearchItem | SeasonItem | EpisodeItem) {
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
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: UserData = JSON.parse(response.body);
        item.UserData!.Played = json.Played
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => playedLoading.value[item.Id] = false)
}

const seasons_result = ref<{[key: string]: EmbyPageList<SeasonItem>}>({})
const episodes_result = ref<{[key: string]: {total: number, [key: number]: EpisodeItem[]}}>({})

const dialogSeriesVisible = ref(false)
const dialogSeasonsVisible = ref(false)
const dialogEmbyServer = ref<EmbyServer>()
const dialogSeries = ref<SearchItem>()
const dialogSeasons = ref<SeasonItem>()
const dialogSeasonsList = ref<SeasonItem[]>([])
const dialogEpisodesList = ref<EpisodeItem[]>([])
const dialogEpisodesCurrentPage = ref(1)
const dialogEpisodesPageSize = ref(10)
const dialogSeasonsLoading = ref(false)
const dialogEpisodesLoading = ref(false)

async function showSeries(series: SeriesItem) {
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
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<SeasonItem> = JSON.parse(response.body);
        seasons_result.value[series.Id] = json
        dialogSeasonsList.value = json.Items
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => dialogSeasonsLoading.value = false)
}
async function getEpisodes(embyServer: EmbyServer, series_id: string, seasons: SeasonItem, currentPage: number, pageSize: number) {
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
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response.body);
        episodes_result.value[series_id + '|' + seasons.Id].total = json.TotalRecordCount
        episodes_result.value[series_id + '|' + seasons.Id][currentPage]= json.Items
        dialogEpisodesList.value = json.Items
        for (let item of json.Items) {
            getTag(item.Id, item.MediaSources)
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => dialogEpisodesLoading.value = false)
}
async function handleEpisodesPageChange(val: number, embyServer: EmbyServer, series_id: string, seasons: SeasonItem) {
    await getEpisodes(embyServer, series_id, seasons, val, dialogEpisodesPageSize.value)
}
async function showSeason(season: SeasonItem) {
    dialogSeasons.value = season
    dialogEpisodesList.value = []
    dialogEmbyServer.value = embyServer
    dialogSeasonsVisible.value = true
    getEpisodes(embyServer, season.SeriesId, season, 1, 10)
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

.box-item {
  padding: 3px 10px;
  cursor: pointer;
  border-bottom: 1px solid #18222C;
}

.box-item:hover {
  background-color: #18222C;
}

.box-item.active {
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