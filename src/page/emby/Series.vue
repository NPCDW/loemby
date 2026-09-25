<template>
    <el-scrollbar>
        <el-skeleton :loading="serieInfoLoading" animated>
            <template #template>
                <div class="series-hero">
                    <div class="series-cover-skeleton">
                        <el-skeleton-item variant="image" style="height: 416px; width: 300px;" />
                    </div>
                    <div class="series-info">
                        <h1><el-skeleton-item variant="h1" style="width: 50%; margin-top: 10px;" /></h1>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <!-- 简介占位：与真实渲染一致，只占 4 行（可展开查看全部） -->
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p><el-skeleton-item variant="text" style="width: 100%" /></p>
                        <p>
                            <el-skeleton-item variant="button" style="width: 15%;margin: 5px;margin-left: 0;" />
                            <el-skeleton-item variant="button" style="width: 15%;margin: 5px;margin-left: 0;" />
                        </p>
                    </div>
                </div>
            </template>
            <div class="series-hero" ref="seriesMoreRef" v-if="currentSeries">
                <div style="min-height: 416px; min-width: 300px;" class="loe-cover-img">
                    <img v-lazy="useImage().images[embyServerId + ':cover:' + currentSeries.Id]" style="max-height: 416px; max-width: 300px;" />
                </div>
                <div class="series-info">
                    <h1>{{ currentSeries.Name }}</h1>
                    <p>{{ currentSeries.ProductionYear }}</p>
                    <!-- 简介：默认只占 4 行，超出部分折叠，点击展开查看全部（不用滚动条） -->
                    <div class="series-overview" :class="{ 'is-expanded': overviewExpanded }" v-if="currentSeries.Overview">
                        <p class="series-overview-text">{{ currentSeries.Overview }}</p>
                        <button class="series-overview-toggle" @click="overviewExpanded = !overviewExpanded">
                            <span>{{ overviewExpanded ? '收起' : '展开' }}</span>
                            <el-icon><i-ep-ArrowDown v-if="!overviewExpanded" /><i-ep-ArrowUp v-else /></el-icon>
                        </button>
                    </div>
                    <!-- 外部标签：与剧集详情页展示方式一致 -->
                    <div class="eps-tags-wrap collapsible" :class="{ 'is-expanded': providerIdsExpanded }" v-if="currentSeries.ProviderIds && Object.keys(currentSeries.ProviderIds).length > 0">
                        <el-tag v-for="(value, key) in currentSeries.ProviderIds" class="eps-provider-tag" disable-transitions>
                            <span class="eps-provider-key">{{ key }}</span>
                            <span class="eps-provider-sep">:</span>
                            <span class="eps-provider-value">{{ value }}</span>
                        </el-tag>
                    </div>
                    <!-- 外部链接：复用剧集详情页子项样式 -->
                    <div class="eps-external-list collapsible" :class="{ 'is-expanded': externalUrlsExpanded }" v-if="currentSeries.ExternalUrls && currentSeries.ExternalUrls.length > 0">
                        <el-tooltip v-for="externalUrl in currentSeries.ExternalUrls" :content="externalUrl.Url" placement="bottom" effect="light">
                            <button class="eps-external-btn" @click="invokeApi.open_url(externalUrl.Url)">
                                <svg-icon v-if="externalUrl.Url.indexOf('imdb.com') !== -1" name="imdb" class="eps-external-icon" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('themoviedb.org') !== -1" name="tmdb" class="eps-external-icon" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('thetvdb.com') !== -1" name="tvdb" class="eps-external-icon" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('trakt.tv') !== -1" name="trakt" class="eps-external-icon" />
                                <svg-icon v-else-if="externalUrl.Url.indexOf('myanimelist.net') !== -1" name="myanimelist" class="eps-external-icon" />
                                <img v-else-if="externalUrl.Url.indexOf('anidb.net') !== -1" src="../../icons/anidb.png" class="eps-external-icon" />
                                <i-ep-Link v-else class="eps-external-icon" />
                                <span class="eps-external-name">{{ externalUrl.Name }}</span>
                            </button>
                        </el-tooltip>
                    </div>
                    <!-- 折叠展开：外部标签 / 外部链接默认各占一行，超出部分点按钮查看全部 -->
                    <div class="series-more">
                        <button class="series-overview-toggle" v-if="hasMoreProviderIds" @click="providerIdsExpanded = !providerIdsExpanded">
                            <span>{{ providerIdsExpanded ? '收起外部标签' : '展开全部外部标签' }}</span>
                            <el-icon><i-ep-ArrowDown v-if="!providerIdsExpanded" /><i-ep-ArrowUp v-else /></el-icon>
                        </button>
                        <button class="series-overview-toggle" v-if="hasMoreExternalUrls" @click="externalUrlsExpanded = !externalUrlsExpanded">
                            <span>{{ externalUrlsExpanded ? '收起外部链接' : '展开全部外部链接' }}</span>
                            <el-icon><i-ep-ArrowDown v-if="!externalUrlsExpanded" /><i-ep-ArrowUp v-else /></el-icon>
                        </button>
                    </div>
                    <!-- 已播放 / 收藏：复用剧集详情页的次要按钮样式 -->
                    <el-button class="eps-secondary-btn" :disabled="playedLoading[currentSeries.Id]" @click="played(currentSeries)">
                        <el-icon color="#67C23A" :size="18" :class="playedLoading[currentSeries.Id] ? 'is-loading' : ''" v-if="currentSeries.UserData?.Played"><i-ep-CircleCheckFilled /></el-icon>
                        <el-icon :size="18" :class="playedLoading[currentSeries.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                        <span>已播放</span>
                    </el-button>
                    <el-button class="eps-secondary-btn" :disabled="starLoading[currentSeries.Id]" @click="star(currentSeries)">
                        <template v-if="currentSeries.UserData?.IsFavorite">
                            <el-icon color="#E6A23C" :size="18" :class="starLoading[currentSeries.Id] ? 'is-loading' : ''"><i-ep-StarFilled /></el-icon>
                            <span>取消收藏</span>
                        </template>
                        <template v-else>
                            <el-icon :size="18" :class="starLoading[currentSeries.Id] ? 'is-loading' : ''"><i-ep-Star /></el-icon>
                            <span>收藏</span>
                        </template>
                    </el-button>
                </div>
            </div>
        </el-skeleton>
        <el-skeleton :loading="episodesLoading" animated>
            <template #template>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;padding: 20px;padding-top: 0;">
                    <el-card class="item-card-skeleton" v-for="i in 5" :key="i">
                        <el-skeleton-item variant="text" style="width: 85%; height: 20px;" />
                        <div style="margin: 10px 0;"><el-skeleton-item variant="text" style="width: 60%; height: 16px;" /></div>
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <el-skeleton-item variant="circle" style="width: 24px; height: 24px;" />
                            <el-skeleton-item variant="button" style="width: 70px; height: 32px;" />
                        </div>
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
                        <div class="season-cover-skeleton">
                            <el-skeleton-item variant="image" style="height: 160px; width: 115px;" />
                        </div>
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
import { computed, onMounted, onUnmounted, ref } from 'vue';
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

