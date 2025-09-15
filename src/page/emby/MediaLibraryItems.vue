<template>
    <div style="padding: 10px;">
        <el-input v-model="search_str" autofocus @keyup.enter="search">
            <template #append>
                <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-scrollbar style="height: calc(100vh - 82px); padding: 0 20px;">
        <el-skeleton :loading="mediaLibraryChildLoading" animated>
            <template #template>
                <div style="padding: 20px; display: flex; flex-wrap: wrap;">
                    <div v-for="i in 18" :key="i" style="display: flex; flex-direction: column; align-items: center; padding: 18px;">
                        <el-skeleton-item variant="image" style="width: 115px; height: 160px;" />
                        <div><el-skeleton-item variant="text" style="width: 60px" /></div>
                    </div>
                </div>
            </template>
            <div style="padding: 20px; display: flex; flex-wrap: wrap;">
                <div v-for="item in mediaLibraryChildList" :key="item.Id"
                    @click="() => {item.Type == 'Series' ? gotoSeries(item.Id) : gotoEpisodes(item.Id)}"
                    style="display: flex; flex-direction: column; align-items: center; padding: 18px;">
                    <div style="min-width: 115px; min-height: 160px;" class="loe-cover-img">
                        <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" style="max-width: 270px; max-height: 160px; cursor: pointer;" />
                    </div>
                    <el-text truncated style="max-width: 115px;">{{ item.Name }}</el-text>
                </div>
            </div>
        </el-skeleton>
        <el-pagination
            style="display: flex; justify-content: center;"
            v-model:current-page="mediaLibraryChildCurrentPage"
            v-model:page-size="mediaLibraryChildPageSize"
            layout="total, prev, pager, next, jumper"
            :total="mediaLibraryChildTotal"
            @current-change="handleMediaLibraryChildPageChange"
            hide-on-single-page
        />
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, SearchItem } from '../../api/embyApi';
import { ElMessage } from 'element-plus';
import { useImage } from '../../store/image';

const router = useRouter()
const route = useRoute()

const parentId = <string>route.params.parentId
const embyServerId = <string>route.params.embyId

const search_str = ref('')
const search = async () => {
    router.push('/nav/emby/' + embyServerId + '/search?search=' + encodeURIComponent(search_str.value))
}

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServerId + '/series/' + seriesId)
}

const mediaLibraryChildLoading = ref<boolean>(false)
const mediaLibraryChildList = ref<SearchItem[]>([])
const mediaLibraryChildCurrentPage = ref(1)
const mediaLibraryChildPageSize = ref(18)
const mediaLibraryChildTotal = ref(0)
const handleMediaLibraryChildPageChange = (val: number) => {
    mediaLibraryChildCurrentPage.value = val
    getMediaLibraryChild(val, mediaLibraryChildPageSize.value)
}
function getMediaLibraryChild(currentPage: number, pageSize: number) {
    mediaLibraryChildLoading.value = true
    return embyApi.getMediaLibraryChild(embyServerId, parentId, (currentPage - 1) * pageSize, pageSize).then(async response => {
        let json: EmbyPageList<SearchItem> = JSON.parse(response);
        mediaLibraryChildList.value = json.Items
        mediaLibraryChildTotal.value = json.TotalRecordCount
        for (let item of mediaLibraryChildList.value) {
            useImage().loadCover(embyServerId, item)
        }
    }).catch(e => ElMessage.error(e)).finally(() => mediaLibraryChildLoading.value = false)
}

handleMediaLibraryChildPageChange(1)
</script>

<style scoped>
</style>