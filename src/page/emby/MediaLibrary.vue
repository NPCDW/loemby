<template>
    <div style="padding: 10px;">
        <el-input v-model="search_str" autofocus @keyup.enter="search">
            <template #append>
                <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
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
                            <div style="min-width: 267px; min-height: 150px;">
                                <img v-lazy="images[item.Id]" style="max-width: 267px; max-height: 150px;" />
                            </div>
                            <span>{{ item.Name }}</span>
                        </div>
                    </div>
                </el-skeleton>
            </el-scrollbar>
        </div>
        <div v-for="mediaLibrary in mediaLibraryList">
            <template v-if="mediaLibraryChildList[mediaLibrary.Id] && mediaLibraryChildList[mediaLibrary.Id].length > 0">
                <h1 @click="gotoMediaLibraryItems(mediaLibrary.Id)">{{ mediaLibrary.Name }}</h1>
                <el-scrollbar>
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
                                    <div style="min-width: 115px; min-height: 160px;">
                                        <img v-lazy="images[item.Id]" style="max-width: 115px; max-height: 160px;" />
                                    </div>
                                    <el-text truncated style="max-width: 115px;">{{ item.Name }}</el-text>
                                </div>
                            </div>
                        </el-skeleton>
                    </div>
                </el-scrollbar>
            </template>
        </div>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, SearchItem, MediaLibraryItem } from '../../api/embyApi';
import { ElMessage } from 'element-plus';
import invokeApi from '../../api/invokeApi';
import { EmbyServer, useEmbyServer } from '../../store/db/embyServer';
import { useProxyServer } from '../../store/db/proxyServer';
import { useEventBus } from '../../store/eventBus';

const router = useRouter()
const route = useRoute()

const embyServer = ref<EmbyServer>({})
async function getEmbyServer(embyId: string) {
    return useEmbyServer().getEmbyServer(embyId).then(value => {
        embyServer.value = value!;
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
function embyServerChanged(payload?: {event?: string, id?: string}) {
    if (payload?.id === route.params.embyId) {
        getEmbyServer(payload?.id)
    }
}
onMounted(() => useEventBus().on('EmbyServerChanged', embyServerChanged))
onUnmounted(() => useEventBus().remove('EmbyServerChanged', embyServerChanged))

const search_str = ref('')
const search = async () => {
    router.push('/nav/emby/' + embyServer.value.id + '/search?search=' + encodeURIComponent(search_str.value))
}

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/series/' + seriesId)
}
function gotoMediaLibraryItems(parentId: string) {
    router.push('/nav/emby/' + embyServer.value.id + '/mediaLibrary/items/' + parentId)
}

const mediaLibraryLoading = ref(false)
const mediaLibraryList = ref<MediaLibraryItem[]>([])
function getMediaLibraryList() {
    mediaLibraryLoading.value = true
    return embyApi.getMediaLibraryList(embyServer.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: EmbyPageList<MediaLibraryItem> = JSON.parse(response.body);
        mediaLibraryList.value = json.Items
        for (let item of mediaLibraryList.value) {
            loadImage(item.Id)
            getMediaLibraryChildLatest(item.Id)
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => mediaLibraryLoading.value = false)
}
const mediaLibraryChildLoading = ref<{[key: string]: boolean}>({})
const mediaLibraryChildList = ref<{[key: string]: SearchItem[]}>({})
function getMediaLibraryChildLatest(parentId: string) {
    mediaLibraryChildLoading.value[parentId] = true
    return embyApi.getMediaLibraryChildLatest(embyServer.value, parentId, 16).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: SearchItem[] = JSON.parse(response.body);
        mediaLibraryChildList.value[parentId] = json
        for (let item of mediaLibraryChildList.value[parentId]) {
            loadImage(item.Id)
        }
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => mediaLibraryChildLoading.value[parentId] = false)
}

const images = ref<{[key: string]: string}>({})
async function loadImage(itemId: string) {
  images.value[itemId] = invokeApi.loadImage({
    image_url: embyApi.getImageUrl(embyServer.value, itemId)!,
    proxy_url: await useProxyServer().getBrowseProxyUrl(embyServer.value.browse_proxy_id),
    user_agent: embyServer.value.user_agent!,
    cache_prefix: ['image', embyServer.value.id!],
  })
}

getEmbyServer(<string>route.params.embyId).then(() => {
    getMediaLibraryList()
})
</script>

<style scoped>
</style>