/* 简介 / 外部标签 / 外部链接的折叠展开状态。
 * 三者默认各占固定行数（简介 4 行、外部标签与外部链接各 1 行），
 * 超出部分由「展开」按钮查看，不再使用滚动条。 */
const overviewExpanded = ref(false)
const providerIdsExpanded = ref(false)
const externalUrlsExpanded = ref(false)

/* 外部标签 / 外部链接的展开按钮只在「确实超出默认行数」时出现：
 * 标签按 28px 高 + 8px 间距估算总宽，链接按 92px 宽 + 12px 间距估算，
 * 与实际容器可用宽度（信息区宽度 - 左右 padding）比较。 */
const seriesMoreWidth = ref(960)
function estimateTagsWidth(count: number) {
    return count * (110 + 8)
}
function estimateExternalWidth(count: number) {
    return count * (92 + 12)
}
const hasMoreProviderIds = computed(() => {
    const ids = currentSeries.value?.ProviderIds
    if (!ids) {
        return false
    }
    return estimateTagsWidth(Object.keys(ids).length) > seriesMoreWidth.value
})
const hasMoreExternalUrls = computed(() => {
    const urls = currentSeries.value?.ExternalUrls
    if (!urls) {
        return false
    }
    return estimateExternalWidth(urls.length) > seriesMoreWidth.value
})

/* 用 ResizeObserver 跟踪容器宽度，避免写死阈值在多分辨率下判断错误 */
const seriesMoreRef = ref<HTMLElement>()
let seriesMoreObserver: ResizeObserver | undefined
onMounted(() => {
    if (!seriesMoreRef.value) {
        return
    }
    seriesMoreObserver = new ResizeObserver(entries => {
        const width = entries[0]?.contentRect.width
        if (width) {
            seriesMoreWidth.value = Math.max(width - 136, 0)
        }
    })
    seriesMoreObserver.observe(seriesMoreRef.value)
})
onUnmounted(() => {
    seriesMoreObserver?.disconnect()
    seriesMoreObserver = undefined
})
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

/* 信息区容器：真实渲染与骨架共用同一类，左右 padding 对齐剧集详情页
 * （.episodes-page 的 32px），避免加载完成时横向跳动。
 * 骨架行与外层 .el-skeleton 相等，这里不额外写 min-height。 */
.series-hero {
    display: flex;
    padding: 20px 32px;
}

/* 剧集信息封面骨架：对齐真实 .loe-cover-img 的 8px 圆角与 300x416 尺寸 */
.series-cover-skeleton {
    border-radius: 8px;
    overflow: hidden;
    min-height: 416px;
    min-width: 300px;
    display: flex;
    justify-content: center;
    align-items: center;
}

.series-cover-skeleton :deep(.el-skeleton__item) {
    border-radius: 8px;
}

/* 剧季封面骨架：对齐真实 .loe-cover-img 的 8px 圆角与 115x160 尺寸 */
.season-cover-skeleton {
    border-radius: 8px;
    overflow: hidden;
    min-height: 160px;
    min-width: 115px;
    display: flex;
    justify-content: center;
    align-items: center;
}

