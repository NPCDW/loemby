<template>
    <div class="roe-page">
        <el-skeleton :loading="serieInfoLoading" animated>
            <template #template>
                <div class="hero">
                    <el-skeleton-item variant="image" style="height: 400px; width: 272px; border-radius: 10px;" />
                    <div class="hero__main">
                        <el-skeleton-item variant="h1" style="width: 45%;" />
                        <el-skeleton-item variant="text" style="width: 100%; margin-top: 16px;" />
                        <el-skeleton-item variant="text" style="width: 100%; margin-top: 8px;" />
                        <el-skeleton-item variant="text" style="width: 70%; margin-top: 8px;" />
                    </div>
                </div>
            </template>
            <div class="hero" v-if="currentSeries">
                <div class="hero__poster loe-cover-img">
                    <img v-lazy="useImage().images[embyServerId + ':cover:' + currentSeries.Id]" />
                </div>
                <div class="hero__main">
                    <h1 class="hero__title">{{ currentSeries.Name }}</h1>
                    <div class="hero__facts">
                        <span v-if="currentSeries.ProductionYear" class="mono">{{ currentSeries.ProductionYear }}</span>
                        <span v-if="seasonsList.length" class="mono">{{ seasonsList.length }} 季</span>
                        <span v-if="currentSeries.OfficialRating" class="chip">{{ currentSeries.OfficialRating }}</span>
                        <span v-for="genre in (currentSeries.Genres || []).slice(0, 4)" :key="genre" class="chip">{{ genre }}</span>
                    </div>
                    <p class="hero__overview">{{ currentSeries.Overview || '暂无简介。' }}</p>

                    <div class="hero__ops">
                        <button class="op" :class="{ 'is-done': currentSeries.UserData?.Played }" :disabled="playedLoading[currentSeries.Id]" @click="played(currentSeries)">
                            <el-icon :size="16" :class="playedLoading[currentSeries.Id] ? 'is-loading' : ''">
                                <i-ep-CircleCheckFilled v-if="currentSeries.UserData?.Played" />
                                <i-ep-CircleCheck v-else />
                            </el-icon>
                            <span>{{ currentSeries.UserData?.Played ? '已播放' : '标记已播放' }}</span>
                        </button>
                        <button class="op" :class="{ 'is-on': currentSeries.UserData?.IsFavorite }" :disabled="starLoading[currentSeries.Id]" @click="star(currentSeries)">
                            <el-icon :size="16" :class="starLoading[currentSeries.Id] ? 'is-loading' : ''">
                                <i-ep-StarFilled v-if="currentSeries.UserData?.IsFavorite" />
                                <i-ep-Star v-else />
                            </el-icon>
                            <span>{{ currentSeries.UserData?.IsFavorite ? '取消收藏' : '收藏' }}</span>
                        </button>
                        <span class="hero__links">
                            <button v-for="externalUrl in currentSeries.ExternalUrls" :key="externalUrl.Url" class="ext" :title="externalUrl.Url" @click="invokeApi.open_url(externalUrl.Url)">
                                <svg-icon v-if="externalUrl.Url.indexOf('imdb.com') !== -1" name="imdb" size="18" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('themoviedb.org') !== -1" name="tmdb" size="18" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('thetvdb.com') !== -1" name="tvdb" size="18" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('trakt.tv') !== -1" name="trakt" size="18" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('myanimelist.net') !== -1" name="myanimelist" size="18" />
                                <img v-else-if="externalUrl.Url.indexOf('anidb.net') !== -1" src="../../icons/anidb.png" style="width: 18px; height: 18px;" />
                                <el-icon v-else :size="16"><i-ep-Link /></el-icon>
                                <span>{{ externalUrl.Name }}</span>
                            </button>
                        </span>
                    </div>
                </div>
            </div>
        </el-skeleton>

        <div class="roe-section" v-if="seasonsList.length">
            <div class="roe-section-head">
                <h2>季</h2>
                <span class="roe-section-count">{{ seasonsList.length }}</span>
            </div>
            <el-skeleton :loading="seasonsLoading" animated>
                <template #template>
                    <div class="strip">
                        <div v-for="i in 6" :key="i" class="strip__cell">
                            <el-skeleton-item variant="image" style="height: 160px; width: 112px;" />
                        </div>
                    </div>
                </template>
                <div class="strip">
                    <button v-for="season in seasonsList" :key="season.Id" class="strip__cell" @click="showSeasons(season)">
                        <span class="strip__img loe-cover-img">
                            <img v-lazy="useImage().images[embyServerId + ':cover:' + season.Id]" />
                        </span>
                        <span class="strip__caption" :title="season.Name">{{ season.Name }}</span>
                    </button>
                </div>
            </el-skeleton>
        </div>

        <div class="roe-section">
            <div class="roe-section-head">
                <h2>剧集</h2>
                <span class="roe-section-count">{{ episodesTotal }}</span>
            </div>
            <el-skeleton :loading="episodesLoading" animated>
                <template #template>
                    <div class="grid">
                        <el-card v-for="i in 4" :key="i" style="width: 344px; height: 182px; background: transparent;" />
                    </div>
                </template>
                <div v-if="episodesList.length" class="grid">
                    <ItemCard v-for="episodeItem in episodesList" :key="episodeItem.Id" :item="episodeItem" :embyServerId="embyServerId" />
                </div>
                <div v-else class="roe-empty">
                    <span class="roe-empty__line">这里还没有剧集</span>
                    <span>展开上方某一季，可以查看该季的全部单集。</span>
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

        <el-dialog v-model="dialogSeasonsVisible" :title="dialogSeasons?.Name" width="860">
            <div class="season-head">
                <p class="season-head__overview">{{ dialogSeasons?.Overview || '暂无简介。' }}</p>
                <div class="season-head__ops">
                    <button class="op" :class="{ 'is-done': dialogSeasons?.UserData?.Played }" :disabled="playedLoading[dialogSeasons!.Id]" @click="played(dialogSeasons!)">
                        <el-icon :size="16" :class="playedLoading[dialogSeasons!.Id] ? 'is-loading' : ''">
                            <i-ep-CircleCheckFilled v-if="dialogSeasons?.UserData?.Played" />
                            <i-ep-CircleCheck v-else />
                        </el-icon>
                        <span>{{ dialogSeasons?.UserData?.Played ? '已播放' : '标记已播放' }}</span>
                    </button>
                    <button class="op" :class="{ 'is-on': dialogSeasons?.UserData?.IsFavorite }" :disabled="starLoading[dialogSeasons!.Id]" @click="star(dialogSeasons!)">
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
                        <div class="ep-row" v-for="i in 6" :key="i">
                            <el-skeleton-item variant="text" style="width: 45%" />
                        </div>
                    </template>
                    <button v-for="episodeItem in dialogEpisodesList" :key="episodeItem.Id" class="ep-row" @click="gotoEpisodes(episodeItem.Id)">
                        <span class="ep-row__num">{{ episodeItem.IndexNumber }}</span>
                        <span class="ep-row__name">{{ episodeItem.Name }}</span>
                        <span class="ep-row__tag" v-for="value in mediaSourceTag[episodeItem.Id]" :key="value">{{ value }}</span>
                        <span class="ep-row__date">{{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}</span>
                        <span class="ep-row__ops" @click.stop>
                            <el-link :underline="false" v-if="episodeItem.UserData" :disabled="starLoading[episodeItem.Id]" @click="star(episodeItem)">
                                <el-icon :size="16" class="mini-icon mini-icon--star" :class="[episodeItem.UserData.IsFavorite ? 'is-on' : '', starLoading[episodeItem.Id] ? 'is-loading' : '']">
                                    <i-ep-StarFilled v-if="episodeItem.UserData.IsFavorite" />
                                    <i-ep-Star v-else />
                                </el-icon>
                            </el-link>
                            <el-link :underline="false" v-if="episodeItem.UserData" :disabled="playedLoading[episodeItem.Id]" @click="played(episodeItem)">
                                <el-icon :size="16" class="mini-icon mini-icon--done" :class="[episodeItem.UserData.Played ? 'is-on' : '', playedLoading[episodeItem.Id] ? 'is-loading' : '']">
                                    <i-ep-CircleCheckFilled v-if="episodeItem.UserData.Played" />
                                    <i-ep-CircleCheck v-else />
                                </el-icon>
                            </el-link>
                        </span>
                    </button>
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
    </div>
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
            let resolution = getResolutionFromMediaSources(mediaSource)
            mediaSourceTag.value[itemId].push(size + " · " + resolution)
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
    if (!item.UserData) {
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
/* 主视觉：海报 + 简介，左侧对齐，唯一的大字号 */
.hero {
    display: flex;
    align-items: flex-start;
    gap: 28px;
}

.hero__poster {
    flex: none;
    width: 272px;
    height: 400px;
}

.hero__poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.hero__main {
    flex: auto;
    min-width: 0;
    padding-top: 4px;
}

.hero__title {
    margin: 0;
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: 1.2;
    letter-spacing: 0.01em;
}

.hero__facts {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    font-size: var(--text-sm);
    color: var(--text-3);
}

.hero__facts .mono {
    font-family: var(--font-mono);
}

.chip {
    padding: 1px 7px;
    border: 1px solid var(--hairline);
    border-radius: 4px;
    font-size: var(--text-xs);
    color: var(--text-2);
}

.hero__overview {
    max-width: 68ch;
    margin: 16px 0 0;
    color: var(--text-2);
    line-height: 1.75;
}

.hero__ops {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 22px;
}

.op {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
}

.op:hover:not(:disabled) {
    color: var(--text-1);
    border-color: #333C48;
}

.op.is-on {
    color: var(--lamp);
    border-color: var(--lamp-line);
}

.op.is-done {
    color: #6FBF7C;
    border-color: rgba(79, 163, 94, 0.4);
}

.op:disabled {
    opacity: 0.6;
    cursor: default;
}

.hero__links {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: 6px;
}

.ext {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 9px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-3);
    font-family: inherit;
    font-size: var(--text-xs);
    cursor: pointer;
}

.ext:hover {
    color: var(--text-1);
    border-color: #333C48;
}

/* 季：横向条 */
.strip {
    display: flex;
    gap: 14px;
    overflow-x: auto;
    padding-bottom: 6px;
}

.strip__cell {
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 7px;
    width: 112px;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
}

.strip__img {
    display: block;
    width: 112px;
    height: 160px;
}

.strip__img img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.strip__caption {
    font-size: var(--text-sm);
    color: var(--text-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.strip__cell:hover .strip__caption {
    color: var(--text-1);
}

.grid {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
}

/* 单集弹窗 */
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

.ep-row {
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

.ep-row:hover {
    background: #1A2028;
}

.ep-row__num {
    flex: none;
    width: 22px;
    font-family: var(--font-mono);
    color: var(--text-3);
}

.ep-row__name {
    flex: auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.ep-row__tag {
    flex: none;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
    border: 1px solid var(--hairline);
    border-radius: 4px;
    padding: 0 5px;
}

.ep-row__date {
    flex: none;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
}

.ep-row__ops {
    flex: none;
    display: flex;
    align-items: center;
    gap: 6px;
}

.mini-icon {
    color: var(--text-3);
}

.mini-icon--star.is-on {
    color: var(--lamp);
}

.mini-icon--done.is-on {
    color: #6FBF7C;
}
</style>
