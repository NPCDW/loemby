<template>
    <div class="roe-page">
        <div class="roe-toolrow">
            <el-button plain @click="gotoMediaLibrary" class="nav-btn">
                <el-icon><i-ep-Film /></el-icon>
                <span>媒体库</span>
            </el-button>
            <el-input v-model="search_str" autofocus @keyup.enter="search" placeholder="在这台服务器中搜索" class="query__input">
                <template #append>
                    <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
                </template>
            </el-input>
        </div>

        <!-- 三个区：继续观看 / 收藏 / 统计。切换用下划线标签，不用卡片外框 -->
        <el-tabs v-model="activePane" @tab-change="handlePaneChange" class="panes">
            <el-tab-pane label="继续观看" name="ContinuePlay">
                <el-scrollbar style="height: calc(100vh - 232px);">
                    <el-skeleton :loading="episodesLoading" animated>
                        <template #template>
                            <div class="continue">
                                <div class="continue__card" v-for="i in 4" :key="i">
                                    <el-skeleton-item variant="image" style="height: 132px; width: 234px;" />
                                </div>
                            </div>
                        </template>
                        <div v-if="episodesList.length" class="continue">
                            <article
                                v-for="episodeItem in episodesList"
                                :key="episodeItem.Id"
                                class="continue__card"
                                :class="{ 'is-dismissed': deletedContinuePlayList.indexOf(episodeItem.Id) > -1 }"
                                @click="gotoEpisodes(episodeItem.Id)"
                            >
                                <div class="continue__art">
                                    <img v-if="episodeItem.Type == 'Episode'" v-lazy="useImage().images[embyServerId + ':parent-cover:' + episodeItem.Id]" />
                                    <img v-else v-lazy="useImage().images[embyServerId + ':cover:' + episodeItem.Id]" />
                                    <span class="continue__bar">
                                        <span class="continue__bar-fill" :style="{ width: (episodeItem.UserData?.Played ? 100 : Math.trunc(episodeItem.UserData?.PlayedPercentage || 0)) + '%' }"></span>
                                    </span>
                                </div>
                                <div class="continue__meta">
                                    <div class="continue__line1">
                                        <span class="continue__series" v-if="episodeItem.Type == 'Episode'">{{ episodeItem.SeriesName }}</span>
                                        <span class="continue__title">{{ episodeItem.Name }}</span>
                                    </div>
                                    <div class="continue__foot">
                                        <span class="continue__ep mono" v-if="episodeItem.Type == 'Episode'">
                                            {{ 'S' + (episodeItem.ParentIndexNumber || '-') + ' E' + (episodeItem.IndexNumber || '-') }}
                                        </span>
                                        <span class="continue__percent mono">
                                            {{ episodeItem.UserData?.Played ? '已看完' : Math.trunc(episodeItem.UserData?.PlayedPercentage || 0) + '%' }}
                                        </span>
                                        <span class="continue__ops" @click.stop>
                                            <button
                                                v-if="deletedContinuePlayList.indexOf(episodeItem.Id) == -1"
                                                class="mini-op"
                                                title="从继续观看中移除"
                                                :disabled="deleteContinuePlayLoading[episodeItem.Id]"
                                                @click="deleteContinuePlay(episodeItem.Id, true)"
                                            >
                                                <el-icon :size="14" :class="deleteContinuePlayLoading[episodeItem.Id] ? 'is-loading' : ''"><i-ep-Delete /></el-icon>
                                            </button>
                                            <button
                                                v-else
                                                class="mini-op mini-op--undo"
                                                :disabled="deleteContinuePlayLoading[episodeItem.Id]"
                                                @click="deleteContinuePlay(episodeItem.Id, false)"
                                            >撤销</button>
                                        </span>
                                    </div>
                                </div>
                            </article>
                        </div>
                        <div v-else class="roe-empty">
                            <span class="roe-empty__line">没有未看完的内容</span>
                            <span>播放任意影片后，会自动出现在这里。</span>
                        </div>
                    </el-skeleton>
                    <el-pagination
                        v-model:current-page="episodesCurrentPage"
                        v-model:page-size="episodesPageSize"
                        layout="total, prev, pager, next, jumper"
                        :total="episodesTotal"
                        @current-change="handleContinuePlayPageChange"
                        hide-on-single-page
                    />
                </el-scrollbar>
            </el-tab-pane>

            <el-tab-pane :label="'收藏 ' + (favoriteTotal || '')" name="Favorite">
                <el-scrollbar style="height: calc(100vh - 232px);">
                    <el-skeleton :loading="favoriteLoading" animated>
                        <template #template>
                            <div class="grid">
                                <div class="grid__cell" v-for="i in 8" :key="i">
                                    <el-skeleton-item variant="image" style="height: 160px; width: 112px;" />
                                </div>
                            </div>
                        </template>
                        <div v-if="favoriteList.length" class="grid">
                            <ItemCard v-for="favoriteItem in favoriteList" :key="favoriteItem.Id" :item="favoriteItem" :embyServerId="embyServerId" :show-series-name="true" />
                        </div>
                        <div v-else class="roe-empty">
                            <span class="roe-empty__line">还没有收藏</span>
                            <span>在影片或单集上点星标，就会汇总到这里。</span>
                        </div>
                    </el-skeleton>
                    <el-pagination
                        v-model:current-page="favoriteCurrentPage"
                        v-model:page-size="favoritePageSize"
                        layout="total, prev, pager, next, jumper"
                        :total="favoriteTotal"
                        @current-change="handleFavoritePageChange"
                        hide-on-single-page
                    />
                </el-scrollbar>
            </el-tab-pane>

            <el-tab-pane label="统计" name="MediaLibraryCount">
                <el-skeleton :loading="mediaLibraryCountLoading" animated>
                    <template #template>
                        <div class="stats">
                            <div class="stat" v-for="i in 3" :key="i">
                                <el-skeleton-item variant="text" style="width: 60px;" />
                                <el-skeleton-item variant="h1" style="width: 90px; margin-top: 8px;" />
                            </div>
                        </div>
                    </template>
                    <div class="stats">
                        <div class="stat">
                            <span class="stat__label">电影</span>
                            <span class="stat__value mono">{{ (mediaLibraryCount?.MovieCount || 0).toLocaleString() }}</span>
                        </div>
                        <div class="stat">
                            <span class="stat__label">剧</span>
                            <span class="stat__value mono">{{ (mediaLibraryCount?.SeriesCount || 0).toLocaleString() }}</span>
                        </div>
                        <div class="stat">
                            <span class="stat__label">单集</span>
                            <span class="stat__value mono">{{ (mediaLibraryCount?.EpisodeCount || 0).toLocaleString() }}</span>
                        </div>
                    </div>
                </el-skeleton>
            </el-tab-pane>
        </el-tabs>
    </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, EpisodeItem, SearchItem, MediaLibraryCount } from '../../api/embyApi';
