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
                <ItemCard v-for="rootItem in emby_search_result.result?.Items" :key="rootItem.Id" :item="rootItem" :embyServerId="embyServerId" />
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
import { ref } from 'vue';
import { useRoute } from 'vue-router';
import embyApi, { EmbyPageList, SearchItem } from '../../api/embyApi';
import ItemCard from '../../components/ItemCard.vue';

const route = useRoute()

const embyServerId = <string>route.params.embyId

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
    return embyApi.search(embyServerId, search_str.value, item_types.value, 0, 30).then(async response => {
        let json: EmbyPageList<SearchItem> = JSON.parse(response);
        emby_search_result.value = {success: true, result: json}
    }).catch(e => {
        emby_search_result.value = {success: false, message: e}
    }).finally(() => search_loading.value = false)
}
search()
</script>

<style scoped>
</style>