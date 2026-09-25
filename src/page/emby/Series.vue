<template>
    <el-scrollbar>
        <el-skeleton :loading="serieInfoLoading" animated>
            <template #template>
                <div style="display: flex; padding: 20px;">
                    <div class="series-cover-skeleton">
                        <el-skeleton-item variant="image" style="height: 416px; width: 300px;" />
                    </div>
                    <div class="series-info">
                        <h1><el-skeleton-item variant="h1" style="width: 50%; margin-top: 10px;" /></h1>
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                        <!-- 简介：与真实渲染一致，占 4 行折叠高度 -->
                        <div class="skeleton-overview">
                            <p v-for="i in 4" :key="i"><el-skeleton-item variant="text" style="width: 100%" /></p>
                        </div>
                        <!-- 外部标签：与真实渲染一致，占一行 -->
                        <div class="skeleton-tags">
                            <el-skeleton-item variant="button" style="width: 120px; height: 28px;" />
                            <el-skeleton-item variant="button" style="width: 96px; height: 28px;" />
                            <el-skeleton-item variant="button" style="width: 132px; height: 28px;" />
                        </div>
                        <!-- 外部链接：与真实渲染一致，占一行 92px 子项 -->
                        <div class="skeleton-links">
                            <el-skeleton-item v-for="i in 5" :key="i" variant="image" style="width: 92px; height: 92px; border-radius: 10px;" />
                        </div>
                        <p>
                            <el-skeleton-item variant="button" style="width: 120px; height: 32px; margin: 5px 12px 5px 0;" />
                            <el-skeleton-item variant="button" style="width: 120px; height: 32px; margin: 5px 0;" />
                        </p>
                    </div>
                </div>
            </template>
            <div style="display: flex; padding: 20px;" v-if="currentSeries">
                <div style="min-height: 416px; min-width: 300px;" class="loe-cover-img">
                    <img v-lazy="useImage().images[embyServerId + ':cover:' + currentSeries.Id]" style="max-height: 416px; max-width: 300px;" />
                </div>
                <div class="series-info" :ref="bindInfoRef">
                    <h1>{{ currentSeries.Name }}</h1>
                    <p>{{ currentSeries.ProductionYear }}</p>
                    <!-- 剧集简介：不悬停时是一张与页面融合的卡片（默认 4 行，超出以省略号结尾），
                         悬停时卡片原地「凸出」到页面最上层，高度过渡到全部内容并带阴影 -->
                    <div class="hover-card-host hover-card-host--overview" v-if="currentSeries.Overview">
                        <div class="hover-card series-overview-card" :class="{ 'is-expanded': overviewExpanded }">
                            <p class="series-overview is-clamp">
                                <span ref="keepRef" class="series-overview__keep">{{ overviewKeep }}</span><span ref="moreRef" class="series-overview__more">{{ overviewMore }}</span><span class="ellipsis-mark" v-if="overviewOverflow">…</span>
                            </p>
                        </div>
                    </div>
                    <!-- 外部标签：不悬停时只占一行（超出以「···」结尾），悬停时卡片凸出展示全部标签 -->
                    <div class="hover-card-host hover-card-host--tags" v-if="currentSeries.ProviderIds && Object.keys(currentSeries.ProviderIds).length > 0">
                        <div ref="tagsWrapRef" class="hover-card eps-tags-wrap" :class="{ 'is-expanded': tagsExpanded }">
                            <el-tag v-for="(value, key) in currentSeries.ProviderIds" :key="key" class="eps-provider-tag" disable-transitions>
                                <span class="eps-provider-key">{{ key }}</span>
                                <span class="eps-provider-sep">:</span>
                                <span class="eps-provider-value">{{ value }}</span>
                            </el-tag>
                            <span class="ellipsis-tail" v-if="tagsOverflow">···</span>
                        </div>
                    </div>
                    <!-- 外部链接：不悬停时只占一行（超出以「···」结尾），悬停时卡片凸出展示全部链接 -->
                    <div class="hover-card-host hover-card-host--links" v-if="currentSeries.ExternalUrls && currentSeries.ExternalUrls.length > 0">
                        <div ref="linksWrapRef" class="hover-card eps-external-list" :class="{ 'is-expanded': linksExpanded }">
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
                            <span class="ellipsis-tail" v-if="linksOverflow">···</span>
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
import { onBeforeUnmount, ref, watch } from 'vue';
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
 * 简介 / 外部标签 / 外部链接：融合卡片 + 悬停凸出
 *
 * 三者各自是一张「卡片」，收起时卡片背景与页面融合（与四周
 * 看不出边界），悬停时同一张卡片原地凸出：高度过渡到全部内容、
 * 提升 z-index、补上背景与阴影，看起来像卡片浮到了页面上方。
 *
 * 卡片是文档流内的元素，收起时按折叠高度占位，所以悬停过程中
 * 卡片自身与其它元素的位置都不会移动，只是「长高」了一层。
 *
 * 是否溢出由 JS 判定：
 *  - 溢出时才在折叠行末尾补省略号（简介「…」/ 标签链接「···」）；
 *  - 未溢出时不加任何强调样式，悬停保持原样（不会有多余的凸出）。
 * ============================================================ */