import { ElMessage } from 'element-plus';
import ItemCard from '../../components/ItemCard.vue';
import { useImage } from '../../store/image';

const router = useRouter()
const route = useRoute()

const embyServerId = <string>route.params.embyId

const search_str = ref('')
const search = async () => {
    router.push('/nav/emby/' + embyServerId + '/search?search=' + encodeURIComponent(search_str.value))
}

const episodesLoading = ref(false)
const episodesList = ref<EpisodeItem[]>([])
const episodesCurrentPage = ref(1)
const episodesPageSize = ref(12)
const episodesTotal = ref(0)
const deletedContinuePlayList = ref<string[]>([])
const handleContinuePlayPageChange = (val: number) => {
    episodesCurrentPage.value = val
    getContinuePlayList(val, episodesPageSize.value)
}

function getContinuePlayList(currentPage: number, pageSize: number) {
    episodesLoading.value = true
    episodesCurrentPage.value = currentPage
    episodesPageSize.value = pageSize
    return embyApi.getContinuePlayList(embyServerId, (currentPage - 1) * pageSize, pageSize).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
        for (let item of episodesList.value) {
            useImage().loadCover(embyServerId, item)
            if (item.Type == 'Episode') {
                useImage().loadParentCover(embyServerId, item)
            }
        }
    }).catch(e => ElMessage.error(e)).finally(() => episodesLoading.value = false)
}

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
}
function gotoMediaLibrary() {
    router.push('/nav/emby/' + embyServerId + '/mediaLibrary')
}

