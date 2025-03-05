<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
        <el-tabs v-model="activePane" @tab-click="handlePaneClick">
            <el-tab-pane label="继续观看" name="ContinuePlay">
                <div>
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
                            <el-card style="width: 300px; margin: 5px;" v-for="episodesItem in episodesList">
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
                    </el-skeleton>
                    <el-pagination
                        v-model:current-page="episodesCurrentPage"
                        v-model:page-size="episodesPageSize"
                        layout="total, prev, pager, next, jumper"
                        :total="episodesTotal"
                        @current-change="handleContinuePlayPageChange"
                        hide-on-single-page
                    />
                </div>
            </el-tab-pane>
            <el-tab-pane label="收藏" name="Favorite">
                <div>
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
                </div>
            </el-tab-pane>
            <el-tab-pane label="媒体库" name="MediaLibrary">
                
            </el-tab-pane>
        </el-tabs>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, EpisodesItems, SearchItems } from '../../api/embyApi';
import { useConfig } from '../../store/config';
import { ElMessage, TabsPaneContext } from 'element-plus';
import { formatBytes } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'
import ItemCard from '../../components/ItemCard.vue';

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.id)!

watch(() => route.params.id, (newId, _oldId) => {
    embyServer = useConfig().getEmbyServer(<string>newId)!

    episodesList.value = []
    episodesCurrentPage.value = 1
    episodesPageSize.value = 6
    episodesTotal.value = 0
    getContinuePlayList(episodesCurrentPage.value, episodesPageSize.value)
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

const activePane = ref('ContinuePlay')
function handlePaneClick(pane: TabsPaneContext, _ev: Event) {
    if (pane.paneName == 'ContinuePlay') {
        episodesList.value = []
        episodesCurrentPage.value = 1
        episodesPageSize.value = 6
        episodesTotal.value = 0
        getContinuePlayList(episodesCurrentPage.value, episodesPageSize.value)
    } else if (pane.paneName == 'Favorite') {
        favoriteList.value = []
        favoriteCurrentPage.value = 1
        favoritePageSize.value = 12
        favoriteTotal.value = 0
        getFavoriteList(favoriteCurrentPage.value, favoritePageSize.value)
    } else if (pane.paneName == 'MediaLibrary') {

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