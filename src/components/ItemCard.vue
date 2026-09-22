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

/** 点击封面：按条目类型走最合理的一步 */
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
            let resolution = getResolutionFromMediaSources(mediaSource)
            mediaSourceTag.value[itemId].push(size + " · " + resolution)
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
/* 卡片：信息分四行，行距即节奏，不使用投影 */
.card {
    display: flex;
    gap: 12px;
    padding: 12px;
    width: 344px;
    margin: 0;
    background: var(--ink-card);
    border: 1px solid var(--hairline);
    border-radius: var(--radius);
}

.card__poster {
    position: relative;
    flex: none;
    width: 112px;
    height: 158px;
    cursor: pointer;
}

.card__poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

/* 播放进度：贴底的一条灯带 */
.card__progress {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 3px;
    background: rgba(10, 12, 16, 0.7);
}

.card__progress-fill {
    display: block;
    height: 100%;
    background: var(--lamp);
}

.card__pending {
    position: absolute;
    top: 6px;
    right: 6px;
    min-width: 20px;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(10, 12, 16, 0.82);
    color: var(--lamp);
    font-family: var(--font-mono);
    font-size: 11px;
    text-align: center;
}

.card__body {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: auto;
}

.card__line1 {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
}

.card__index {
    flex: none;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
}

.card__title {
    font-size: var(--text-base);
    font-weight: 600;
    line-height: 1.35;
    color: var(--text-1);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
}

.card__series {
    margin-top: 3px;
    font-size: var(--text-sm);
}

.card__facts {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px;
    margin-top: 8px;
    font-size: var(--text-xs);
    color: var(--text-3);
}

.card__mono {
    font-family: var(--font-mono);
}

.card__quality {
    padding: 0 5px;
    border: 1px solid var(--hairline);
    border-radius: 4px;
}

/* 卡片操作：默认安静，悬停/聚焦时出现 */
.card__ops {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: auto;
    padding-top: 10px;
}

.card__op {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-3);
    font-family: inherit;
    font-size: var(--text-xs);
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
}

.card__op:hover:not(:disabled) {
    color: var(--text-1);
    border-color: #333C48;
}

.card__op.is-on {
    color: var(--lamp);
    border-color: var(--lamp-line);
}

.card__op.is-done {
    color: #6FBF7C;
    border-color: rgba(79, 163, 94, 0.4);
}

.card__op:disabled {
    cursor: default;
    opacity: 0.6;
}

.card__more {
    margin-left: auto;
    padding: 4px 2px;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-xs);
    cursor: pointer;
}

.card__more:hover {
    color: var(--lamp);
}

/* 季 / 集 选择器：左列表右剧集，单层边框 */
.season-picker {
    display: flex;
    gap: 18px;
    height: 460px;
}

.season-picker__aside {
    flex: none;
    width: 208px;
    border-right: 1px solid var(--hairline);
    padding-right: 12px;
    overflow: hidden;
}

.season-picker__main {
    flex: auto;
    min-width: 0;
    overflow: hidden;
}

.season-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 9px 10px;
    margin-bottom: 2px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-sm);
    text-align: left;
    cursor: pointer;
}

.season-item:hover {
    background: #1A2028;
    color: var(--text-1);
}

.season-item.is-active {
    background: var(--lamp-soft);
    color: var(--text-1);
    box-shadow: inset 2px 0 0 var(--lamp);
}

.season-item__name {
    font-weight: 500;
    color: inherit;
}

.season-item__foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.season-item__year,
.season-item__pending {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
}

.season-item__ops {
    display: flex;
    align-items: center;
    gap: 6px;
}

.season-icon {
    color: var(--text-3);
}

.season-icon--star.is-on {
    color: var(--lamp);
}

.season-icon--done.is-on {
    color: #6FBF7C;
}

.episode-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 10px;
    border: none;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    color: var(--text-1);
    font-family: inherit;
    font-size: var(--text-sm);
    text-align: left;
    cursor: pointer;
}

.episode-row:hover {
    background: #1A2028;
}

.episode-row__num {
    flex: none;
    width: 22px;
    font-family: var(--font-mono);
    color: var(--text-3);
}

.episode-row__name {
    flex: auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.episode-row__tag {
    flex: none;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
    border: 1px solid var(--hairline);
    border-radius: 4px;
    padding: 0 5px;
}

.episode-row__date {
    flex: none;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
}

.episode-row__ops {
    flex: none;
    display: flex;
    align-items: center;
    gap: 6px;
}

.season-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding-bottom: 12px;
    margin-bottom: 6px;
    border-bottom: 1px solid var(--hairline);
}

.season-head__overview {
    margin: 0;
    max-width: 62ch;
    color: var(--text-2);
    font-size: var(--text-sm);
    line-height: 1.7;
}

.season-head__ops {
    flex: none;
    display: flex;
    gap: 6px;
}
</style>