.season-cover-skeleton :deep(.el-skeleton__item) {
    border-radius: 8px;
}

/* 剧集卡片骨架：尺寸与配色已由 style.css 的 .item-card-skeleton 统一提供 */

/* 骨架信息区：复用真实 .series-info 的内边距（20px），
 * 纵向间距与真实渲染的「标题 / 年份 / 简介 / 标签 / 链接 / 按钮」逐行对齐。 */
.series-info .el-skeleton__item {
    border-radius: 6px;
}

.series-info h1,
.series-info p {
    margin: 0;
}

.series-info h1 {
    margin-bottom: 10px;
}

.series-info p {
    margin-bottom: 8px;
}

/* ============================================================
 * 信息区：外部链接 / 外部标签 / 已播放收藏 复用剧集详情页（Episodes.vue）的样式。
 * 两页的 scoped 样式互相隔离，这里复制同名类，改一处时记得同步另一处。
 * ============================================================ */
.series-info {
    padding: 20px 24px;
    min-width: 0;
}

.series-info > *:last-child {
    margin-bottom: 0;
}

/* ===== 简介：默认 4 行，超出折叠，点「展开」查看全部（不用滚动条） ===== */
.series-overview {
    margin: 4px 0 14px;
}

.series-overview-text {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    /* 默认只占 4 行 */
    -webkit-line-clamp: 4;
    line-clamp: 4;
    overflow: hidden;
    margin: 0;
    line-height: 1.6;
    color: var(--el-text-color-regular, #cfd3dc);
    word-break: break-word;
}

.series-overview.is-expanded .series-overview-text {
    -webkit-line-clamp: unset;
    line-clamp: unset;
    overflow: visible;
}

/* ===== 展开 / 收起按钮（简介、外部标签、外部链接共用） ===== */
.series-overview-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 6px;
    padding: 0;
    border: none;
    background: transparent;
    font-size: 12px;
    line-height: 1;
    color: var(--el-color-primary, #409eff);
    cursor: pointer;
    transition: color 0.2s ease;
}

.series-overview-toggle:hover {
    color: var(--el-color-primary-light-3, #79bbff);
}

.series-more {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 16px;
    margin: 2px 0 16px;
}

/* ===== 外部标签 / 外部链接默认各只占一行，点「展开」查看全部 ===== */
.eps-tags-wrap.collapsible,
.eps-external-list.collapsible {
    max-height: 28px;
    overflow: hidden;
}

.eps-external-list.collapsible {
    max-height: 92px;
}

.eps-tags-wrap.collapsible.is-expanded,
.eps-external-list.collapsible.is-expanded {
    max-height: none;
    overflow: visible;
}

/* 外部标签：同 .eps-provider-tag，key 大写、冒号后留一个空格 */
.eps-tags-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.eps-provider-tag {
    display: inline-flex;
    align-items: center;
    height: 28px;
    padding: 0 10px;
    background-color: var(--el-fill-color-light, #262727);
    border-color: var(--el-border-color-extra-light, #2b2b2c);
}

.eps-provider-key {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--el-text-color-secondary, #909399);
}

.eps-provider-sep {
    font-size: 11px;
    color: var(--el-text-color-secondary, #909399);
}

.eps-provider-sep + .eps-provider-value {
    margin-left: 4px;
}

.eps-provider-value {
    font-size: 12px;
    color: var(--el-text-color-regular, #cfd3dc);
}

/* 外部链接子项：同 .eps-external-btn，图标在上、名称在下 */
.eps-external-list {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin: 4px 0 12px;
}

.eps-external-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 92px;
    height: 92px;
    border-radius: 10px;
    cursor: pointer;
    background-color: var(--el-fill-color-light, #262727);
    border: 1px solid var(--el-border-color-extra-light, #2b2b2c);
    transition: all 0.2s ease;
}

.eps-external-btn:hover {
    border-color: var(--el-color-primary-light-5, #3375b9);
    transform: translateY(-2px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.25);
}

.eps-external-icon {
    width: 42px;
    height: 42px;
    color: var(--el-text-color-regular, #cfd3dc);
}

.eps-external-name {
    font-size: 11px;
    color: var(--el-text-color-secondary, #909399);
}

/* 已播放 / 收藏按钮：同 .eps-secondary-btn */
.eps-secondary-btn {
    background-color: transparent;
    border-color: var(--el-border-color-extra-light, #2b2b2c);
    color: var(--el-text-color-regular, #cfd3dc);
    transition: all 0.2s ease;
}

.eps-secondary-btn:hover {
    border-color: var(--el-color-primary-light-5, #3375b9);
    color: var(--el-color-primary, #409eff);
    background-color: var(--el-color-primary-light-9, #18222c);
}
</style>
/* ===== 折叠展开状态下的间距收尾 ===== */
/* 外部标签 / 外部链接收起时同样保留行下间距，避免展开按钮紧贴内容 */
.eps-tags-wrap.collapsible {
    margin-bottom: 12px;
}

.eps-external-list.collapsible {
    margin-bottom: 12px;
}
