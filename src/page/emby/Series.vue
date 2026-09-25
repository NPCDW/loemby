<template>
    <el-scrollbar>
        <el-skeleton :loading="serieInfoLoading" animated>
            <template #template>
                <div style="display: flex; padding: 20px;">
                    <div class="series-cover-skeleton">
                        <el-skeleton-item variant="image" style="height: 416px; width: 300px;" />
                    </div>
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
                <div class="series-info">
                    <h1>{{ currentSeries.Name }}</h1>
                    <p>{{ currentSeries.ProductionYear }}</p>
                    <!-- 剧集简介：默认只占 4 行，悬停以浮层卡片展示完整内容（浮层绝对定位，不占流、不移动任何元素） -->
                    <div class="hover-card-host" v-if="currentSeries.Overview">
                        <p class="series-overview is-clamp">{{ currentSeries.Overview }}</p>
                        <div class="hover-card hover-card--overview">
                            <div class="hover-card__title">剧集简介</div>
                            <div class="hover-card__body">{{ currentSeries.Overview }}</div>
                        </div>
                    </div>
                    <!-- 外部标签：默认只占一行，悬停以浮层卡片展示全部 -->
                    <div class="hover-card-host" v-if="currentSeries.ProviderIds && Object.keys(currentSeries.ProviderIds).length > 0">
                        <div ref="tagsWrapRef" class="eps-tags-wrap is-single-line">
                            <el-tag v-for="(value, key) in currentSeries.ProviderIds" :key="key" class="eps-provider-tag" disable-transitions>
                                <span class="eps-provider-key">{{ key }}</span>
                                <span class="eps-provider-sep">:</span>
                                <span class="eps-provider-value">{{ value }}</span>
                            </el-tag>
                            <span class="ellipsis-tail" v-if="tagsOverflow">···</span>
                        </div>
                        <div class="hover-card hover-card--tags">
                            <div class="hover-card__title">外部标签</div>
                            <div class="hover-card__body eps-tags-wrap">
                                <el-tag v-for="(value, key) in currentSeries.ProviderIds" :key="key" class="eps-provider-tag" disable-transitions>
                                    <span class="eps-provider-key">{{ key }}</span>
                                    <span class="eps-provider-sep">:</span>
                                    <span class="eps-provider-value">{{ value }}</span>
                                </el-tag>
                            </div>
                        </div>
                    </div>
                    <!-- 外部链接：默认只占一行，悬停以浮层卡片展示全部 -->
                    <div class="hover-card-host" v-if="currentSeries.ExternalUrls && currentSeries.ExternalUrls.length > 0">
                        <div ref="linksWrapRef" class="eps-external-list is-single-line">
                            <el-tooltip v-for="externalUrl in currentSeries.ExternalUrls" :key="externalUrl.Url" :content="externalUrl.Url" placement="bottom" effect="light">
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
                            <div class="ellipsis-tail" v-if="linksOverflow">···</div>
                        </div>
                        <div class="hover-card hover-card--links">
                            <div class="hover-card__title">外部链接</div>
                            <div class="hover-card__body eps-external-list">
                                <el-tooltip v-for="externalUrl in currentSeries.ExternalUrls" :key="externalUrl.Url" :content="externalUrl.Url" placement="bottom" effect="light">
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
                        </div>
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
import { nextTick, onBeforeUnmount, ref, watch } from 'vue';
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

/* ============================================================
 * 简介 / 外部标签 / 外部链接的折叠与悬停浮层
 *
 * 折叠：简介 4 行，标签 / 链接各一行；浮层一律用绝对定位的
 * `.hover-card`，脱离文档流，悬停时只显示浮层，触发元素本身
 * 与其它元素的位置都不会发生变化。
 * 这里的 JS 只负责「是否溢出」这一个判断，用来决定折叠行末尾
 * 的「···」要不要出现；浮层显隐完全交给 CSS 的 :hover，不依赖
 * 任何测量结果，因此不会出现「溢出没测到 → 悬停失效」的问题。
 * ============================================================ */
const tagsWrapRef = ref<HTMLElement>()
const linksWrapRef = ref<HTMLElement>()
const tagsOverflow = ref(false)
const linksOverflow = ref(false)

/** 单行不换行时，用「内容总宽 > 可见宽」判断是否被裁掉 */
function measureLine(el?: HTMLElement) {
    if (!el) {
        return false
    }
    // scrollWidth 在 flex + overflow:hidden 下可能等于 clientWidth，
    // 这里同时用最后可见子项的实际右边界兜底，避免漏判。
    if (el.scrollWidth - el.clientWidth > 1) {
        return true
    }
    const box = el.getBoundingClientRect()
    const children = Array.from(el.children).filter((c) => !c.classList.contains('ellipsis-tail'))
    const last = children[children.length - 1] as HTMLElement | undefined
    if (!last) {
        return false
    }
    const lastRight = last.getBoundingClientRect().right
    // 右侧留了 30px 的省略号空隙，超出即视为溢出
    return lastRight - (box.right - 30) > 1
}

