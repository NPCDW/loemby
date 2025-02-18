<template>
    <div>
        <el-input v-model="search_str" @keyup.enter="search" style="padding: 10px;">
            <template #append>
                <el-button @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
        
        <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
            <el-collapse v-model="embyServerKeys">
                <el-collapse-item :title="val.server.server_name" :name="rootKey" v-for="(val, rootKey) in root_search_result">
                    <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <el-card style="width: 300px; margin: 5px;" v-for="rootItem in val.result.Items">
                            <p>{{ rootItem.Name }}</p>
                            <p v-if="rootItem.Type == 'Series'">
                                {{ rootItem.ProductionYear + (rootItem.EndDate && rootItem.EndDate.substring(0, 4) != rootItem.ProductionYear + '' ? '-' + rootItem.EndDate.substring(0, 4) : '') }}
                                总集数：{{ rootItem.UserData.PlayCount + rootItem.UserData.UnplayedItemCount }}
                            </p>
                            <p v-else>
                                {{ rootItem.ProductionYear }} 最大媒体流：{{ formatBytes(maxMediaSources(rootItem.MediaSources)) }}
                            </p>
                            <el-button v-if="rootItem.Type == 'Series'" @click="getSeasons(val.server, rootItem)" type="primary" plain>剧集</el-button>
                        </el-card>
                    </div>
                </el-collapse-item>
            </el-collapse>
        </el-scrollbar>

        <el-dialog
            v-model="dialogSeriesVisible"
            :title="dialogSeries?.Name"
            width="800"
        >
            <div class="note-container">
                <div class="note-sidebar">
                    <el-scrollbar>
                        <div
                             v-for="seasonsItem in dialogSeasonsList"
                            :key="seasonsItem.Id"
                            class="note-item"
                            :class="{ active: dialogSeasons?.Id === seasonsItem.Id }"
                            @click="getEpisodes(dialogEmbyServer!, dialogSeries!.Id, seasonsItem, 1, 10)"
                        >
                            <h3>{{ seasonsItem.Name }}</h3>
                            <p>{{ seasonsItem.ProductionYear }} 总集数：{{ seasonsItem.UserData.PlayCount + seasonsItem.UserData.UnplayedItemCount }}</p>
                        </div>
                    </el-scrollbar>
                </div>
                <div class="note-content">
                    <el-scrollbar>
                        <div v-for="episodesItem in dialogEpisodesList" class="note-item">
                            <p>{{ episodesItem.Name }}</p>
                            <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} 最大媒体流：{{ formatBytes(maxMediaSources(episodesItem.MediaSources)) }}</p>
                        </div>
                        <el-pagination
                            v-if="episodes_result[dialogEmbyServer!.id + '|' + dialogSeries!.Id + '|' + dialogSeasons?.Id]"
                            v-model:current-page="dialogEpisodesCurrentPage"
                            v-model:page-size="dialogEpisodesPageSize"
                            layout="total, prev, pager, next, jumper"
                            :total="episodes_result[dialogEmbyServer!.id + '|' + dialogSeries!.Id + '|' + dialogSeasons!.Id].total"
                            @current-change="handleEpisodesPageChange(dialogEpisodesCurrentPage, dialogEmbyServer!, dialogSeries!.Id, dialogSeasons!)"
                            hide-on-single-page
                        />
                    </el-scrollbar>
                </div>
            </div>
        </el-dialog>
    </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage } from 'element-plus'
import { formatBytes } from '../util/str_util'

const search_str = ref('')
const embyServerKeys = ref<string[]>([])

const root_search_result = ref<{[key: string]: {server: EmbyServerConfig, currentPage: number, pageSize: number, result: EmbyPageList<SearchItems>}}>({})
const seasons_result = ref<{[key: string]: EmbyPageList<SeasonsItems>}>({})
const episodes_result = ref<{[key: string]: {total: number, [key: number]: EpisodesItems[]}}>({})

