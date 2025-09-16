<template>
    <div style="padding: 10px;">
        <el-input v-model="search_str" autofocus @keyup.enter="search">
            <template #append>
                <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-scrollbar style="height: calc(100vh - 82px); padding: 0 20px;">
        <div>
            <el-scrollbar>
                <el-skeleton :loading="mediaLibraryLoading" animated>
                    <template #template>
                        <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                            <div v-for="i in 5" :key="i" style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                <el-skeleton-item variant="image" style="width: 267px; height: 150px;" />
                                <p><el-skeleton-item variant="text" style="width: 100px" /></p>
                            </div>
                        </div>
                    </template>
                    <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                        <div v-for="item in mediaLibraryList" :key="item.Id" @click="gotoMediaLibraryItems(item.Id)" style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                            <div style="min-width: 267px; min-height: 150px;" class="loe-cover-img">
                                <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" style="max-width: 267px; max-height: 150px; cursor: pointer;" />
                            </div>
                            <span>{{ item.Name }}</span>
                        </div>
                    </div>
                </el-skeleton>
            </el-scrollbar>
        </div>
        <div v-for="mediaLibrary in mediaLibraryList">
            <div style="display: flex; align-items: baseline;">
                <h1>{{ mediaLibrary.Name }}</h1>
                <el-link type="primary" @click="gotoMediaLibraryItems(mediaLibrary.Id)" style="margin-left: 10px;">more+</el-link>
            </div>
            <el-scrollbar style="min-height: 240px;">
                <div style="display: flex;">
                    <el-skeleton :loading="mediaLibraryChildLoading[mediaLibrary.Id]" animated>
                        <template #template>
                            <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                                <div v-for="i in 8" :key="i" style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                    <el-skeleton-item variant="image" style="width: 115px; height: 160px;" />
                                    <p><el-skeleton-item variant="text" style="width: 60px" /></p>
                                </div>
                            </div>
                        </template>
                        <div style="display: flex; flex-wrap: nowrap; flex-direction: row; padding: 20px;">
                            <div v-for="item in mediaLibraryChildList[mediaLibrary.Id]" :key="item.Id"
                                @click="() => {item.Type == 'Series' ? gotoSeries(item.Id) : gotoEpisodes(item.Id)}"
                                style="display: flex; flex-direction: column; align-items: center; padding: 10px;">
                                <div style="min-width: 115px; min-height: 160px;" class="loe-cover-img">
                                    <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" style="max-height: 160px; cursor: pointer;" />
                                </div>
                                <el-text truncated style="max-width: 115px;">{{ item.Name }}</el-text>
                            </div>
                        </div>
                    </el-skeleton>
                </div>
            </el-scrollbar>
        </div>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, SearchItem, MediaLibraryItem } from '../../api/embyApi';
import { ElMessage } from 'element-plus';
import { useImage } from '../../store/image';

const router = useRouter()
const route = useRoute()

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
function gotoMediaLibraryItems(parentId: string) {
    router.push('/nav/emby/' + embyServerId + '/mediaLibrary/items/' + parentId)
}

const mediaLibraryLoading = ref(false)
const mediaLibraryList = ref<MediaLibraryItem[]>([])
function getMediaLibraryList() {
    mediaLibraryLoading.value = true
    return embyApi.getMediaLibraryList(embyServerId).then(async response => {
        let json: EmbyPageList<MediaLibraryItem> = JSON.parse(response);
        mediaLibraryList.value = json.Items
        for (let item of mediaLibraryList.value) {
            useImage().loadCover(embyServerId, item)
            getMediaLibraryChildLatest(item.Id)
        }
    }).catch(e => ElMessage.error(e)).finally(() => mediaLibraryLoading.value = false)
}
const mediaLibraryChildLoading = ref<{[key: string]: boolean}>({})
const mediaLibraryChildList = ref<{[key: string]: SearchItem[]}>({})
function getMediaLibraryChildLatest(parentId: string) {
    mediaLibraryChildLoading.value[parentId] = true
    return embyApi.getMediaLibraryChildLatest(embyServerId, parentId, 16).then(async response => {
        let json: SearchItem[] = JSON.parse(response);
        mediaLibraryChildList.value[parentId] = json
        for (let item of mediaLibraryChildList.value[parentId]) {
            useImage().loadCover(embyServerId, item)
        }
    }).catch(e => ElMessage.error(e)).finally(() => mediaLibraryChildLoading.value[parentId] = false)
}

getMediaLibraryList()
</script>

<style scoped>
</style>