const tagsWrapRef = ref<HTMLElement>()
const linksWrapRef = ref<HTMLElement>()
const overviewOverflow = ref(false)
const tagsOverflow = ref(false)
const linksOverflow = ref(false)
const overviewExpanded = ref(false)
const tagsExpanded = ref(false)
const linksExpanded = ref(false)

/** 折叠行末尾省略号的占位宽度：ellipsis-tail 的 padding-left + 一点余量 */
const TAIL_WIDTH = 30

const overviewText = ref('')
/** 折叠时能完整放下的文本，超出部分放进 .series-overview__more 并在悬停时过渡展开 */
const overviewKeep = ref('')
const overviewMore = ref('')
const keepRef = ref<HTMLElement>()
const moreRef = ref<HTMLElement>()

function resetExpanded() {
    overviewExpanded.value = false
    tagsExpanded.value = false
    linksExpanded.value = false
}

/**
 * 简介：在折叠卡片里逐字二分，找出 4 行内能完整放下的最长前缀，
 * 把正文切成「折叠可见」与「悬停后展开」两段，长度之差即高度过渡的距离。
 *
 * 二分期间直接改写已经渲染出来的两个 span（而不是先写 ref 再等 Vue 渲染），
 * 这样每次试探都能立刻读到布局结果；循环结束后才把最终值写回 ref。
 */
function measureOverviewKeep() {
    const text = overviewText.value
    const keepEl = keepRef.value
    const moreEl = moreRef.value
    if (!text || !keepEl || !moreEl) {
        overviewKeep.value = text
        overviewMore.value = ''
        return
    }
    const card = keepEl.closest('.hover-card') as HTMLElement | null
    if (!card) {
        overviewKeep.value = text
        overviewMore.value = ''
        return
    }
    let low = 0
    let high = text.length
    // 试探 n 个字符时，卡片高度是否还在折叠高度内
    const fits = (n: number) => {
        keepEl.textContent = text.slice(0, n)
        moreEl.textContent = n < text.length ? '…' : ''
        return card.scrollHeight <= card.clientHeight + 1
    }
    if (fits(text.length)) {
        // 未溢出：整段都在折叠行数内，没有需要展开的部分
        keepEl.textContent = text
        moreEl.textContent = ''
        overviewKeep.value = text
        overviewMore.value = ''
        return
    }
    while (low < high) {
        const mid = Math.ceil((low + high) / 2)
        if (fits(mid)) {
            low = mid
        } else {
            high = mid - 1
        }
    }
    // 不要从单词 / 数字中间截断（中英文混排同样适用）
    const WORD = /[0-9A-Za-z]/
    while (low > 0 && low < text.length && WORD.test(text[low - 1]) && WORD.test(text[low])) {
        low--
    }
    const keep = Math.max(low, 1)
    keepEl.textContent = text.slice(0, keep)
    moreEl.textContent = text.slice(keep)
    overviewKeep.value = text.slice(0, keep)
    overviewMore.value = text.slice(keep)
}

/** 单行不换行时，用「内容总宽 > 可见宽」判断是否被裁掉 */
function isLineOverflow(el: HTMLElement) {
    // 最后一个子项的右边界一旦越过「卡片右边 - 省略号宽度」，就说明有内容要藏在「···」后面
    const box = el.getBoundingClientRect()
    const children = Array.from(el.children).filter(
        (c) => !c.classList.contains('ellipsis-tail') && c.getBoundingClientRect().width > 0,
    )
    const last = children[children.length - 1] as HTMLElement | undefined
    if (!last) {
        return false
    }
    return last.getBoundingClientRect().right - (box.right - TAIL_WIDTH) > 1
}