const dialogSeriesVisible = ref(false)
const dialogEmbyServer = ref<EmbyServerConfig>()
const dialogSeries = ref<SearchItems>()
const dialogSeasons = ref<SeasonsItems>()
const dialogSeasonsList = ref<SeasonsItems[]>([])
const dialogEpisodesList = ref<EpisodesItems[]>([])
const dialogEpisodesCurrentPage = ref(1)
const dialogEpisodesPageSize = ref(10)

interface EmbyPageList<T> {
    TotalRecordCount: number,
    Items: T[]
}

interface SearchItems {
    Name: string,
    Id: string,
    ProductionYear: number,
    EndDate: string,
    Type: string,
    MediaSources: {
        Size: number,
        Name: string
    }[],
    UserData: {
        UnplayedItemCount: number,
        PlayCount: number
    }
}

interface SeasonsItems {
    Name: string,
    Id: string,
    ProductionYear: number,
    UserData: {
        UnplayedItemCount: number,
        PlayCount: number
    }
}

interface EpisodesItems {
    Name: string,
    Id: string,
    PremiereDate: string,
    MediaSources: {
        Size: number,
        Name: string
    }[],
}

async function search() {
    embyServerKeys.value = []
    let config = await useConfig().get_config();
    for (let embyServer of config.emby_server!) {
        singleEmbySearch(embyServer, 1, 30)
    }
}
async function singleEmbySearch(embyServer: EmbyServerConfig, currentPage: number, pageSize: number) {
    return embyApi.search(embyServer, search_str.value, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EmbyPageList<SearchItems> = await response.json();
        root_search_result.value[embyServer.id!] = {server: embyServer, currentPage, pageSize, result: json}
        if (json.Items.length > 0) {
            embyServerKeys.value.push(embyServer.id!)
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
async function getSeasons(embyServer: EmbyServerConfig, series: SearchItems) {
    dialogSeasonsList.value = []
    dialogEpisodesList.value = []
    dialogEmbyServer.value = embyServer
    dialogSeries.value = series
    dialogSeriesVisible.value = true
    if (seasons_result.value[embyServer.id! + '|' + series.Id]) {
        dialogSeasonsList.value = seasons_result.value[embyServer.id! + '|' + series.Id].Items
        return
    }
    return embyApi.seasons(embyServer, series.Id).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EmbyPageList<SeasonsItems> = await response.json();
        seasons_result.value[embyServer.id! + '|' + series.Id] = json
        dialogSeasonsList.value = json.Items
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
async function getEpisodes(embyServer: EmbyServerConfig, series_id: string, seasons: SeasonsItems, currentPage: number, pageSize: number) {
    dialogEpisodesList.value = []
    dialogSeasons.value = seasons
    if (!episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id]) {
        episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id] = {total: 0}
    }
    if (episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id][currentPage]) {
        dialogEpisodesList.value = episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id][currentPage]
        return
    }
    return embyApi.episodes(embyServer, series_id, seasons.Id, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = await response.json();
        episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id].total = json.TotalRecordCount
        episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id][currentPage]= json.Items
        dialogEpisodesList.value = json.Items
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
async function handleEpisodesPageChange(val: number, embyServer: EmbyServerConfig, series_id: string, seasons: SeasonsItems) {
    await getEpisodes(embyServer, series_id, seasons, val, dialogEpisodesPageSize.value)
}

const maxMediaSources = (mediaSources: {Size: number}[]) => {
    let max = 0
    for (let mediaSource of mediaSources) {
        if (mediaSource.Size > max) {
            max = mediaSource.Size
        }
    }
    return max
}
</script>

<style scoped>
.note-container {
  display: flex;
  height: 500px;
}

.note-sidebar {
  width: 30%;
  border-right: 1px solid #18222C;
  padding-right: 20px;
  overflow-y: auto;
}

.note-item {
  padding: 3px 10px;
  cursor: pointer;
  border-bottom: 1px solid #18222C;
}

.note-item:hover {
  background-color: #18222C;
}

.note-item.active {
  color: #409EFF;
}

.note-content {
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