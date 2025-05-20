<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
        
        <el-scrollbar ref="scrollbarRef" style="height: calc(100vh - 52px); padding: 0 20px;">
            <el-collapse v-model="embyServerKeys">
                <el-collapse-item :title="embySearchItem.embyServer.server_name" :name="embySearchItem.embyServer.id" :disabled="embySearchItem.result?.Items.length == 0" v-for="embySearchItem in emby_search_result_list">
                    <template #icon>
                        <span style="display: flex; align-items: center; margin: auto 18px auto auto;">
                            <el-icon v-if="embySearchItem.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                            <el-icon v-else-if="!embySearchItem.success" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon>
                            <el-icon v-else-if="embySearchItem.result?.Items.length == 0" style="color: #909399;">empty</el-icon>
                            <el-icon v-else style="color: #67C23A;"><i-ep-SuccessFilled /></el-icon>
                        </span>
                    </template>
                    <div v-if="embySearchItem.success" style="display: flex; flex-wrap: wrap; flex-direction: row;">
                        <ItemCard v-for="rootItem in embySearchItem.result?.Items" :key="rootItem.Id" :item="rootItem" :embyServer="embySearchItem.embyServer" />
                    </div>
                    <div v-else style="text-align: center;">
                        <el-text type="danger" style="word-break: break-all;display: block;">{{ embySearchItem.message }}</el-text>
                        <el-button type="primary" @click="singleEmbySearch(embySearchItem.embyServer)">重试</el-button>
                    </div>
                </el-collapse-item>
            </el-collapse>
        </el-scrollbar>
        <el-button circle style="position: absolute; bottom: 20px; right: 20px" @click="scrollbarRef!.setScrollTop(0)"><i-ep-ArrowUpBold /></el-button>
    </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'
import embyApi, { EmbyPageList, SearchItem } from '../api/embyApi'
import ItemCard from '../components/ItemCard.vue';
import { EmbyServer, useEmbyServer } from '../store/db/embyServer';
import { ElMessage, ScrollbarInstance } from 'element-plus';
import { useEventBus } from '../store/eventBus';

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
onMounted(() => useEventBus().on('EmbyServerChanged', listAllEmbyServer))
onUnmounted(() => useEventBus().remove('EmbyServerChanged', listAllEmbyServer))

const scrollbarRef = ref<ScrollbarInstance>()
const search_loading = ref(false)
const search_str = ref('')
const embyServerKeys = ref<string[]>([])

const emby_search_result = ref<{[key: string]: {embyServer: EmbyServer, request_status: boolean, success: boolean, message?: string, result?: EmbyPageList<SearchItem>}}>({})

const emby_search_result_list = computed(() => {
    const embyServersSort = embyServers.value.map(item=> item.id)
    Object.entries(emby_search_result.value).map(([_key, value]) => (value)).sort((a,b) => embyServersSort.indexOf(a.embyServer.id) - embyServersSort.indexOf(b.embyServer.id))
    return emby_search_result.value
})

async function search() {
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
    return embyApi.search(embyServer, search_str.value, 0, 30).then(async response => {
        if (response.status_code != 200) {
            emby_search_result.value[embyServer.id!] = {embyServer: embyServer, request_status: false, success: false, message: response.status_code + ' ' + response.status_text}
            return
        }
        let json: EmbyPageList<SearchItem> = JSON.parse(response.body);
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