function measureOverflow() {
    resetExpanded()
    // 简介：先按当前宽度算出「折叠可见前缀」，前缀之外还有内容即为溢出
    measureOverviewKeep()
    overviewOverflow.value = overviewMore.value.length > 0
    // 标签 / 链接：收起态卡片被 overflow 裁掉的部分即溢出，再补一次右边界兜底
    tagsOverflow.value = !!tagsWrapRef.value && (tagsWrapRef.value.scrollWidth - tagsWrapRef.value.clientWidth > 1 || isLineOverflow(tagsWrapRef.value))
    linksOverflow.value = !!linksWrapRef.value && (linksWrapRef.value.scrollWidth - linksWrapRef.value.clientWidth > 1 || isLineOverflow(linksWrapRef.value))
}

/** 重测可能改变前缀长度，用 rAF 串联，避免在同一帧里反复触发布局 */
let measureScheduled = false
function scheduleMeasure() {
    if (measureScheduled) {
        return
    }
    measureScheduled = true
    requestAnimationFrame(() => {
        measureScheduled = false
        measureOverflow()
    })
}

const infoRef = ref<HTMLElement>()
let infoResizeObserver: ResizeObserver | undefined

watch(currentSeries, (series) => {
    resetExpanded()
    overviewText.value = series?.Overview || ''
    scheduleMeasure()
    if (!infoResizeObserver) {
        infoResizeObserver = new ResizeObserver(() => scheduleMeasure())
    }
    infoResizeObserver.disconnect()
    if (infoRef.value) {
        // 信息区宽度一变，折叠行能放下的字数就变了，需要重测
        infoResizeObserver.observe(infoRef.value)
    }
    // 链接图标是异步加载的图片，加载完成后宽度会变化，稍后再补测一次
    window.setTimeout(scheduleMeasure, 300)
    window.setTimeout(scheduleMeasure, 900)
}, { immediate: true })

function bindInfoRef(el: any) {
    infoRef.value = el as HTMLElement | undefined
    // 首次挂载时 ref 才可用，补一次测量（watch 的 immediate 那次拿不到元素）
    if (el) {
        scheduleMeasure()
    }
}

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
 * 剧集信息骨架：与真实渲染的折叠态同结构同高度，
 * 加载完成时只换内容、不产生纵向位移。
 * ============================================================ */
