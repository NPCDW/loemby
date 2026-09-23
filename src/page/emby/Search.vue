<template>
    <div class="roe-page">
        <div class="roe-toolrow">
            <el-checkbox-group v-model="item_types">
                <el-checkbox-button value="Movie">电影</el-checkbox-button>
                <el-checkbox-button value="Series">剧</el-checkbox-button>
                <el-checkbox-button value="Episode">集</el-checkbox-button>
            </el-checkbox-group>
            <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" placeholder="在这台服务器中搜索" class="query__input">
                <template #append>
                    <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
                </template>
            </el-input>
        </div>

        <div v-if="!emby_search_result.success" class="roe-empty">
            <span class="roe-empty__line">搜索失败</span>
            <span class="error-text">{{ emby_search_result.message }}</span>
            <el-button type="primary" plain @click="search()">重试</el-button>
        </div>

        <div v-else-if="emby_search_result.result && emby_search_result.result.Items.length === 0" class="roe-empty">
            <span class="roe-empty__line">没有找到「{{ search_str }}」</span>
            <span>换个片名，或把上方类型放宽到「集」。</span>
        </div>

        <div v-else class="grid">
            <ItemCard v-for="rootItem in emby_search_result.result?.Items" :key="rootItem.Id" :item="rootItem" :embyServerId="embyServerId" />
        </div>
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
.query__input {
    flex: auto;
}

.grid {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
}

.error-text {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    word-break: break-all;
}
</style>
