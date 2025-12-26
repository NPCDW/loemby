<template>
    <div style="padding: 10px; display: flex;">
        <el-button type="primary" @click="gotoMediaLibrary" style="flex: none; margin-right: 5px;"><i-ep-Film /> 媒体库</el-button>
        <el-input v-model="search_str" autofocus @keyup.enter="search" style="flex: auto;">
            <template #append>
                <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-tabs v-model="activePane" @tab-change="handlePaneChange" style="height: calc(100vh - 82px); padding: 0 20px;">
        <el-tab-pane label="继续观看" name="ContinuePlay">
            <el-scrollbar style="height: calc(100vh - 137px);">
                <el-skeleton :loading="episodesLoading" animated>
                    <template #template>
                        <div class="episode-grid">
                            <el-card class="episode-card" v-for="i in 6" :key="i">
                                <div class="episode-cover">
                                    <el-skeleton-item variant="image" style="height: 160px; width: 115px;" />
                                </div>
                                <div class="episode-info">
                                    <div class="episode-content">
                                        <div class="episode-title">
                                            <h1><el-skeleton-item variant="text" style="width: 50%" /></h1>
                                        </div>
                                        <div class="episode-number">
                                            <p><el-skeleton-item variant="text" style="width: 80%" /></p>
                                        </div>
                                    </div>
                                </div>
                            </el-card>
                        </div>
                    </template>
                    <div class="episode-grid">
                        <div class="episode-card" v-for="episodeItem in episodesList" :key="episodeItem.Id">
                            <div class="episode-cover">
                                <template v-if="episodeItem.Type == 'Episode'">
                                    <img v-lazy="useImage().images[embyServerId + ':parent-cover:' + episodeItem.Id]" />
                                </template>
                                <template v-else>
                                    <img v-lazy="useImage().images[embyServerId + ':cover:' + episodeItem.Id]" />
                                </template>
                            </div>
                            <div class="episode-info">
                                <div class="background-pattern">
                                    <img v-lazy="useImage().images[embyServerId + ':cover:' + episodeItem.Id]" style="width: 100%;height: 100%;object-fit: cover" />
                                </div>
                                <div class="episode-content">
                                    <div>
                                        <template v-if="episodeItem.Type == 'Episode'">
                                            <el-link :underline="false" @click="gotoSeries(episodeItem.SeriesId)" style="display: block;">
                                                <div class="episode-title">{{ episodeItem.SeriesName }}</div>
                                            </el-link>
                                            <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)" style="display: block;">
                                                <div class="episode-number">{{ 'S' + (episodeItem.ParentIndexNumber || '-') + 'E' + (episodeItem.IndexNumber || '-') + '. ' + episodeItem.Name }}</div>
                                            </el-link>
                                        </template>
                                        <template v-else>
                                            <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)" style="display: block;">
                                                <div class="episode-title">{{ episodeItem.Name }}</div>
                                            </el-link>
                                        </template>
                                    </div>
                                    <div class="episode-duration">
                                        <el-progress style="width: 80%;" :percentage="episodeItem.UserData?.Played ? 100 : episodeItem.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" />
                                        <el-button type="primary" @click="gotoEpisodes(episodeItem.Id)">Go</el-button>
                                        <template v-if="deletedContinuePlayList.indexOf(episodeItem.Id) == -1">
                                            <el-button plain type="danger" :loading="deleteContinuePlayLoading[episodeItem.Id]" @click="deleteContinuePlay(episodeItem.Id, true)"><i-ep-Delete /></el-button></template>
                                        <template v-else>
                                            <el-button type="danger" :loading="deleteContinuePlayLoading[episodeItem.Id]" @click="deleteContinuePlay(episodeItem.Id, false)">撤销</el-button>
                                        </template>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div style="display: flex;justify-content: center; margin-top: 10px;">
                        <el-empty v-if="episodesList && episodesList.length == 0" :image-size="200" description="" />
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
        <el-tab-pane label="收藏" name="Favorite">
            <el-scrollbar style="height: calc(100vh - 137px);">
                <el-skeleton :loading="favoriteLoading" animated>
                    <template #template>
                        <div class="episode-grid">
                            <el-card class="episode-card" v-for="i in 6" :key="i">
                                <div class="episode-cover">
                                    <el-skeleton-item variant="image" style="height: 160px; width: 115px;" />
                                </div>
                                <div class="episode-info">
                                    <div class="episode-content">
                                        <div class="episode-title">
                                            <h1><el-skeleton-item variant="text" style="width: 50%" /></h1>
                                        </div>
                                        <div class="episode-number">
                                            <p><el-skeleton-item variant="text" style="width: 80%" /></p>
                                        </div>
                                    </div>
                                </div>
                            </el-card>
                        </div>
                    </template>
                    <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <ItemCard v-for="favoriteItem in favoriteList" :key="favoriteItem.Id" :item="favoriteItem" :embyServerId="embyServerId" :show-series-name="true" />
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
            <el-descriptions title="媒体库统计" :column="1" size="large" label-width="40">
                <el-descriptions-item label="电影">
                    <el-icon v-if="mediaLibraryCountLoading" class="is-loading"><i-ep-Loading /></el-icon>
                    <span v-else>{{ mediaLibraryCount?.MovieCount.toLocaleString() }}</span>
                </el-descriptions-item>
                <el-descriptions-item label="剧">
                    <el-icon v-if="mediaLibraryCountLoading" class="is-loading"><i-ep-Loading /></el-icon>
                    <span v-else>{{ mediaLibraryCount?.SeriesCount.toLocaleString() }}</span>
                </el-descriptions-item>
                <el-descriptions-item label="剧集">
                    <el-icon v-if="mediaLibraryCountLoading" class="is-loading"><i-ep-Loading /></el-icon>
                    <span v-else>{{ mediaLibraryCount?.EpisodeCount.toLocaleString() }}</span>
                </el-descriptions-item>
            </el-descriptions>
        </el-tab-pane>
    </el-tabs>
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
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServerId + '/series/' + seriesId)
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
    width: 100%;
    height: 100%;
    object-fit: cover;
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