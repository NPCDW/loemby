<template>
    <div class="roe-page">
        <div class="roe-toolrow">
            <el-checkbox-group v-model="item_types">
                <el-checkbox-button value="Movie">电影</el-checkbox-button>
                <el-checkbox-button value="Series">剧</el-checkbox-button>
                <el-checkbox-button value="Episode">集</el-checkbox-button>
            </el-checkbox-group>
            <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" placeholder="在所有服务器中搜索" class="query__input">
                <template #append>
                    <el-button type="primary" :loading="search_loading" @click="search">
                        <el-icon><i-ep-Search /></el-icon>
                    </el-button>
                </template>
            </el-input>
        </div>

        <div v-if="Object.keys(emby_search_result_list).length === 0" class="roe-empty">
            <span class="roe-empty__line">一次搜索，覆盖全部服务器</span>
            <span>输入片名后按回车，结果会按服务器分组显示。</span>
        </div>

        <el-scrollbar ref="scrollbarRef" v-else>
            <el-collapse v-model="embyServerKeys">
                <el-collapse-item
                    v-for="entry in emby_search_result_list"
                    :key="entry.embyServer.id"
                    :name="entry.embyServer.id"
                    :disabled="entry.result?.Items.length == 0"
                >
                    <template #title>
                        <span class="group__title">{{ entry.embyServer.server_name }}</span>
                        <span class="group__state" :data-state="entry.request_status ? 'loading' : entry.success ? 'ok' : 'error'">
                            {{ entry.request_status ? '搜索中' : !entry.success ? '请求失败' : (entry.result?.Items.length ? entry.result!.Items.length + ' 条' : '无结果') }}
                        </span>
                    </template>
                    <div v-if="entry.success" class="grid">
                        <ItemCard v-for="rootItem in entry.result?.Items" :key="rootItem.Id" :item="rootItem" :embyServerId="entry.embyServer.id!" />
                    </div>
                    <div v-else class="group__error">
                        <span>{{ entry.message }}</span>
                        <el-button type="primary" plain size="small" @click="singleEmbySearch(entry.embyServer)">重试</el-button>
                    </div>
                </el-collapse-item>
            </el-collapse>
        </el-scrollbar>
    </div>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue'
import embyApi, { EmbyPageList, SearchItem } from '../api/embyApi'
import ItemCard from '../components/ItemCard.vue';
import { EmbyServer, useEmbyServer } from '../store/db/embyServer';
import { ElMessage, ScrollbarInstance } from 'element-plus';

const embyServers = ref<EmbyServer[]>([])
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
        embyServers.value.forEach(item => {
            if (emby_search_result.value[item.id!]) {
                emby_search_result.value[item.id!].embyServer = item
            }
        })
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
listAllEmbyServer()

const scrollbarRef = ref<ScrollbarInstance>()
const search_loading = ref(false)
const search_str = ref('')
const embyServerKeys = ref<string[]>([])
const item_types = ref<string[]>(['Movie', 'Series'])

const emby_search_result = ref<{[key: string]: {embyServer: EmbyServer, request_status: boolean, success: boolean, message?: string, result?: EmbyPageList<SearchItem>}}>({})

const emby_search_result_list = computed(() => {
    const embyServersSort = embyServers.value.map(item=> item.id)
    Object.entries(emby_search_result.value).map(([_key, value]) => (value)).sort((a,b) => embyServersSort.indexOf(a.embyServer.id) - embyServersSort.indexOf(b.embyServer.id))
    return emby_search_result.value
})

async function search() {
    if (search_str.value == '') {
        return
    }
    embyServerKeys.value = []
    emby_search_result.value = {}
    let promises = []

    search_loading.value = true
    for (let embyServer of embyServers.value) {
        if (!embyServer.disabled) {
            let promise = singleEmbySearch(embyServer)
            promises.push(promise)
        }
    }
    Promise.allSettled(promises).then(() => search_loading.value = false);
}
async function singleEmbySearch(embyServer: EmbyServer) {
    emby_search_result.value[embyServer.id!] = {embyServer: embyServer, request_status: true, success: false}
    return embyApi.search(embyServer.id!, search_str.value, item_types.value, 0, 30).then(async response => {
        let json: EmbyPageList<SearchItem> = JSON.parse(response);
        emby_search_result.value[embyServer.id!] = {embyServer: embyServer, request_status: false, success: true, result: json}
        if (json.Items.length > 0) {
            embyServerKeys.value.push(embyServer.id!)
        } else {
            if (embyServerKeys.value.includes(embyServer.id!)) {
                embyServerKeys.value.splice(embyServerKeys.value.indexOf(embyServer.id!), 1)
            }
        }
    }).catch(e => {
        emby_search_result.value[embyServer.id!] = {embyServer: embyServer, request_status: false, success: false, message: e}
    })
}
</script>

<style scoped>
</style>