function measureAll() {
    tagsOverflow.value = measureLine(tagsWrapRef.value)
    linksOverflow.value = measureLine(linksWrapRef.value)
}

let infoResizeObserver: ResizeObserver | undefined
watch(currentSeries, () => {
    nextTick(() => {
        measureAll()
        // 信息区宽度随窗口变化，宽度变了要重新判断是否溢出
        if (!infoResizeObserver) {
            infoResizeObserver = new ResizeObserver(() => measureAll())
        }
        infoResizeObserver.disconnect()
        if (tagsWrapRef.value) {
            infoResizeObserver.observe(tagsWrapRef.value)
        }
        if (linksWrapRef.value) {
            infoResizeObserver.observe(linksWrapRef.value)
        }
        // 链接图标是异步图片，加载完成后宽度会变化，稍后再补测一次
        window.setTimeout(measureAll, 300)
    })
}, { immediate: true })

onBeforeUnmount(() => {
    infoResizeObserver?.disconnect()
    infoResizeObserver = undefined
})

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

/* ============================================================
 * 信息区：外部链接 / 外部标签 / 已播放收藏 复用剧集详情页（Episodes.vue）的样式。
 * 两页的 scoped 样式互相隔离，这里复制同名类，改一处时记得同步另一处。
 * ============================================================ */
.series-info {
    padding: 20px;
    min-width: 0;
}

/* 剧集简介：默认折叠为 4 行，超出部分以省略号结尾，悬停依赖外层 popover 展示全文 */
.series-overview {
    margin: 12px 0;
    line-height: 1.6;
    color: var(--el-text-color-regular, #cfd3dc);
    font-size: 14px;
}

/* 默认折叠为 4 行；悬停由外层 .hover-card 展示全文 */
.series-overview.is-clamp {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 4;
    line-clamp: 4;
    overflow: hidden;
    /* 行高 1.6 时 4 行约为 6.4em，加一点余量做兜底裁剪 */
    max-height: 6.6em;
}

/* 折叠态的行内容器：单行不换行 + 溢出裁掉，右端留出省略号的空隙 */
.eps-tags-wrap.is-single-line,
.eps-external-list.is-single-line {
    flex-wrap: nowrap;
    overflow: hidden;
    padding-right: 30px;
}

/* 折叠行是浮层的悬停热区，高度固定为一行，避免影响其它元素 */
.eps-tags-wrap.is-single-line {
    height: 28px;
    align-items: center;
}

.eps-external-list.is-single-line {
    height: 92px;
    align-items: center;
}

/* 收起时结尾的省略号，绝对定位在折叠行右端，不参与布局 */
.ellipsis-tail {
    position: absolute;
    top: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    height: 100%;
    color: var(--el-text-color-secondary, #909399);
    font-weight: 600;
    letter-spacing: 1px;
    padding-left: 24px;
    background: linear-gradient(90deg, transparent, var(--el-bg-color, #141414) 60%);
    pointer-events: none;
}

/* 折叠行作为省略号的定位容器 */
.eps-tags-wrap.is-single-line,
.eps-external-list.is-single-line {
    position: relative;
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
    margin: 16px 0;
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

/* ============================================================
 * 悬停浮层卡片：绝对定位，脱离文档流。
 * 触发元素折叠态照常占位，悬停只是在其旁边多出一个浮层，
 * 因此触发元素与其它元素的位置都不会移动。
 * ============================================================ */
.hover-card-host {
    position: relative;
}

/* 折叠行本身作为「悬停热区」，浮层从它的右侧展开 */
.hover-card {
    position: absolute;
    left: calc(100% + 12px);
    top: 0;
    z-index: 2000;
    display: none;
    box-sizing: border-box;
    padding: 12px 14px;
    border-radius: 8px;
    background-color: var(--el-bg-color-overlay, #1d1e1f);
    border: 1px solid var(--el-border-color-lighter, #363637);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    pointer-events: auto;
}

/* 悬停触发元素或浮层自身时保持显示，便于在浮层内滚动 / 点击 */
.hover-card-host:hover > .hover-card,
.hover-card-host:focus-within > .hover-card,
.hover-card:hover {
    display: block;
}

/* 简介浮层：正文较长，给一个舒适的阅读宽度并限制最大高度 */
.hover-card--overview {
    width: min(520px, 42vw);
}

.hover-card--tags {
    width: min(420px, 36vw);
}

.hover-card--links {
    width: min(560px, 46vw);
}

.hover-card__title {
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--el-text-color-secondary, #909399);
}

.hover-card__body {
    max-height: 50vh;
    overflow-y: auto;
    font-size: 13px;
    line-height: 1.7;
    color: var(--el-text-color-regular, #cfd3dc);
    white-space: pre-line;
    overflow-wrap: anywhere;
}

/* 浮层里的标签 / 链接恢复多行排布 */
.hover-card__body.eps-tags-wrap,
.hover-card__body.eps-external-list {
    margin: 0;
    flex-wrap: wrap;
    overflow: visible;
    padding-right: 0;
    white-space: normal;
}
</style>
