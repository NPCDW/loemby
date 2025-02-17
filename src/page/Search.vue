<template>
    <div>
        <el-input v-model="search_str" @keyup.enter="search">
            <template #append>
                <el-button @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
        <div v-for="(val, rootKey) in root_search_result">
            <div>{{ val.server.server_name }}</div>
            <v-card class="mx-auto" max-width="344" v-for="rootItem in val.result.Items">
                <!-- <v-img height="200px" src="https://cdn.vuetifyjs.com/images/cards/sunshine.jpg" cover></v-img> -->

                <v-card-title>
                    {{ rootItem.Name }}
                </v-card-title>

                <v-card-subtitle v-if="rootItem.Type == 'Series'">
                    {{ rootItem.ProductionYear + '-' + rootItem.EndDate.substring(0, 4) }} 总集数：{{ rootItem.UserData.PlayCount + rootItem.UserData.UnplayedItemCount }}
                </v-card-subtitle>
                <v-card-subtitle v-else>
                    {{ rootItem.ProductionYear }} 最大媒体流：{{ formatBytes(maxMediaSources(rootItem.MediaSources)) }}
                </v-card-subtitle>

                <v-card-actions>
                    <v-btn color="orange-lighten-2" text="Explore"></v-btn>

                    <v-spacer></v-spacer>

                    <v-btn @click="showSeasons[rootItem.Id] = !showSeasons[rootItem.Id]">
                        <el-icon v-if="showSeasons[rootItem.Id]"><i-ep-ArrowUpBold /></el-icon>
                        <el-icon v-else @click="getSeasons(val.server, rootItem.Id)"><i-ep-ArrowDownBold /></el-icon>
                    </v-btn>
                </v-card-actions>

                <v-expand-transition>
                    <div v-show="showSeasons[rootItem.Id]">
                        <v-divider></v-divider>

                        <v-card class="mx-auto" max-width="344" v-for="seasonsItem in seasons_result[rootKey][rootItem.Id].Items">
                            <!-- <v-img height="200px" src="https://cdn.vuetifyjs.com/images/cards/sunshine.jpg" cover></v-img> -->

                            <v-card-title>
                                {{ seasonsItem.Name }}
                            </v-card-title>

                            <v-card-subtitle>
                                {{ seasonsItem.ProductionYear }} 总集数：{{ seasonsItem.UserData.PlayCount + seasonsItem.UserData.UnplayedItemCount }}
                            </v-card-subtitle>

                            <v-card-actions>
                                <v-btn color="orange-lighten-2" text="Explore"></v-btn>

                                <v-spacer></v-spacer>

                                <v-btn @click="showEpisodes[seasonsItem.Id] = !showEpisodes[seasonsItem.Id]">
                                    <el-icon v-if="showEpisodes[seasonsItem.Id]"><i-ep-ArrowUpBold /></el-icon>
                                    <el-icon v-else @click="getEpisodes(val.server, rootItem.Id, seasonsItem.Id, 1, 10)"><i-ep-ArrowDownBold /></el-icon>
                                </v-btn>
                            </v-card-actions>

                            <v-expand-transition>
                                <div v-show="showEpisodes[seasonsItem.Id]">
                                    <v-divider></v-divider>

                                    <v-card class="mx-auto" max-width="344" v-for="episodesItem in episodes_result[rootKey][rootItem.Id][seasonsItem.Id].result.Items">
                                        <!-- <v-img height="200px" src="https://cdn.vuetifyjs.com/images/cards/sunshine.jpg" cover></v-img> -->

                                        <v-card-title>
                                            {{ episodesItem.Name }}
                                        </v-card-title>

                                        <v-card-subtitle>
                                            {{ episodesItem.PremiereDate.substring(0, 10) }} 最大媒体流：{{ formatBytes(maxMediaSources(episodesItem.MediaSources)) }}
                                        </v-card-subtitle>

                                        <el-pagination
                                            v-model:current-page="val.currentPage"
                                            v-model:page-size="val.pageSize"
                                            layout="total, prev, pager, next, jumper"
                                            :total="val.result.TotalRecordCount"
                                            @current-change="handleEpisodesPageChange(val.currentPage, val.server, rootItem.Id, seasonsItem.Id)"
                                            hide-on-single-page
                                        />
                                    </v-card>
                                </div>
                            </v-expand-transition>
                        </v-card>
                    </div>
                </v-expand-transition>
            </v-card>
            <el-pagination
                v-model:current-page="val.currentPage"
                v-model:page-size="val.pageSize"
                layout="total, prev, pager, next, jumper"
                :total="val.result.TotalRecordCount"
                @current-change="handleRootSearchPageChange(val.currentPage, val.server)"
                hide-on-single-page
            />
        </div>
    </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage } from 'element-plus'

