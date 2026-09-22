<template>
    <div class="page">
        <div class="toolrow">
            <el-input
                v-model="keyword"
                autofocus
                placeholder="在这台服务器中搜索"
                class="toolrow__query"
                @keyup.enter="goSearch"
            >
                <template #append>
                    <el-button type="primary" @click="goSearch">
                        <el-icon><i-ep-Search /></el-icon>
                    </el-button>
                </template>
            </el-input>
            <span class="toolrow__spacer" />
            <span v-if="libraries.length" class="num">{{ libraries.length }} 个媒体库</span>
        </div>

        <!-- 第一层：媒体库本身 -->
        <LibraryShelf
            title="媒体库"
            :items="libraries"
            :emby-server-id="serverId"
            :loading="loading"
            wide
            empty-title="这台服务器没有可用媒体库"
            empty-hint="确认账号有媒体库权限，或到设置里检查线路与代理。"
            @open="item => gotoItems(item.Id)"
        />

        <!-- 第二层：电影库与剧集库各自的最新入库 -->
        <LibraryShelf
            v-for="library in expandedLibraries"
            :key="library.Id"
            :title="library.Name"
            :caption="`${(children[library.Id] || []).length} 部最新入库`"
            :items="children[library.Id] || []"
            :emby-server-id="serverId"
            :loading="!!childLoading[library.Id]"
            empty-title="这个库还没有入库内容"
            empty-hint="等服务器完成扫描与刮削后就会出现在这里。"
            @open="openItem"
        >
            <template #actions>
                <button class="ghost ghost--bare" type="button" @click="gotoItems(library.Id)">
                    查看全部
                </button>
            </template>
        </LibraryShelf>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import LibraryShelf from '../../components/media/LibraryShelf.vue';
import embyApi, { type MediaLibraryItem, type SearchItem } from '../../api/embyApi';
import { useImage } from '../../store/image';

/**
 * 媒体库总览。
 *
 * 只有电影库与剧集库会展开「最新入库」，其余类型（音乐、书籍等）
 * 只出现在天桥里——避免首页被一堆用不上的横排占满。
 */
const route = useRoute();
const router = useRouter();
const image = useImage();
const serverId = route.params.embyId as string;

const libraries = ref<MediaLibraryItem[]>([]);
const loading = ref(false);
const children = reactive<Record<string, SearchItem[]>>({});
const childLoading = reactive<Record<string, boolean>>({});

const expandedLibraries = computed(() =>
    libraries.value.filter(item => ['movies', 'tvshows'].includes(item.CollectionType)),
);

async function loadLibraries() {
    loading.value = true;
    try {
        const result = await embyApi.getMediaLibraryList(serverId);
        libraries.value = result.Items ?? [];
        await image.warmCovers(serverId, libraries.value);
        await Promise.all(expandedLibraries.value.map(library => loadLatest(library.Id)));
    } catch (e) {
        ElMessage.error('获取媒体库失败 ' + e);
    } finally {
        loading.value = false;
    }
}

async function loadLatest(parentId: string) {
    childLoading[parentId] = true;
    try {
        const items = await embyApi.getMediaLibraryChildLatest(serverId, parentId, 16);
        children[parentId] = items ?? [];
        await image.warmCovers(serverId, children[parentId]);
    } catch (e) {
        ElMessage.error('获取最新入库失败 ' + e);
    } finally {
        childLoading[parentId] = false;
    }
}

const keyword = ref('');

function goSearch() {
    if (!keyword.value.trim()) {
        return;
    }
    router.push(`/nav/emby/${serverId}/search?search=${encodeURIComponent(keyword.value.trim())}`);
}

function gotoItems(parentId: string) {
    router.push(`/nav/emby/${serverId}/mediaLibrary/items/${parentId}`);
}

function openItem(item: SearchItem | MediaLibraryItem) {
    if (item.Type === 'Series') {
        router.push(`/nav/emby/${serverId}/series/${item.Id}`);
    } else {
        router.push(`/nav/emby/${serverId}/episodes/${item.Id}`);
    }
}

onMounted(loadLibraries);
</script>

<style scoped>
.toolrow__query {
    flex: 1 1 20rem;
    max-width: 32rem;
}
</style>
