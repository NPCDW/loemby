<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
        
        <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
            <el-collapse v-model="embyServerKeys">
                <el-collapse-item :title="embySearchItem.server.server_name" :name="embySearchItem.server.id" :disabled="embySearchItem.result?.Items.length == 0" v-for="embySearchItem in emby_search_result_list">
                    <template #icon>
                        <span style="display: flex; align-items: center; margin: auto 18px auto auto;">
                            <el-icon v-if="embySearchItem.server.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                            <el-icon v-else-if="embySearchItem.server.request_fail" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon>
                            <el-icon v-else-if="embySearchItem.result?.Items.length == 0" style="color: #909399;">empty</el-icon>
                            <el-icon v-else style="color: #67C23A;"><i-ep-SuccessFilled /></el-icon>
                        </span>
                    </template>
                    <div v-if="embySearchItem.success" style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <el-card style="width: 300px; margin: 5px;" v-for="rootItem in embySearchItem.result?.Items">
                            <template #header>
                                <el-link v-if="rootItem.Type == 'Series'" :underline="false" @click="gotoSeries(embySearchItem.server.id!, rootItem.Id)">{{ rootItem.Name }}</el-link>
                                <el-link v-else :underline="false" @click="gotoEpisodes(embySearchItem.server.id!, rootItem.Id)">{{ rootItem.Name }}</el-link>
                            </template>
                            <div style="margin-bottom: 10px;">
                                <span v-if="rootItem.Type == 'Series'">
                                    {{ rootItem.ProductionYear + (rootItem.EndDate && rootItem.EndDate.substring(0, 4) != rootItem.ProductionYear + '' ? '-' + rootItem.EndDate.substring(0, 4) : '') }}
                                </span>
                                <span v-else>
                                    {{ rootItem.ProductionYear }} <el-tag disable-transitions>{{ rootItem.MediaSources ? formatBytes(maxMediaSources(rootItem.MediaSources)?.Size!) : 0 }}</el-tag>
                                </span>
                            </div>
                            <div style="display: flex;justify-content: space-between;">
                                <span>
                                    <el-link :underline="false" v-if="rootItem.UserData" :disabled="starLoading[embySearchItem.server.id + rootItem.Id]" @click="star(embySearchItem.server.id!, rootItem)">
                                        <el-icon color="#E6A23C" :size="24" :class="starLoading[embySearchItem.server.id + rootItem.Id] ? 'is-loading' : ''" v-if="rootItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                        <el-icon :size="24" :class="starLoading[embySearchItem.server.id + rootItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                                    </el-link>
                                    <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[embySearchItem.server.id + rootItem.Id]" v-if="rootItem.UserData" @click="played(embySearchItem.server.id!, rootItem)">
                                        <el-icon color="#67C23A" :size="24" :class="playedLoading[embySearchItem.server.id + rootItem.Id] ? 'is-loading' : ''" v-if="rootItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                        <el-icon :size="24" :class="playedLoading[embySearchItem.server.id + rootItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                    </el-link>
                                </span>
                                <span>
                                    <el-badge :value="rootItem.UserData?.UnplayedItemCount" :max="999" :show-zero="false" type="primary">
                                        <el-button v-if="rootItem.Type == 'Series'" @click="getSeasons(embySearchItem.server, rootItem)" type="primary" plain>剧集</el-button>
                                    </el-badge>
                                </span>
                            </div>
                        </el-card>
                    </div>
                    <div v-else style="text-align: center;">
                        <el-text type="danger" style="word-break: break-all;display: block;">{{ embySearchItem.message }}</el-text>
                        <el-button type="primary" @click="singleEmbySearch(embySearchItem.server)">重试</el-button>
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
                        <el-skeleton :loading="dialogSeasonsLoading" animated>
                            <template #template>
                                <div class="note-item" v-for="i in 5" :key="i">
                                    <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                                </div>
                            </template>
                            <div
                                v-for="seasonsItem in dialogSeasonsList"
                                :key="seasonsItem.Id"
                                class="note-item"
                                :class="{ active: dialogSeasons?.Id === seasonsItem.Id }"
                                @click="getEpisodes(dialogEmbyServer!, dialogSeries!.Id, seasonsItem, 1, 10)"
                            >
                                <h3>{{ 'S' + seasonsItem.IndexNumber + '. ' + seasonsItem.Name }}</h3>
                                <div style="display: flex;justify-content: space-between;">
                                    <span>
                                        <span style="margin-right: 10px;">{{ seasonsItem.ProductionYear }}</span>
                                        <el-tag type="primary" effect="dark" round disable-transitions>{{ seasonsItem.UserData?.UnplayedItemCount }}</el-tag>
                                    </span>
                                    <span>
                                        <el-link :underline="false" v-if="seasonsItem.UserData" :disabled="starLoading[dialogEmbyServer!.id + seasonsItem.Id]" @click="star(dialogEmbyServer!.id!, seasonsItem)">
                                            <el-icon color="#E6A23C" :size="24" :class="starLoading[dialogEmbyServer!.id + seasonsItem.Id] ? 'is-loading' : ''" v-if="seasonsItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                            <el-icon :size="24" :class="starLoading[dialogEmbyServer!.id + seasonsItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                                        </el-link>
                                        <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[dialogEmbyServer!.id + seasonsItem.Id]" v-if="seasonsItem.UserData" @click="played(dialogEmbyServer!.id!, seasonsItem)">
                                            <el-icon color="#67C23A" :size="24" :class="playedLoading[dialogEmbyServer!.id + seasonsItem.Id] ? 'is-loading' : ''" v-if="seasonsItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                            <el-icon :size="24" :class="playedLoading[dialogEmbyServer!.id + seasonsItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                        </el-link>
                                    </span>
                                </div>
                            </div>
                        </el-skeleton>
                    </el-scrollbar>
                </div>
                <div class="note-content">
                    <el-scrollbar>
                        <el-skeleton :loading="dialogEpisodesLoading" animated>
                            <template #template>
                                <div class="note-item" v-for="i in 5" :key="i">
                                    <p><el-skeleton-item variant="text" style="width: 50%" /></p>
                                    <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                                </div>
                            </template>
                            <div v-for="episodesItem in dialogEpisodesList" class="note-item" style="display: flex;justify-content: space-between; align-items: center;">
                                <div>
                                    <p>
                                        <el-link :underline="false" @click="gotoEpisodes(dialogEmbyServer!.id!, episodesItem.Id)">
                                            <el-badge is-dot :value="episodesItem.UserData?.Played" type="primary" :offset="[5, 0]">
                                                {{ episodesItem.IndexNumber + '. ' + episodesItem.Name }}
                                            </el-badge>
                                        </el-link>
                                    </p>
                                    <div style="display: flex;justify-content: space-between;">
                                        <span>
                                            <span style="margin-right: 10px;">{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }}</span>
                                            <el-tag disable-transitions>{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</el-tag>
                                        </span>
                                        <span>
                                            <el-link :underline="false" v-if="episodesItem.UserData" :disabled="starLoading[dialogEmbyServer!.id + episodesItem.Id]" @click="star(dialogEmbyServer!.id!, episodesItem)">
                                                <el-icon color="#E6A23C" :size="24" :class="starLoading[dialogEmbyServer!.id + episodesItem.Id] ? 'is-loading' : ''" v-if="episodesItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                                <el-icon :size="24" :class="starLoading[dialogEmbyServer!.id + episodesItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                                            </el-link>
                                            <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[dialogEmbyServer!.id + episodesItem.Id]" v-if="episodesItem.UserData" @click="played(dialogEmbyServer!.id!, episodesItem)">
                                                <el-icon color="#67C23A" :size="24" :class="playedLoading[dialogEmbyServer!.id + episodesItem.Id] ? 'is-loading' : ''" v-if="episodesItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                                <el-icon :size="24" :class="playedLoading[dialogEmbyServer!.id + episodesItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                                            </el-link>
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </el-skeleton>
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
import { computed, ref } from 'vue'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi, { EmbyPageList, EpisodesItems, SearchItems, SeasonsItems, UserData } from '../api/embyApi'
import { ElMessage } from 'element-plus'
import { formatBytes } from '../util/str_util'
import { maxMediaSources } from '../util/play_info_util'
import { useRouter } from 'vue-router'

const router = useRouter()

const search_loading = ref(false)
const search_str = ref('')
const embyServerKeys = ref<string[]>([])
const mpv_config = ref(false)

const emby_search_result = ref<{[key: string]: {server: EmbyServerConfig, success: boolean, message?: string, result?: EmbyPageList<SearchItems>}}>({})
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
const dialogSeasonsLoading = ref(false)
const dialogEpisodesLoading = ref(false)

const emby_search_result_list = computed(() => {
  return Object.entries(emby_search_result.value).map(([_key, value]) => (value)).sort((a, b) => {
        if (a.success && a.result && a.result.Items.length > 0) {
            if (b.success && b.result && b.result.Items.length > 0) {
                return 0; // 保持顺序不变
            }
            return -1; // a 排在 b 前面
        } else if (a.success && a.result && a.result.Items.length === 0) {
            if (b.success && b.result && b.result.Items.length > 0) {
                return 1; // b 排在 a 前面
            } else if (b.success && b.result && b.result.Items.length === 0) {
                return 0; // 保持顺序不变
            }
            return -1; // a 排在 b 前面
        } else {
            if (b.success) {
                return 1; // b 排在 a 前面
            }
            return 0; // 保持顺序不变
        }
    });
})

async function search() {
    search_loading.value = true
    embyServerKeys.value = []
    emby_search_result.value = {}
    let promises = []
    let config = useConfig().get_config()
    mpv_config.value = config.mpv_path ? true : false
    for (let embyServer of config.emby_server!) {
        if (!embyServer.disabled) {
            let promise = singleEmbySearch(embyServer)
            promises.push(promise)
        }
    }
    Promise.allSettled(promises).then(() => search_loading.value = false);
}
async function singleEmbySearch(embyServer: EmbyServerConfig) {
    embyServer = useConfig().getEmbyServer(embyServer.id!)!
    embyServer.request_status = true
    return embyApi.search(embyServer, search_str.value, 0, 30).then(async response => {
        if (response.status_code != 200) {
            emby_search_result.value[embyServer.id!] = {server: embyServer, success: false, message: 'response status' + response.status_code + ' ' + response.status_text}
            embyServer.request_fail = true
            return
        }
        let json: EmbyPageList<SearchItems> = JSON.parse(response.body);
        emby_search_result.value[embyServer.id!] = {server: embyServer, success: true, result: json}
        if (json.Items.length > 0) {
            embyServerKeys.value.push(embyServer.id!)
        } else {
            if (embyServerKeys.value.includes(embyServer.id!)) {
                embyServerKeys.value.splice(embyServerKeys.value.indexOf(embyServer.id!), 1)
            }
        }
        embyServer.request_fail = false
    }).catch(e => {
        emby_search_result.value[embyServer.id!] = {server: embyServer, success: false, message: e}
        embyServer.request_fail = true
    }).finally(() => embyServer.request_status = false)
}
async function getSeasons(embyServer: EmbyServerConfig, series: SearchItems) {
    embyServer = useConfig().getEmbyServer(embyServer.id!)!
    dialogSeasons.value = undefined
    dialogSeasonsLoading.value = true
    dialogSeasonsList.value = []
    dialogEpisodesList.value = []
    dialogEmbyServer.value = embyServer
    dialogSeries.value = series
    dialogSeriesVisible.value = true
    if (seasons_result.value[embyServer.id! + '|' + series.Id]) {
        dialogSeasonsList.value = seasons_result.value[embyServer.id! + '|' + series.Id].Items
        dialogSeasonsLoading.value = false
        return
    }
    return embyApi.seasons(embyServer, series.Id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<SeasonsItems> = JSON.parse(response.body);
        seasons_result.value[embyServer.id! + '|' + series.Id] = json
        dialogSeasonsList.value = json.Items
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => dialogSeasonsLoading.value = false)
}
async function getEpisodes(embyServer: EmbyServerConfig, series_id: string, seasons: SeasonsItems, currentPage: number, pageSize: number) {
    embyServer = useConfig().getEmbyServer(embyServer.id!)!
    dialogEpisodesLoading.value = true
    dialogEpisodesList.value = []
    dialogEpisodesCurrentPage.value = currentPage
    dialogEpisodesPageSize.value = pageSize
    dialogSeasons.value = seasons
    if (!episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id]) {
        episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id] = {total: 0}
    }
    if (episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id][currentPage]) {
        dialogEpisodesList.value = episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id][currentPage]
        dialogEpisodesLoading.value = false
        return
    }
    return embyApi.episodes(embyServer, series_id, seasons.Id, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id].total = json.TotalRecordCount
        episodes_result.value[embyServer.id! + '|' + series_id + '|' + seasons.Id][currentPage]= json.Items
        dialogEpisodesList.value = json.Items
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => dialogEpisodesLoading.value = false)
}
async function handleEpisodesPageChange(val: number, embyServer: EmbyServerConfig, series_id: string, seasons: SeasonsItems) {
    embyServer = useConfig().getEmbyServer(embyServer.id!)!
    await getEpisodes(embyServer, series_id, seasons, val, dialogEpisodesPageSize.value)
}

function gotoEpisodes(embyId: string, episodesId: string) {
    router.push('/nav/emby/' + embyId + '/episodes/' + episodesId)
}
function gotoSeries(embyId: string, seriesId: string) {
    router.push('/nav/emby/' + embyId + '/series/' + seriesId)
}

const starLoading = ref<{[key: string]: boolean}>({})
function star(embyServerId: string, item: SearchItems | EpisodesItems | SeasonsItems) {
    if (!item.UserData) {
        return
    }
    let embyServer = useConfig().getEmbyServer(embyServerId)!
    starLoading.value[embyServerId + item.Id] = true
    let fun;
    if (item.UserData.IsFavorite) {
        fun = embyApi.unstar(embyServer, item.Id)
    } else {
        fun = embyApi.star(embyServer, item.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: UserData = JSON.parse(response.body);
        for (let i = 0; i < emby_search_result.value[embyServer.id!].result!.Items.length; i++) {
            if (emby_search_result.value[embyServer.id!].result!.Items[i].Id == item.Id) {
                emby_search_result.value[embyServer.id!].result!.Items[i]!.UserData!.IsFavorite = json.IsFavorite
                break
            }
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => starLoading.value[embyServerId + item.Id] = false)
}

const playedLoading = ref<{[key: string]: boolean}>({})
function played(embyServerId: string, item: SearchItems | EpisodesItems | SeasonsItems) {
    if (!item.UserData) {
        return
    }
    let embyServer = useConfig().getEmbyServer(embyServerId)!
    playedLoading.value[embyServerId + item.Id] = true
    let fun;
    if (item.UserData.Played) {
        fun = embyApi.unplayed(embyServer, item.Id)
    } else {
        fun = embyApi.played(embyServer, item.Id)
    }
    return fun.then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: UserData = JSON.parse(response.body);
        for (let i = 0; i < emby_search_result.value[embyServer.id!].result!.Items.length; i++) {
            if (emby_search_result.value[embyServer.id!].result!.Items[i].Id == item.Id) {
                emby_search_result.value[embyServer.id!].result!.Items[i]!.UserData!.Played = json.Played
                break
            }
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => playedLoading.value[embyServerId + item.Id] = false)
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
  margin-bottom: 0;
}

.el-scrollbar {
  height: 100%;
}
</style>