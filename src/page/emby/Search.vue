<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    
        <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
            <div v-if="emby_search_result.success" style="display: flex; flex-wrap: wrap; flex-direction: row;">
                <el-card style="width: 300px; margin: 5px;" v-for="rootItem in emby_search_result.result?.Items">
                    <p>{{ rootItem.Name }}</p>
                    <p v-if="rootItem.Type == 'Series'">
                        {{ rootItem.ProductionYear + (rootItem.EndDate && rootItem.EndDate.substring(0, 4) != rootItem.ProductionYear + '' ? '-' + rootItem.EndDate.substring(0, 4) : '') }}
                        未播放：{{ rootItem.UserData?.UnplayedItemCount }}
                    </p>
                    <p v-else>
                        {{ rootItem.ProductionYear }} <el-tag disable-transitions>{{ rootItem.MediaSources ? formatBytes(maxMediaSources(rootItem.MediaSources)?.Size!) : 0 }}</el-tag>
                    </p>
                    <div style="display: flex;justify-content: space-between;">
                        <span>
                            <el-link :underline="false" v-if="rootItem.UserData" :disabled="starLoading[rootItem.Id]" @click="star(rootItem)">
                                <el-icon color="#E6A23C" :size="24" :class="starLoading[rootItem.Id] ? 'is-loading' : ''" v-if="rootItem.UserData.IsFavorite"><i-ep-StarFilled /></el-icon>
                                <el-icon :size="24" :class="starLoading[rootItem.Id] ? 'is-loading' : ''" v-else><i-ep-Star /></el-icon>
                            </el-link>
                            <el-link style="margin-left: 7px;" :underline="false" :disabled="playedLoading[rootItem.Id]" v-if="rootItem.UserData" @click="played(rootItem)">
                                <el-icon color="#67C23A" :size="24" :class="playedLoading[rootItem.Id] ? 'is-loading' : ''" v-if="rootItem.UserData.Played"><i-ep-CircleCheckFilled /></el-icon>
                                <el-icon :size="24" :class="playedLoading[rootItem.Id] ? 'is-loading' : ''" v-else><i-ep-CircleCheck /></el-icon>
                            </el-link>
                        </span>
                        <span>
                            <template v-if="rootItem.Type == 'Series'">
                                <el-button @click="gotoSeries(rootItem.Id)" type="success" plain circle><el-icon><i-ep-ArrowRightBold /></el-icon></el-button>
                            </template>
                            <el-button v-else @click="gotoEpisodes(rootItem.Id)" type="success" plain circle><el-icon><i-ep-ArrowRightBold /></el-icon></el-button>
                        </span>
                    </div>
                </el-card>
            </div>
            <div v-else style="text-align: center;">
                <el-text type="danger" style="word-break: break-all;display: block;">{{ emby_search_result.message }}</el-text>
                <el-button type="primary" @click="search()">重试</el-button>
            </div>
        </el-scrollbar>
    </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useConfig } from '../../store/config';
import embyApi, { EmbyPageList, SearchItems, UserData } from '../../api/embyApi';
import { formatBytes } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'
import { ElMessage } from 'element-plus';

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.id)!

const search_str = ref(<string>route.query.search)
const search_loading = ref(false)
const emby_search_result = ref<{success: boolean, message?: string, result?: EmbyPageList<SearchItems>}>({success: true})
const search = async () => {
    search_loading.value = true
    emby_search_result.value = {success: true}
    return embyApi.search(embyServer, search_str.value, 0, 30).then(async response => {
        if (response.status_code != 200) {
            emby_search_result.value = {success: false, message: 'response status' + response.status_code + ' ' + response.status_text}
            return
        }
        let json: EmbyPageList<SearchItems> = JSON.parse(response.body);
        emby_search_result.value = {success: true, result: json}
    }).catch(e => {
        emby_search_result.value = {success: false, message: e}
    }).finally(() => search_loading.value = false)
}
search()

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/series/' + seriesId)
}

const starLoading = ref<{[key: string]: boolean}>({})
function star(item: SearchItems) {
    if (!item.UserData) {
        return
    }
    starLoading.value[item.Id] = true
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
        for (let i = 0; i < emby_search_result.value.result!.Items.length; i++) {
            if (emby_search_result.value.result!.Items[i].Id == item.Id) {
                emby_search_result.value.result!.Items[i]!.UserData!.IsFavorite = json.IsFavorite
                break
            }
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => starLoading.value[item.Id] = false)
}

const playedLoading = ref<{[key: string]: boolean}>({})
function played(item: SearchItems) {
    if (!item.UserData) {
        return
    }
    playedLoading.value[item.Id] = true
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
        for (let i = 0; i < emby_search_result.value.result!.Items.length; i++) {
            if (emby_search_result.value.result!.Items[i].Id == item.Id) {
                emby_search_result.value.result!.Items[i]!.UserData!.Played = json.Played
                break
            }
        }
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => playedLoading.value[item.Id] = false)
}
</script>

<style scoped>
</style>