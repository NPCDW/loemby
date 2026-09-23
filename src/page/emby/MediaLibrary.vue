<template>
    <div class="roe-page">
        <div class="roe-toolrow">
            <el-input v-model="search_str" autofocus @keyup.enter="search" placeholder="在这台服务器中搜索" class="query__input">
                <template #append>
                    <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
                </template>
            </el-input>
            <el-button plain @click="scrollToTop"><el-icon><i-ep-ArrowUpBold /></el-icon> 回到顶部</el-button>
        </div>

        <el-scrollbar ref="pageScrollbar" style="padding-bottom: 40px;">
            <!-- 第一层：媒体库本身，横向天桥 -->
            <div class="roe-section">
                <div class="roe-section-head">
                    <h2>媒体库</h2>
                    <span class="roe-section-count">{{ mediaLibraryList.length }}</span>
                </div>
                <el-skeleton :loading="mediaLibraryLoading" animated>
                    <template #template>
                        <div class="shelf">
                            <div v-for="i in 4" :key="i" class="shelf__cover">
                                <el-skeleton-item variant="image" style="height: 130px; width: 230px;" />
                            </div>
                        </div>
                    </template>
                    <div class="shelf">
                        <button v-for="item in mediaLibraryList" :key="item.Id" class="shelf__cover" @click="gotoMediaLibraryItems(item.Id)">
                            <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" />
                            <span class="shelf__name">{{ item.Name }}</span>
                        </button>
                    </div>
                </el-skeleton>
            </div>

            <!-- 第二层：每个电影/剧集库的最新条目 -->
            <div
                v-for="mediaLibrary in latestLibraries"
                :key="mediaLibrary.Id"
                class="roe-section"
            >
                <div class="roe-section-head">
                    <h2>{{ mediaLibrary.Name }}</h2>
                    <span class="roe-section-count">{{ (mediaLibraryChildList[mediaLibrary.Id] || []).length }} 部最新</span>
                    <button class="head-more" @click="gotoMediaLibraryItems(mediaLibrary.Id)">查看全部</button>
                </div>
                <el-skeleton :loading="mediaLibraryChildLoading[mediaLibrary.Id]" animated>
                    <template #template>
                        <div class="shelf">
                            <div v-for="i in 8" :key="i" class="shelf__poster">
                                <el-skeleton-item variant="image" style="height: 160px; width: 112px;" />
                            </div>
                        </div>
                    </template>
                    <div class="shelf">
                        <button
                            v-for="item in mediaLibraryChildList[mediaLibrary.Id]"
                            :key="item.Id"
                            class="shelf__poster"
                            @click="item.Type == 'Series' ? gotoSeries(item.Id) : gotoEpisodes(item.Id)"
                        >
                            <span class="shelf__img loe-cover-img">
                                <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" />
                            </span>
                            <span class="shelf__caption" :title="item.Name">{{ item.Name }}</span>
                        </button>
                    </div>
                </el-skeleton>
            </div>

            <div v-if="!mediaLibraryLoading && mediaLibraryList.length === 0" class="roe-empty">
                <span class="roe-empty__line">这台服务器没有可用媒体库</span>
                <span>确认账号有媒体库权限，或到设置中检查线路与代理。</span>
            </div>
        </el-scrollbar>
    </div>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, SearchItem, MediaLibraryItem } from '../../api/embyApi';
import { ElMessage, ScrollbarInstance } from 'element-plus';
import { useImage } from '../../store/image';

const router = useRouter()

/* —— 仅用于页面呈现：回到顶部 + 电影/剧集库筛选，不改动取数逻辑 —— */
const pageScrollbar = ref<ScrollbarInstance>()
function scrollToTop() {
    pageScrollbar.value!.setScrollTop(0)
}

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
/** 只有电影库与剧集库需要展开最新条目，其余库只出现在天桥里 */
const latestLibraries = computed(() => mediaLibraryList.value.filter(
    item => item.CollectionType == 'movies' || item.CollectionType == 'tvshows'
))
function getMediaLibraryList() {
    mediaLibraryLoading.value = true
    return embyApi.getMediaLibraryList(embyServerId).then(async response => {
        let json: EmbyPageList<MediaLibraryItem> = JSON.parse(response);
        mediaLibraryList.value = json.Items
        for (let item of mediaLibraryList.value) {
            useImage().loadCover(embyServerId, item)
            if (item.CollectionType == 'movies' || item.CollectionType == 'tvshows') {
                getMediaLibraryChildLatest(item.Id)
            }
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
