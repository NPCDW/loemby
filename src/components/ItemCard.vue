<template>
    <article class="card">
        <!-- 封面：状态用零散信息表达，不堆标签 -->
        <div class="card__poster loe-cover-img" @click="openPrimary()">
            <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" />
            <span v-if="item.UserData?.Played" class="card__progress">
                <span class="card__progress-fill" style="width: 100%"></span>
            </span>
            <span v-else-if="progressPercent > 0" class="card__progress">
                <span class="card__progress-fill" :style="{width: progressPercent + '%'}"></span>
            </span>
            <span v-if="item.UserData?.UnplayedItemCount" class="card__pending">{{ item.UserData.UnplayedItemCount }}</span>
        </div>

        <div class="card__body">
            <div class="card__line1">
                <span v-if="item.Type == 'Episode'" class="card__index">
                    {{ 'S' + ((item as EpisodeItem).ParentIndexNumber || '-') + ' E' + ((item as EpisodeItem).IndexNumber || '-') }}
                </span>
                <span class="card__title" :title="item.Name">{{ item.Name }}</span>
            </div>

            <div v-if="showSeriesName && ((item as EpisodeItem).SeriesName)" class="card__series">
                <el-link :underline="false" @click="gotoSeries((item as EpisodeItem).SeriesId)">{{ (item as EpisodeItem).SeriesName }}</el-link>
            </div>

            <div class="card__facts">
                <span v-if="yearText">{{ yearText }}</span>
                <span v-if="dateText" class="card__mono">{{ dateText }}</span>
                <span v-for="value in mediaSourceTag[item.Id]" :key="value" class="card__mono card__quality">{{ value }}</span>
            </div>

            <div class="card__ops">
                <button
                    class="card__op"
                    :class="{ 'is-on': item.UserData?.IsFavorite }"
                    :disabled="starLoading[item.Id]"
                    :title="item.UserData?.IsFavorite ? '取消收藏' : '收藏'"
                    @click="star(item)"
                >
                    <el-icon :size="16" :class="starLoading[item.Id] ? 'is-loading' : ''">
                        <i-ep-StarFilled v-if="item.UserData?.IsFavorite" />
                        <i-ep-Star v-else />
                    </el-icon>
                </button>
                <button
                    class="card__op"
                    :class="{ 'is-done': item.UserData?.Played }"
                    :disabled="playedLoading[item.Id]"
                    :title="item.UserData?.Played ? '标记为未播放' : '标记为已播放'"
                    @click="played(item)"
                >
                    <el-icon :size="16" :class="playedLoading[item.Id] ? 'is-loading' : ''">
                        <i-ep-CircleCheckFilled v-if="item.UserData?.Played" />
                        <i-ep-CircleCheck v-else />
                    </el-icon>
                </button>
                <button v-if="item.Type == 'Series' || item.Type == 'Season'" class="card__more" @click="openList()">
                    展开剧集
                </button>
            </div>
        </div>
    </article>

    <el-dialog
        v-model="dialogSeriesVisible"
        :title="dialogSeries?.Name"
        width="860"
    >
        <div class="season-picker">
            <div class="season-picker__aside">
                <el-scrollbar>
                    <el-skeleton :loading="dialogSeasonsLoading" animated>
                        <template #template>
                            <div class="season-item" v-for="i in 5" :key="i">
                                <el-skeleton-item variant="text" style="width: 60%" />
                                <el-skeleton-item variant="text" style="width: 35%; margin-top: 6px;" />
                            </div>
                        </template>
                        <button
                            v-for="seasonItem in dialogSeasonsList"
                            :key="seasonItem.Id"
                            class="season-item"
                            :class="{ 'is-active': dialogSeasons?.Id === seasonItem.Id }"
                            @click="getEpisodes(dialogEmbyServerId!, dialogSeries!.Id, seasonItem, 1, 10)"
                        >
                            <span class="season-item__name">{{ 'S' + seasonItem.IndexNumber + '. ' + seasonItem.Name }}</span>
                            <span class="season-item__foot">
                                <span class="season-item__year">{{ seasonItem.ProductionYear }}</span>
                                <span class="season-item__ops">
                                    <el-link :underline="false" v-if="seasonItem.UserData" :disabled="starLoading[seasonItem.Id]" @click.stop="star(seasonItem)">
                                        <el-icon :size="16" class="season-icon season-icon--star" :class="[seasonItem.UserData.IsFavorite ? 'is-on' : '', starLoading[seasonItem.Id] ? 'is-loading' : '']">
                                            <i-ep-StarFilled v-if="seasonItem.UserData.IsFavorite" />
                                            <i-ep-Star v-else />
                                        </el-icon>
                                    </el-link>
                                    <el-link :underline="false" v-if="seasonItem.UserData" :disabled="playedLoading[seasonItem.Id]" @click.stop="played(seasonItem)">
                                        <el-icon :size="16" class="season-icon season-icon--done" :class="[seasonItem.UserData.Played ? 'is-on' : '', playedLoading[seasonItem.Id] ? 'is-loading' : '']">
                                            <i-ep-CircleCheckFilled v-if="seasonItem.UserData.Played" />
                                            <i-ep-CircleCheck v-else />
                                        </el-icon>
                                    </el-link>
                                    <span v-if="seasonItem.UserData?.UnplayedItemCount" class="season-item__pending">{{ seasonItem.UserData.UnplayedItemCount }}</span>
                                </span>
                            </span>
                        </button>
                    </el-skeleton>
                </el-scrollbar>
            </div>
            <div class="season-picker__main">
                <el-scrollbar>
                    <el-skeleton :loading="dialogEpisodesLoading" animated>
                        <template #template>
                            <div class="episode-row" v-for="i in 6" :key="i">
                                <el-skeleton-item variant="text" style="width: 45%" />
                                <el-skeleton-item variant="text" style="width: 25%" />
                            </div>
                        </template>
                        <button v-for="episodeItem in dialogEpisodesList" :key="episodeItem.Id" class="episode-row" @click="gotoEpisodes(episodeItem.Id)">
                            <span class="episode-row__num">{{ episodeItem.IndexNumber }}</span>
                            <span class="episode-row__name">{{ episodeItem.Name }}</span>
                            <span class="episode-row__tag" v-for="value in mediaSourceTag[episodeItem.Id]" :key="value">{{ value }}</span>
                            <span class="episode-row__date">{{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}</span>
                            <span class="episode-row__ops" @click.stop>
                                <el-link :underline="false" v-if="episodeItem.UserData" :disabled="starLoading[episodeItem.Id]" @click="star(episodeItem)">
                                    <el-icon :size="16" class="season-icon season-icon--star" :class="[episodeItem.UserData.IsFavorite ? 'is-on' : '', starLoading[episodeItem.Id] ? 'is-loading' : '']">
                                        <i-ep-StarFilled v-if="episodeItem.UserData.IsFavorite" />
                                        <i-ep-Star v-else />
                                    </el-icon>
                                </el-link>
                                <el-link :underline="false" v-if="episodeItem.UserData" :disabled="playedLoading[episodeItem.Id]" @click="played(episodeItem)">
                                    <el-icon :size="16" class="season-icon season-icon--done" :class="[episodeItem.UserData.Played ? 'is-on' : '', playedLoading[episodeItem.Id] ? 'is-loading' : '']">
                                        <i-ep-CircleCheckFilled v-if="episodeItem.UserData.Played" />
                                        <i-ep-CircleCheck v-else />
                                    </el-icon>
                                </el-link>
                            </span>
                        </button>
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
        width="860"
    >
        <div class="season-head">
            <p class="season-head__overview">{{ dialogSeasons?.Overview || '暂无简介。' }}</p>
            <div class="season-head__ops">
                <button class="card__op" :class="{ 'is-done': dialogSeasons?.UserData?.Played }" :disabled="playedLoading[dialogSeasons!.Id]" @click="played(dialogSeasons!)">
                    <el-icon :size="16" :class="playedLoading[dialogSeasons!.Id] ? 'is-loading' : ''">
                        <i-ep-CircleCheckFilled v-if="dialogSeasons?.UserData?.Played" />
                        <i-ep-CircleCheck v-else />
                    </el-icon>
                    <span>{{ dialogSeasons?.UserData?.Played ? '已播放' : '标记已播放' }}</span>
                </button>
                <button class="card__op" :class="{ 'is-on': dialogSeasons?.UserData?.IsFavorite }" :disabled="starLoading[dialogSeasons!.Id]" @click="star(dialogSeasons!)">
                    <el-icon :size="16" :class="starLoading[dialogSeasons!.Id] ? 'is-loading' : ''">
                        <i-ep-StarFilled v-if="dialogSeasons?.UserData?.IsFavorite" />
                        <i-ep-Star v-else />
                    </el-icon>
                    <span>{{ dialogSeasons?.UserData?.IsFavorite ? '取消收藏' : '收藏' }}</span>
                </button>
            </div>
        </div>
        <el-scrollbar style="max-height: 60vh;">
            <el-skeleton :loading="dialogEpisodesLoading" animated>
                <template #template>
                    <div class="episode-row" v-for="i in 6" :key="i">
                        <el-skeleton-item variant="text" style="width: 45%" />
                        <el-skeleton-item variant="text" style="width: 25%" />
                    </div>
                </template>
                <button v-for="episodeItem in dialogEpisodesList" :key="episodeItem.Id" class="episode-row" @click="gotoEpisodes(episodeItem.Id)">
                    <span class="episode-row__num">{{ episodeItem.IndexNumber }}</span>
                    <span class="episode-row__name">{{ episodeItem.Name }}</span>
                    <span class="episode-row__tag" v-for="value in mediaSourceTag[episodeItem.Id]" :key="value">{{ value }}</span>
                    <span class="episode-row__date">{{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}</span>
                    <span class="episode-row__ops" @click.stop>
                        <el-link :underline="false" v-if="episodeItem.UserData" :disabled="starLoading[episodeItem.Id]" @click="star(episodeItem)">
                            <el-icon :size="16" class="season-icon season-icon--star" :class="[episodeItem.UserData.IsFavorite ? 'is-on' : '', starLoading[episodeItem.Id] ? 'is-loading' : '']">
                                <i-ep-StarFilled v-if="episodeItem.UserData.IsFavorite" />
                                <i-ep-Star v-else />
                            </el-icon>
                        </el-link>
                        <el-link :underline="false" v-if="episodeItem.UserData" :disabled="playedLoading[episodeItem.Id]" @click="played(episodeItem)">
                            <el-icon :size="16" class="season-icon season-icon--done" :class="[episodeItem.UserData.Played ? 'is-on' : '', playedLoading[episodeItem.Id] ? 'is-loading' : '']">
                                <i-ep-CircleCheckFilled v-if="episodeItem.UserData.Played" />
                                <i-ep-CircleCheck v-else />
                            </el-icon>
                        </el-link>
                    </span>
                </button>
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
import { computed, ref } from 'vue';
import embyApi, { EmbyPageList, EpisodeItem, MediaSource, SearchItem, SeasonItem, SeriesItem, UserData } from '../api/embyApi';
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus';
import { formatBytes } from '../util/str_util'
import { getResolutionFromMediaSources } from '../util/play_info_util'
import { useImage } from '../store/image'

const router = useRouter()

const {item, embyServerId, showSeriesName} = defineProps<{
  item: SearchItem,
  embyServerId: string,
  showSeriesName?: boolean,
}>()

/* —— 仅用于展示的派生值，不改动原有取数逻辑 —— */
const progressPercent = computed(() => {
    const data = item.UserData
    if (!data || data.Played) return 0
    return Math.trunc(data.PlayedPercentage || 0)
})

const yearText = computed(() => {
    if (item.Type == 'Series') {
        const start = item.ProductionYear
        const end = (item as SeriesItem).EndDate && (item as SeriesItem).EndDate!.substring(0, 4) != item.ProductionYear + ''
            ? (item as SeriesItem).EndDate!.substring(0, 4)
            : ''
        return end ? start + ' – ' + end : start
    }
    return item.ProductionYear
})

const dateText = computed(() => {
    if (item.Type == 'Episode') {
        return (item as EpisodeItem).PremiereDate ? (item as EpisodeItem).PremiereDate!.substring(0, 10) : ''
    }
    return ''
})

function openPrimary() {
    if (item.Type == 'Series') {
        gotoSeries(item.Id)
    } else if (item.Type == 'Season') {
        showSeason(item as SeasonItem)
    } else {
        gotoEpisodes(item.Id)
    }
}

function openList() {
    if (item.Type == 'Series') {
        showSeries(item as SeriesItem)
    } else {
        showSeason(item as SeasonItem)
    }
}

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
