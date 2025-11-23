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
                {{ 'S' + ((item as EpisodeItem).ParentIndexNumber || '-') + 'E' + ((item as EpisodeItem).IndexNumber || '-') + '. ' + item.Name }}
            </el-link>
            <el-link v-else-if="item.Type == 'Season'" :underline="false" @click="showSeason(item as SeasonItem)">
                {{ 'S' + ((item as SeasonItem).IndexNumber || '-') + '. ' + item.Name }}
            </el-link>
            <el-link v-else-if="item.Type == 'Movie'" :underline="false" @click="gotoEpisodes(item.Id)">{{ item.Name }}</el-link>
        </div>
        <div style="margin: 10px 0;">
            <span v-if="item.Type == 'Series'">
                {{ item.ProductionYear + ((item as SeriesItem).EndDate && (item as SeriesItem).EndDate.substring(0, 4) != item.ProductionYear + '' ? '-' + (item as SeriesItem).EndDate.substring(0, 4) : '') }}
            </span>
            <span v-else-if="item.Type == 'Episode'" style="display: flex;justify-content: space-between;align-items: center;">
                <span>{{ (item as EpisodeItem).PremiereDate ? (item as EpisodeItem).PremiereDate.substring(0, 10) : '' }}</span>
                <span style="display: flex; flex-direction: column;">
                    <el-tag disable-transitions v-for="value in mediaSourceTag[item.Id]">{{ value }}</el-tag>
                </span>
            </span>
            <span v-else-if="item.Type == 'Movie'" style="display: flex;justify-content: space-between;align-items: center;">
                <span>{{ item.ProductionYear }}</span>
                <span style="display: flex; flex-direction: column;">
                    <el-tag disable-transitions v-for="value in mediaSourceTag[item.Id]">{{ value }}</el-tag>
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
                            @click="getEpisodes(dialogEmbyServerId!, dialogSeries!.Id, seasonItem, 1, 10)"
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
                        v-if="episodes_result[dialogSeries!.Id + '|' + dialogSeasons?.Id]"
                        v-model:current-page="dialogEpisodesCurrentPage"
                        v-model:page-size="dialogEpisodesPageSize"
                        layout="total, prev, pager, next, jumper"
                        :total="episodes_result[dialogSeries!.Id + '|' + dialogSeasons!.Id].total"
                        @current-change="handleEpisodesPageChange(dialogEpisodesCurrentPage, dialogEmbyServerId!, dialogSeries!.Id, dialogSeasons!)"
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
                        <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)">
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
                v-if="episodes_result[dialogSeasons!.SeriesId + '|' + dialogSeasons?.Id]"
                v-model:current-page="dialogEpisodesCurrentPage"
                v-model:page-size="dialogEpisodesPageSize"
                layout="total, prev, pager, next, jumper"
                :total="episodes_result[dialogSeasons!.SeriesId + '|' + dialogSeasons!.Id].total"
                @current-change="handleEpisodesPageChange(dialogEpisodesCurrentPage, dialogEmbyServerId!, dialogSeasons!.SeriesId, dialogSeasons!)"
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
import { formatBytes } from '../util/str_util'
import { getResolutionFromMediaSources } from '../util/play_info_util'

const router = useRouter()

const {item, embyServerId, showSeriesName} = defineProps<{
  item: SearchItem,
  embyServerId: string,
  showSeriesName?: boolean,
}>()

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
getTag(item.Id, (item as EpisodeItem).MediaSources)

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServerId + '/series/' + seriesId)
}

const starLoading = ref<{[key: string]: boolean}>({})
function star(item: SearchItem | SeasonItem | EpisodeItem) {
    if (!item.UserData) {
        return
    }
    starLoading.value[item.Id] = true
    let fun;
    if (item.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServerId!, item.Id)
    } else {
        fun = embyApi.star(embyServerId!, item.Id)
    }
    return fun.then(async response => {
        let json: UserData = JSON.parse(response);
        item.UserData!.IsFavorite = json.IsFavorite
    }).catch(e => ElMessage.error(e)).finally(() => starLoading.value[item.Id] = false)
}

const playedLoading = ref<{[key: string]: boolean}>({})
function played(item: SearchItem | SeasonItem | EpisodeItem) {
    if (!item.UserData) {
        return
    }
    playedLoading.value[item.Id] = true
    let fun;
    if (item.UserData.Played) {
        fun = embyApi.unplayed(embyServerId!, item.Id)
    } else {
        fun = embyApi.played(embyServerId!, item.Id)
    }
    return fun.then(async response => {
        let json: UserData = JSON.parse(response);
        item.UserData!.Played = json.Played
    }).catch(e => ElMessage.error(e)).finally(() => playedLoading.value[item.Id] = false)
}

const seasons_result = ref<{[key: string]: EmbyPageList<SeasonItem>}>({})
const episodes_result = ref<{[key: string]: {total: number, [key: number]: EpisodeItem[]}}>({})

const dialogSeriesVisible = ref(false)
const dialogSeasonsVisible = ref(false)
const dialogEmbyServerId = ref<string>()
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
    dialogEmbyServerId.value = embyServerId
    dialogSeries.value = series
    dialogSeriesVisible.value = true
    if (seasons_result.value[series.Id]) {
        dialogSeasonsList.value = seasons_result.value[series.Id].Items
        dialogSeasonsLoading.value = false
        return
    }
    return embyApi.seasons(embyServerId!, series.Id).then(async response => {
        let json: EmbyPageList<SeasonItem> = JSON.parse(response);
        seasons_result.value[series.Id] = json
        dialogSeasonsList.value = json.Items
    }).catch(e => ElMessage.error(e)).finally(() => dialogSeasonsLoading.value = false)
}
async function getEpisodes(embyServerId: string, series_id: string, seasons: SeasonItem, currentPage: number, pageSize: number) {
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
    return embyApi.episodes(embyServerId!, series_id, seasons.Id, (currentPage - 1) * pageSize, pageSize).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
        episodes_result.value[series_id + '|' + seasons.Id].total = json.TotalRecordCount
        episodes_result.value[series_id + '|' + seasons.Id][currentPage]= json.Items
        dialogEpisodesList.value = json.Items
        for (let item of json.Items) {
            getTag(item.Id, item.MediaSources)
        }
    }).catch(e => ElMessage.error(e)).finally(() => dialogEpisodesLoading.value = false)
}
async function handleEpisodesPageChange(val: number, embyServerId: string, series_id: string, seasons: SeasonItem) {
    await getEpisodes(embyServerId, series_id, seasons, val, dialogEpisodesPageSize.value)
}
async function showSeason(season: SeasonItem) {
    dialogSeasons.value = season
    dialogEpisodesList.value = []
    dialogEmbyServerId.value = embyServerId
    dialogSeasonsVisible.value = true
    getEpisodes(embyServerId, season.SeriesId, season, 1, 10)
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