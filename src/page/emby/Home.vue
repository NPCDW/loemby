<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-tabs v-model="activePane" @tab-change="handlePaneChange" style="height: calc(100vh - 52px); padding: 0 20px;">
        <el-tab-pane label="继续观看" name="ContinuePlay">
            <el-scrollbar style="height: calc(100vh - 107px);">
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
                        <el-card style="width: 300px; margin: 5px;" v-for="episodesItem in episodesList" :key="episodesItem.Id">
                            <template v-if="episodesItem.Type == 'Episode'">
                                <el-link :underline="false" @click="gotoSeries(episodesItem.SeriesId)"><h2>{{ episodesItem.SeriesName }}</h2></el-link>
                                <p><el-link :underline="false" @click="gotoEpisodes(episodesItem.Id)">{{ 'S' + episodesItem.ParentIndexNumber + 'E' + episodesItem.IndexNumber + '. ' + episodesItem.Name }}</el-link></p>
                            </template>
                            <template v-else>
                                <el-link :underline="false" @click="gotoEpisodes(episodesItem.Id)"><h2>{{ episodesItem.Name }}</h2></el-link>
                            </template>
                            <p><el-progress :percentage="episodesItem.UserData?.Played ? 100 : episodesItem.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" /></p>
                            <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} <el-tag disable-transitions>{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</el-tag></p>
                            <p><el-button type="primary" @click="gotoEpisodes(episodesItem.Id)">继续</el-button></p>
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
            <el-scrollbar style="height: calc(100vh - 107px);">
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
                        <ItemCard v-for="favoriteItem in favoriteList" :key="favoriteItem.Id" :item="favoriteItem" :embyServer="embyServer" />
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
        <el-tab-pane label="媒体库" name="MediaLibrary">
            <el-scrollbar style="height: calc(100vh - 107px);">
                <div>
                    <el-scrollbar>
                        <el-skeleton :loading="mediaLibraryLoading" animated>
                            <template #template>
                                <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                                    <div v-for="i in 5" :key="i" style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                        <el-skeleton-item variant="image" style="width: 267px; height: 150px;" />
                                        <p><el-skeleton-item variant="text" style="width: 100px" /></p>
                                    </div>
                                </div>
                            </template>
                            <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                                <div v-for="item in mediaLibraryList" :key="item.Id" style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                    <div style="min-width: 267px; min-height: 150px;">
                                        <img v-lazy="images[item.Id]" style="max-width: 267px; max-height: 150px;" />
                                    </div>
                                    <span>{{ item.Name }}</span>
                                </div>
                            </div>
                        </el-skeleton>
                    </el-scrollbar>
                </div>
                <div v-for="mediaLibrary in mediaLibraryList">
                    <template v-if="mediaLibraryChildList[mediaLibrary.Id] && mediaLibraryChildList[mediaLibrary.Id].length > 0">
                        <h1>{{ mediaLibrary.Name }}</h1>
                        <el-scrollbar>
                            <div style="display: flex;">
                                <el-skeleton :loading="mediaLibraryChildLoading[mediaLibrary.Id]" animated>
                                    <template #template>
                                        <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                                            <div v-for="i in 8" :key="i" style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                                <el-skeleton-item variant="image" style="width: 115px; height: 160px;" />
                                                <p><el-skeleton-item variant="text" style="width: 60px" /></p>
                                            </div>
                                        </div>
                                    </template>
                                    <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                                        <div v-for="item in mediaLibraryChildList[mediaLibrary.Id]" :key="item.Id"
                                            @click="() => {item.Type == 'Series' ? gotoSeries(item.Id) : gotoEpisodes(item.Id)}"
                                            style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                            <div style="min-width: 115px; min-height: 160px;">
                                                <img v-lazy="images[item.Id]" style="max-width: 115px; max-height: 160px;" />
                                            </div>
                                            <el-text truncated style="max-width: 115px;">{{ item.Name }}</el-text>
                                        </div>
                                    </div>
                                </el-skeleton>
                            </div>
                        </el-scrollbar>
                    </template>
                </div>
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
import { watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, EpisodesItems, SearchItems, MediaLibraryItem, MediaLibraryCount } from '../../api/embyApi';
import { useConfig } from '../../store/config';
import { ElMessage } from 'element-plus';
import { formatBytes } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'
import ItemCard from '../../components/ItemCard.vue';
import invoke from '../../api/invoke';

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.id)!

