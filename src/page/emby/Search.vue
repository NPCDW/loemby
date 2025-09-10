<template>
    <div>
        <div style="display: flex; padding: 10px;">
            <el-checkbox-group v-model="item_types" style="flex: none; margin-right: 5px;">
                <el-checkbox-button key="Movie" value="Movie">电影</el-checkbox-button>
                <el-checkbox-button key="Series" value="Series">剧</el-checkbox-button>
                <el-checkbox-button key="Episode" value="Episode">集</el-checkbox-button>
            </el-checkbox-group>
            <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="flex: auto;">
                <template #append>
                    <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
                </template>
            </el-input>
        </div>
    
        <el-scrollbar style="height: calc(100vh - 82px); padding: 0 20px;">
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
// const search_type = ref<string>('keyword')
const search_loading = ref(false)
const item_types = ref<string[]>(['Movie', 'Series'])
const emby_search_result = ref<{success: boolean, message?: string, result?: EmbyPageList<SearchItem>}>({success: true})
const search = async () => {
    if (search_str.value == '') {
        return
    }
    search_loading.value = true
    emby_search_result.value = {success: true}
    return embyApi.search(embyServer.value.id!, search_str.value, item_types.value, 0, 30).then(async response => {
        let json: EmbyPageList<SearchItem> = JSON.parse(response);
        emby_search_result.value = {success: true, result: json}
    }).catch(e => {
        emby_search_result.value = {success: false, message: e}
    }).finally(() => search_loading.value = false)
}
getEmbyServer().then(() => search())
</script>

<style scoped>
</style>