const deleteContinuePlayLoading = ref<{[key: string]: boolean}>({})
function deleteContinuePlay(episodesId: string, hide: boolean) {
    deleteContinuePlayLoading.value[episodesId] = true
    return embyApi.hideFromResume(embyServerId, episodesId, hide).then(async () => {
        if (hide) {
            deletedContinuePlayList.value.push(episodesId)
        } else {
            deletedContinuePlayList.value.splice(deletedContinuePlayList.value.indexOf(episodesId), 1)
        }
    }).catch(e => ElMessage.error(e)).finally(() => deleteContinuePlayLoading.value[episodesId] = false)
}

const favoriteLoading = ref(false)
const favoriteList = ref<SearchItem[]>([])
const favoriteCurrentPage = ref(1)
const favoritePageSize = ref(6)
const favoriteTotal = ref(0)
const handleFavoritePageChange = (val: number) => {
    favoriteCurrentPage.value = val
    getFavoriteList(val, favoritePageSize.value)
}

function getFavoriteList(currentPage: number, pageSize: number) {
    favoriteLoading.value = true
    favoriteCurrentPage.value = currentPage
    favoritePageSize.value = pageSize
    return embyApi.getFavoriteList(embyServerId, (currentPage - 1) * pageSize, pageSize).then(async response => {
        let json: EmbyPageList<SearchItem> = JSON.parse(response);
        favoriteList.value = json.Items
        favoriteTotal.value = json.TotalRecordCount
    }).catch(e => ElMessage.error(e)).finally(() => favoriteLoading.value = false)
}

const mediaLibraryCountLoading = ref(false)
const mediaLibraryCount = ref<MediaLibraryCount>()
function getMediaLibraryCount() {
    mediaLibraryCountLoading.value = true
    return embyApi.count(embyServerId).then(async response => {
        let json: MediaLibraryCount = JSON.parse(response);
        mediaLibraryCount.value = json
    }).catch(e => ElMessage.error(e)).finally(() => mediaLibraryCountLoading.value = false)
}

const activePane = ref('ContinuePlay')
function handlePaneChange() {
    if (activePane.value == 'ContinuePlay') {
        episodesList.value = []
        episodesCurrentPage.value = 1
        episodesPageSize.value = 6
        episodesTotal.value = 0
        deletedContinuePlayList.value = []
        getContinuePlayList(episodesCurrentPage.value, episodesPageSize.value)
    } else if (activePane.value == 'Favorite') {
        favoriteList.value = []
        favoriteCurrentPage.value = 1
        favoritePageSize.value = 12
        favoriteTotal.value = 0
        getFavoriteList(favoriteCurrentPage.value, favoritePageSize.value)
    } else if (activePane.value == 'MediaLibraryCount') {
        mediaLibraryCount.value = undefined
        getMediaLibraryCount()
    }
}
handlePaneChange()
</script>

<style scoped>
.el-scrollbar {
  height: 100%;
}

.episode-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 25px;
}

.episode-card {
    display: flex;
    height: 180px;
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
}

.episode-cover {
    flex: 0 0 120px;
    height: 100%;
    overflow: hidden;
}

.episode-cover img {
    border-radius: 10px;
    width: 100%;
    height: 100%;
    /* object-fit: cover; */
}

.episode-info {
    flex: 1;
    padding: 20px;
    position: relative;
    overflow: hidden;
}

.background-pattern {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    opacity: 0.45;
    background-size: cover;
    background-position: center;
    /* 添加从左到右的透明度渐变 */
    mask-image: linear-gradient(to left, 
        rgba(0,0,0,0.8) 0%, 
        rgba(0,0,0,0.5) 30%, 
        rgba(0,0,0,0.2) 70%, 
        rgba(0,0,0,0) 100%);
    -webkit-mask-image: linear-gradient(to left, 
        rgba(0,0,0,0.8) 0%, 
        rgba(0,0,0,0.5) 30%, 
        rgba(0,0,0,0.2) 70%, 
        rgba(0,0,0,0) 100%);
}

.episode-content {
    position: relative;
    z-index: 2;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
}

.episode-title {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 5px;
    line-height: 1.3;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
}

.episode-number {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.9);
    margin-bottom: 15px;
    font-weight: 400;
}

.episode-duration {
    display: flex;
    align-items: center;
    margin-top: auto;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.85);
}
</style>
