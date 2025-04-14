<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    
        <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
            <div v-if="emby_search_result.success" style="display: flex; flex-wrap: wrap; flex-direction: row;">
                <ItemCard v-for="rootItem in emby_search_result.result?.Items" :key="rootItem.Id" :item="rootItem" :embyServer="embyServer" />
            </div>
            <div v-else style="text-align: center;">
                <el-text type="danger" style="word-break: break-all;display: block;">{{ emby_search_result.message }}</el-text>
                <el-button type="primary" @click="search()">重试</el-button>
            </div>
            <div v-if="emby_search_result.success && emby_search_result.result?.Items.length == 0" style="text-align: center;">
                <el-empty :image-size="200" />
            </div>
        </el-scrollbar>
    </div>
</template>

<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import embyApi, { EmbyPageList, SearchItem } from '../../api/embyApi';
import ItemCard from '../../components/ItemCard.vue';
import { EmbyServer, useEmbyServer } from '../../store/db/embyServer';
import { ElMessage } from 'element-plus';
import { useEventBus } from '../../store/eventBus';

const route = useRoute()

const embyServer = ref<EmbyServer>({})
async function getEmbyServer() {
    return useEmbyServer().getEmbyServer(<string>route.params.embyId).then(value => {
        embyServer.value = value!;
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
function embyServerChanged(payload?: {event?: string, id?: string}) {
    if (payload?.id === route.params.embyId) {
        getEmbyServer()
    }
}
onMounted(() => useEventBus().on('EmbyServerChanged', embyServerChanged))
onUnmounted(() => useEventBus().remove('EmbyServerChanged', embyServerChanged))

const search_str = ref(<string>route.query.search)
const search_loading = ref(false)
const emby_search_result = ref<{success: boolean, message?: string, result?: EmbyPageList<SearchItem>}>({success: true})
const search = async () => {
    search_loading.value = true
    emby_search_result.value = {success: true}
    return embyApi.search(embyServer.value, search_str.value, 0, 30).then(async response => {
        if (response.status_code != 200) {
            emby_search_result.value = {success: false, message: response.status_code + ' ' + response.status_text}
            return
        }
        let json: EmbyPageList<SearchItem> = JSON.parse(response.body);
        emby_search_result.value = {success: true, result: json}
    }).catch(e => {
        emby_search_result.value = {success: false, message: e}
    }).finally(() => search_loading.value = false)
}
getEmbyServer().then(() => search())
</script>

<style scoped>
</style>