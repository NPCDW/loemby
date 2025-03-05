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
import { ref } from 'vue';
import { useRoute } from 'vue-router';
import { useConfig } from '../../store/config';
import embyApi, { EmbyPageList, SearchItems } from '../../api/embyApi';
import ItemCard from '../../components/ItemCard.vue';

const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.embyId)!

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
</script>

<style scoped>
</style>