const search_str = ref('')
const root_search_result = ref<{[key: string]: {server: EmbyServerConfig, currentPage: number, pageSize: number, result: SearchResult}}>({})
const seasons_result = ref<{[key: string]: {[key: string]: SeasonsResult}}>({})
const episodes_result = ref<{[key: string]: {[key: string]: {[key: string]: {currentPage: number, pageSize: number, result: EpisodesResult}}}}>({})

const showSeasons = ref<{[key: string]: boolean}>({})
const showEpisodes = ref<{[key: string]: boolean}>({})

interface SearchResult {
    TotalRecordCount: number,
    Items: {
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
    }[]
}

interface SeasonsResult {
    TotalRecordCount: number,
    Items: {
        Name: string,
        Id: string,
        ProductionYear: number,
        UserData: {
            UnplayedItemCount: number,
            PlayCount: number
        }
    }[]
}

interface EpisodesResult {
    TotalRecordCount: number,
    Items: {
        Name: string,
        Id: string,
        PremiereDate: string,
        MediaSources: {
            Size: number,
            Name: string
        }[],
    }[]
}

async function search() {
    let config = await useConfig().get_config();
    for (let embyServer of config.emby_server!) {
        singleEmbySearch(embyServer, 1, 10);
    }
}
async function handleRootSearchPageChange(val: number, embyServer: EmbyServerConfig) {
    await singleEmbySearch(embyServer, val, 10)
}
async function singleEmbySearch(embyServer: EmbyServerConfig, currentPage: number, pageSize: number) {
    let search_result = {server: embyServer, currentPage: currentPage, pageSize: pageSize, result: {} as SearchResult}
    return embyApi.search(embyServer, search_str.value, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: SearchResult = await response.json();
        search_result.result = json
        root_search_result.value[embyServer.id!] = search_result
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
async function getSeasons(embyServer: EmbyServerConfig, series_id: string) {
    if (seasons_result.value[embyServer.id!][series_id]) {
        return
    }
    embyApi.seasons(embyServer, series_id).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: SeasonsResult = await response.json();
        seasons_result.value[embyServer.id!][series_id] = json
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
async function getEpisodes(embyServer: EmbyServerConfig, series_id: string, seasons_id: string, currentPage: number, pageSize: number) {
    if (episodes_result.value[embyServer.id!][series_id][seasons_id]) {
        return
    }
    let result = {currentPage: currentPage, pageSize: pageSize, result: {} as EpisodesResult}
    return embyApi.episodes(embyServer, series_id, seasons_id, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: EpisodesResult = await response.json();
        result.result = json
        episodes_result.value[embyServer.id!][series_id][seasons_id] = result
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}
async function handleEpisodesPageChange(val: number, embyServer: EmbyServerConfig, series_id: string, seasons_id: string) {
    await getEpisodes(embyServer, series_id, seasons_id, val, 10)
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
const formatBytes = (size: number) => {
  const units: string[] = ['KB', 'MB', 'GB', 'TB'];
  for (let index = 0; index < units.length; index++) {
    size /= 1024;
    if (size < 1024) {
      return size.toFixed(2) + " " + units[index];
    }
  }
  return size.toFixed(2) + units[units.length - 1]
}

</script>