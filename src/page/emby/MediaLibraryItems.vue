<template>
    <div class="roe-page">
        <div class="roe-toolrow">
            <el-input v-model="search_str" autofocus @keyup.enter="search" placeholder="在这台服务器中搜索" class="query__input">
                <template #append>
                    <el-button type="primary" @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
                </template>
            </el-input>
            <span class="count" v-if="!mediaLibraryChildLoading">{{ mediaLibraryChildTotal }} 项</span>
        </div>

        <el-skeleton :loading="mediaLibraryChildLoading" animated>
            <template #template>
                <div class="wall">
                    <div v-for="i in 14" :key="i" class="wall__cell">
                        <el-skeleton-item variant="image" style="height: 160px; width: 112px;" />
                    </div>
                </div>
            </template>
            <div v-if="mediaLibraryChildList.length" class="wall">
                <button
                    v-for="item in mediaLibraryChildList"
                    :key="item.Id"
                    class="wall__cell"
                    @click="item.Type == 'BoxSet' ? gotoCurrent(item.Id) : item.Type == 'Series' ? gotoSeries(item.Id) : gotoEpisodes(item.Id)"
                >
                    <span class="wall__img loe-cover-img">
                        <img v-lazy="useImage().images[embyServerId + ':cover:' + item.Id]" />
                    </span>
                    <span class="wall__caption" :title="item.Name">{{ item.Name }}</span>
                </button>
            </div>
            <div v-else class="roe-empty">
                <span class="roe-empty__line">这个分类是空的</span>
                <span>换一个媒体库，或确认服务器上是否已完成刮削。</span>
            </div>
        </el-skeleton>

        <el-pagination
            v-model:current-page="mediaLibraryChildCurrentPage"
            v-model:page-size="mediaLibraryChildPageSize"
            layout="total, prev, pager, next, jumper"
            :total="mediaLibraryChildTotal"
            @current-change="handleMediaLibraryChildPageChange"
            hide-on-single-page
        />
    </div>
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
function gotoCurrent(folderId: string) {
    router.push('/nav/emby/' + embyServerId + '/mediaLibrary/items/' + folderId)
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
.query__input {
    flex: auto;
}

.count {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-3);
    flex: none;
}

/* 网格：等宽海报墙，间距统一，不用卡片包壳 */
.wall {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(112px, 1fr));
    gap: 18px 14px;
}

.wall__cell {
    display: flex;
    flex-direction: column;
    gap: 7px;
    padding: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
}

.wall__img {
    display: block;
    width: 100%;
    aspect-ratio: 112 / 160;
}

.wall__img img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.wall__caption {
    font-size: var(--text-sm);
    color: var(--text-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.wall__cell:hover .wall__caption {
    color: var(--text-1);
}
</style>
