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
            <span v-if="!loading" class="num">{{ total }} 项</span>
        </div>

        <AppGridSkeleton v-if="loading" :count="18" />

        <div v-else-if="items.length" class="grid">
            <ItemPoster
                v-for="item in items"
                :key="item.Id"
                :item="item"
                :emby-server-id="serverId"
                show-series-name
                @open="openItem"
            />
        </div>

        <AppEmpty
            v-else
            title="这个分类是空的"
            hint="换一个媒体库，或确认服务器上已经完成刮削。"
        />

        <el-pagination
            v-model:current-page="page"
            v-model:page-size="pageSize"
            layout="total, prev, pager, next, jumper"
            :total="total"
            hide-on-single-page
            @current-change="load"
        />
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import AppGridSkeleton from '../../components/base/AppGridSkeleton.vue';
import AppEmpty from '../../components/base/AppEmpty.vue';
import ItemPoster from '../../components/item/ItemPoster.vue';
import embyApi, { type SearchItem } from '../../api/embyApi';
import { useImage } from '../../store/image';

/**
 * 媒体库条目（海报墙）。
 *
 * 支持合集（BoxSet）：点进合集继续下钻，点剧集进详情，点电影/单集进播放详情。
 */
const route = useRoute();
const router = useRouter();
const image = useImage();
const serverId = route.params.embyId as string;
const parentId = route.params.parentId as string;

const items = ref<SearchItem[]>([]);
const total = ref(0);
const loading = ref(false);
const page = ref(1);
const pageSize = ref(30);

async function load(target = 1) {
    page.value = target;
    loading.value = true;
    try {
        const result = await embyApi.getMediaLibraryChild(serverId, parentId, (target - 1) * pageSize.value, pageSize.value);
        items.value = result.Items ?? [];
        total.value = result.TotalRecordCount ?? 0;
        await image.warmCovers(serverId, items.value);
    } catch (e) {
        ElMessage.error('获取条目失败 ' + e);
    } finally {
        loading.value = false;
    }
}

const keyword = ref('');

function goSearch() {
    if (!keyword.value.trim()) {
        return;
    }
    router.push(`/nav/emby/${serverId}/search?search=${encodeURIComponent(keyword.value.trim())}`);
}

function openItem(item: SearchItem) {
    if (item.Type === 'BoxSet') {
        router.push(`/nav/emby/${serverId}/mediaLibrary/items/${item.Id}`);
    } else if (item.Type === 'Series') {
        router.push(`/nav/emby/${serverId}/series/${item.Id}`);
    } else {
        router.push(`/nav/emby/${serverId}/episodes/${item.Id}`);
    }
}

onMounted(() => load(1));
</script>

<style scoped>
.toolrow__query {
    flex: 1 1 20rem;
    max-width: 32rem;
}
</style>