.skeleton-overview {
    /* 与 .hover-card-host--overview 的折叠高度一致 */
    height: calc(6.4em + 20px);
    overflow: hidden;
    color: var(--el-text-color-regular, #cfd3dc);
    font-size: 14px;
    padding: 10px 12px;
}

.skeleton-overview p {
    margin: 0;
    line-height: 1.6;
}

/* 外部标签 / 外部链接骨架：高度与真实折叠卡片一致（44px / 108px） */
.skeleton-tags {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 8px;
    margin: 4px 0;
    overflow: hidden;
}

.skeleton-links {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    height: 108px;
    padding: 8px;
    margin: 8px 0;
    overflow: hidden;
}

/* ============================================================
 * 信息区：外部链接 / 外部标签 / 已播放收藏 复用剧集详情页（Episodes.vue）的样式。
 * 两页的 scoped 样式互相隔离，这里复制同名类，改一处时记得同步另一处。
 * ============================================================ */
.series-info {
    padding: 20px;
    min-width: 0;
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
 * 融合卡片 + 悬停凸出
 *
 * 收起：卡片是一张与页面融合的普通卡片（无背景 / 边框 / 阴影），
 *       只按折叠高度占位（简介 4 行、标签 / 链接各一行）。
 * 悬停：同一张卡片被「抽」出文档流（host 用 padding-top 占住原来的
 *       位置），高度过渡到全部内容，同时补上背景、边框与阴影并提到
 *       最上层 —— 看起来就像卡片原地凸了出来。
 * 因为 host 的占位高度始终等于收起态的卡片高度，展开 / 收起过程中
 *       卡片自身与其它元素都不会发生位移。
 * ============================================================ */
.hover-card-host {
    position: relative;
    display: block;
    margin: 0;
    /* 高度固定为折叠态高度，卡片展开时抽出文档流也不会带动下方元素 */
    box-sizing: content-box;
}

.hover-card {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    box-sizing: border-box;
    border-radius: 8px;
    border: 1px solid transparent;
    background-color: transparent;
    box-shadow: 0 0 0 rgba(0, 0, 0, 0);
    /* 高度从固定的折叠高度过渡到 auto（浏览器会解析成展开后的实际高度） */
    transition:
        height 0.28s cubic-bezier(0.4, 0, 0.2, 1),
        background-color 0.22s ease,
        border-color 0.22s ease,
        box-shadow 0.24s ease,
        padding 0.22s ease;
}

/* 展开后允许滚动 / 显示阴影，收起时由内容裁切出折叠行 */
.hover-card-host:hover > .hover-card,
.hover-card-host .hover-card.is-expanded {
    z-index: 60;
    /* 展开后至少和占位一样高，保证卡片不会比收起态更矮 */
    min-height: var(--collapsed-height, 0);
    overflow-y: auto;
    max-height: 60vh;
    background-color: var(--el-bg-color-overlay, #1d1e1f);
    border-color: var(--el-border-color-lighter, #363637);
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.55);
}

/* ---------------- 简介卡片 ---------------- */
/* 折叠高度 = 4 行 × 行高 1.6 × 字号 14px + 上下 padding。
   占位高度与卡片高度共用同一个变量，展开 / 收起时后续元素零位移。 */
.hover-card-host--overview {
    --collapsed-height: calc(6.4em + 20px);
    height: var(--collapsed-height);
}

.hover-card.series-overview-card {
    height: var(--collapsed-height, calc(6.4em + 20px));
    overflow: hidden;
    padding: 10px 12px;
    color: var(--el-text-color-regular, #cfd3dc);
    font-size: 14px;
}

.hover-card-host--overview:hover > .hover-card.series-overview-card,
.hover-card-host--overview .hover-card.series-overview-card.is-expanded {
    height: auto;
    border-radius: 10px;
}

/* 简介正文：展开动画做在卡片高度上，这里按内容自然高度排版 */
.series-overview {
    margin: 0;
    line-height: 1.6;
    font-size: inherit;
    color: inherit;
    white-space: pre-line;
    overflow-wrap: anywhere;
}

/* ---------------- 外部标签卡片 ---------------- */
/* 折叠高度 = 28px 标签 + 上下 padding；占位高度与卡片高度一致 */
.hover-card-host--tags {
    --collapsed-height: 44px;
    height: var(--collapsed-height);
    margin: 4px 0;
}

.hover-card.eps-tags-wrap {
    height: var(--collapsed-height, 44px);
    padding: 8px;
}

/* ---------------- 外部链接卡片 ---------------- */
/* 折叠高度 = 92px 子项 + 上下 padding；占位高度与卡片高度一致 */
.hover-card-host--links {
    --collapsed-height: 108px;
    height: var(--collapsed-height);
    margin: 8px 0;
}

.hover-card.eps-external-list {
    height: var(--collapsed-height, 108px);
    padding: 8px;
}

/* 展开后恢复多行排布 */
.hover-card.eps-tags-wrap.is-expanded,
.hover-card-host--tags:hover > .hover-card.eps-tags-wrap,
.hover-card.eps-external-list.is-expanded,
.hover-card-host--links:hover > .hover-card.eps-external-list {
    height: auto;
    flex-wrap: wrap;
    border-radius: 10px;
}

/* 收起时结尾的「···」，绝对定位在折叠行右端，不参与布局 */
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
    z-index: 1;
}

/* 展开态下省略号不再显示（卡片已展示全部内容） */
.hover-card-host:hover > .hover-card .ellipsis-tail,
.hover-card.is-expanded .ellipsis-tail,
.hover-card-host:hover > .hover-card .ellipsis-mark,
.hover-card.is-expanded .ellipsis-mark {
    display: none;
}

/* 折叠行是省略号的定位容器 */
.eps-tags-wrap,
.eps-external-list {
    position: relative;
}

/* ---------------- 折叠态的行内容器 ---------------- */
/* 标签：单行不换行，超出部分裁掉并让出省略号的空隙 */
.eps-tags-wrap {
    box-sizing: border-box;
    width: 100%;
    display: flex;
    flex-wrap: nowrap;
    align-items: center;
    gap: 8px;
    overflow: hidden;
}

/* 链接：单行不换行，图标在上、名称在下 */
.eps-external-list {
    box-sizing: border-box;
    width: 100%;
    display: flex;
    flex-wrap: nowrap;
    align-items: flex-start;
    gap: 12px;
    overflow: hidden;
}

</style>
