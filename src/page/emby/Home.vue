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
                        <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                            <el-card class="box-item" v-for="i in 5" :key="i">
                                <el-skeleton-item variant="h1" style="width: 50%; margin-top: 10px;" />
                                <p><el-skeleton-item variant="text" style="width: 80%" /></p>
                                <p><el-skeleton-item variant="text" style="width: 90%" /></p>
                                <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                                <p><el-skeleton-item variant="button" style="width: 30%" /></p>
                            </el-card>
                        </div>
                    </template>
                    <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <el-card style="width: 300px; margin: 5px;" v-for="episodeItem in episodesList" :key="episodeItem.Id">
                            <template v-if="episodeItem.Type == 'Episode'">
                                <el-link :underline="false" @click="gotoSeries(episodeItem.SeriesId)"><h2>{{ episodeItem.SeriesName }}</h2></el-link>
                                <p><el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)">{{ 'S' + (episodeItem.ParentIndexNumber || -1) + 'E' + (episodeItem.IndexNumber || -1) + '. ' + episodeItem.Name }}</el-link></p>
                            </template>
                            <template v-else>
                                <el-link :underline="false" @click="gotoEpisodes(episodeItem.Id)"><h2>{{ episodeItem.Name }}</h2></el-link>
                            </template>
                            <p><el-progress :percentage="episodeItem.UserData?.Played ? 100 : episodeItem.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" /></p>
                            <p>
                                {{ episodeItem.PremiereDate ? episodeItem.PremiereDate.substring(0, 10) : '' }}
                            </p>
                            <p>
                                <el-button type="primary" @click="gotoEpisodes(episodeItem.Id)">Go</el-button>
                                <template v-if="deletedContinuePlayList.indexOf(episodeItem.Id) == -1">
                                    <el-button plain type="danger" :loading="deleteContinuePlayLoading[episodeItem.Id]" @click="deleteContinuePlay(episodeItem.Id, true)"><i-ep-Delete /></el-button></template>
                                <template v-else>
                                    <el-button type="danger" :loading="deleteContinuePlayLoading[episodeItem.Id]" @click="deleteContinuePlay(episodeItem.Id, false)">撤销</el-button>
                                </template>
                            </p>
                        </el-card>
                    </div>
                    <div style="display: flex;justify-content: center;">
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
                        <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                            <el-card class="box-item" v-for="i in 5" :key="i">
                                <el-skeleton-item variant="h1" style="width: 50%; margin-top: 10px;" />
                                <p><el-skeleton-item variant="text" style="width: 80%" /></p>
                                <p><el-skeleton-item variant="text" style="width: 90%" /></p>
                                <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                                <p><el-skeleton-item variant="button" style="width: 30%" /></p>
                            </el-card>
                        </div>
                    </template>
                    <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <ItemCard v-for="favoriteItem in favoriteList" :key="favoriteItem.Id" :item="favoriteItem" :embyServer="embyServer" :show-series-name="true" />
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
import { onMounted, onUnmounted, ref, watchEffect } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, EpisodeItem, SearchItem, MediaLibraryCount } from '../../api/embyApi';
import { ElMessage } from 'element-plus';
import ItemCard from '../../components/ItemCard.vue';
import { EmbyServer, useEmbyServer } from '../../store/db/embyServer';
import { useEventBus } from '../../store/eventBus';

const router = useRouter()
const route = useRoute()

const embyServer = ref<EmbyServer>({})
async function getEmbyServer(embyId: string) {
    return useEmbyServer().getEmbyServer(embyId).then(value => {
        embyServer.value = value!;
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
function embyServerChanged(payload?: {event?: string, id?: string}) {
    if (payload?.id === route.params.embyId) {
        getEmbyServer(payload?.id)
    }
}
onMounted(() => useEventBus().on('EmbyServerChanged', embyServerChanged))
onUnmounted(() => useEventBus().remove('EmbyServerChanged', embyServerChanged))

watchEffect(async () => {
    await getEmbyServer(<string>route.params.embyId)
    handlePaneChange()
})

const search_str = ref('')
const search = async () => {
    router.push('/nav/emby/' + embyServer.value.id + '/search?search=' + encodeURIComponent(search_str.value))
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
    return embyApi.getContinuePlayList(embyServer.value.id!, (currentPage - 1) * pageSize, pageSize).then(async response => {
        let json: EmbyPageList<EpisodeItem> = JSON.parse(response);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => ElMessage.error(e)).finally(() => episodesLoading.value = false)
}

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/series/' + seriesId)
}
function gotoMediaLibrary() {
    router.push('/nav/emby/' + embyServer.value.id + '/mediaLibrary')
}

const deleteContinuePlayLoading = ref<{[key: string]: boolean}>({})
function deleteContinuePlay(episodesId: string, hide: boolean) {
    deleteContinuePlayLoading.value[episodesId] = true
    return embyApi.hideFromResume(embyServer.value.id!, episodesId, hide).then(async () => {
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
    return embyApi.getFavoriteList(embyServer.value.id!, (currentPage - 1) * pageSize, pageSize).then(async response => {
        let json: EmbyPageList<SearchItem> = JSON.parse(response);
        favoriteList.value = json.Items
        favoriteTotal.value = json.TotalRecordCount
    }).catch(e => ElMessage.error(e)).finally(() => favoriteLoading.value = false)
}

const mediaLibraryCountLoading = ref(false)
const mediaLibraryCount = ref<MediaLibraryCount>()
function getMediaLibraryCount() {
    mediaLibraryCountLoading.value = true
    return embyApi.count(embyServer.value.id!).then(async response => {
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

</script>

<style scoped>
.box-container {
  display: flex;
  height: 500px;
}

.box-sidebar {
  width: 30%;
  border-right: 1px solid #18222C;
  padding-right: 20px;
  overflow-y: auto;
}

.box-item {
    width: 300px; margin: 5px;
}

.box-item:hover {
  background-color: #18222C;
}

.box-item.active {
  color: #409EFF;
}

.box-content {
  width: 70%;
  padding-left: 20px;
}

h2 {
  margin-top: 0;
}

.el-scrollbar {
  height: 100%;
}

</style>