watch(() => route.params.id, (newId, _oldId) => {
    embyServer = useConfig().getEmbyServer(<string>newId)!

    handlePaneChange()
})

const search_str = ref('')
const search = async () => {
    router.push('/nav/emby/' + embyServer.id + '/search?search=' + encodeURIComponent(search_str.value))
}

const episodesLoading = ref(false)
const episodesList = ref<EpisodesItems[]>([])
const episodesCurrentPage = ref(1)
const episodesPageSize = ref(6)
const episodesTotal = ref(0)
const handleContinuePlayPageChange = (val: number) => {
    episodesCurrentPage.value = val
    getContinuePlayList(val, episodesPageSize.value)
}

function getContinuePlayList(currentPage: number, pageSize: number) {
    episodesLoading.value = true
    episodesCurrentPage.value = currentPage
    episodesPageSize.value = pageSize
    return embyApi.getContinuePlayList(embyServer, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => episodesLoading.value = false)
}
getContinuePlayList(episodesCurrentPage.value, episodesPageSize.value)

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/series/' + seriesId)
}

const favoriteLoading = ref(false)
const favoriteList = ref<SearchItems[]>([])
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
    return embyApi.getFavoriteList(embyServer, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<SearchItems> = JSON.parse(response.body);
        favoriteList.value = json.Items
        favoriteTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => favoriteLoading.value = false)
}

const mediaLibraryLoading = ref(false)
const mediaLibraryList = ref<MediaLibraryItem[]>([])
function getMediaLibraryList() {
    mediaLibraryLoading.value = true
    return embyApi.getMediaLibraryList(embyServer).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<MediaLibraryItem> = JSON.parse(response.body);
        mediaLibraryList.value = json.Items
        for (let item of mediaLibraryList.value) {
            loadImage(item.Id)
            getMediaLibraryChildLatest(item.Id)
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => mediaLibraryLoading.value = false)
}
const mediaLibraryChildLoading = ref<{[key: string]: boolean}>({})
const mediaLibraryChildList = ref<{[key: string]: SearchItems[]}>({})
function getMediaLibraryChildLatest(parentId: string) {
    mediaLibraryChildLoading.value[parentId] = true
    return embyApi.getMediaLibraryChildLatest(embyServer, parentId, 16).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: SearchItems[] = JSON.parse(response.body);
        mediaLibraryChildList.value[parentId] = json
        for (let item of mediaLibraryChildList.value[parentId]) {
            loadImage(item.Id)
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => mediaLibraryChildLoading.value[parentId] = false)
}

const images = ref<{[key: string]: string}>({})
function loadImage(itemId: string) {
    invoke.loadImage({
        image_url: embyApi.getImageUrl(embyServer, itemId)!,
        proxy_url: useConfig().getBrowseProxyUrl(embyServer.browse_proxy_id),
        user_agent: embyServer.user_agent!,
    }).then(response => {
        images.value[itemId] = response
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}

const mediaLibraryCountLoading = ref(false)
const mediaLibraryCount = ref<MediaLibraryCount>()
function getMediaLibraryCount() {
    mediaLibraryCountLoading.value = true
    return embyApi.count(embyServer).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: MediaLibraryCount = JSON.parse(response.body);
        mediaLibraryCount.value = json
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => mediaLibraryCountLoading.value = false)
}

const activePane = ref('ContinuePlay')
function handlePaneChange() {
    if (activePane.value == 'ContinuePlay') {
        episodesList.value = []
        episodesCurrentPage.value = 1
        episodesPageSize.value = 6
        episodesTotal.value = 0
        getContinuePlayList(episodesCurrentPage.value, episodesPageSize.value)
    } else if (activePane.value == 'Favorite') {
        favoriteList.value = []
        favoriteCurrentPage.value = 1
        favoritePageSize.value = 12
        favoriteTotal.value = 0
        getFavoriteList(favoriteCurrentPage.value, favoritePageSize.value)
    } else if (activePane.value == 'MediaLibrary') {
        mediaLibraryList.value = []
        getMediaLibraryList()
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