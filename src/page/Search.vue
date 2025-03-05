<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
        
        <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
            <el-collapse v-model="embyServerKeys">
                <el-collapse-item :title="embySearchItem.embyServer.server_name" :name="embySearchItem.embyServer.id" :disabled="embySearchItem.result?.Items.length == 0" v-for="embySearchItem in emby_search_result_list">
                    <template #icon>
                        <span style="display: flex; align-items: center; margin: auto 18px auto auto;">
                            <el-icon v-if="embySearchItem.embyServer.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                            <el-icon v-else-if="embySearchItem.embyServer.request_fail" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon>
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
    </div>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi, { EmbyPageList, SearchItems } from '../api/embyApi'
import ItemCard from '../components/ItemCard.vue';

const search_loading = ref(false)
const search_str = ref('')
const embyServerKeys = ref<string[]>([])
const mpv_config = ref(false)

const emby_search_result = ref<{[key: string]: {embyServer: EmbyServerConfig, success: boolean, message?: string, result?: EmbyPageList<SearchItems>}}>({})

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
            emby_search_result.value[embyServer.id!] = {embyServer: embyServer, success: false, message: 'response status' + response.status_code + ' ' + response.status_text}
            embyServer.request_fail = true
            return
        }
        let json: EmbyPageList<SearchItems> = JSON.parse(response.body);
        emby_search_result.value[embyServer.id!] = {embyServer: embyServer, success: true, result: json}
        if (json.Items.length > 0) {
            embyServerKeys.value.push(embyServer.id!)
        } else {
            if (embyServerKeys.value.includes(embyServer.id!)) {
                embyServerKeys.value.splice(embyServerKeys.value.indexOf(embyServer.id!), 1)
            }
        }
        embyServer.request_fail = false
    }).catch(e => {
        emby_search_result.value[embyServer.id!] = {embyServer: embyServer, success: false, message: e}
        embyServer.request_fail = true
    }).finally(() => embyServer.request_status = false)
}
</script>

<style scoped>
</style>