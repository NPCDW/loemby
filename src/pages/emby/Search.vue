<template>
    <div class="page">
        <div class="toolrow">
            <el-checkbox-group v-model="itemTypes" class="types">
                <el-checkbox-button value="Movie">电影</el-checkbox-button>
                <el-checkbox-button value="Series">剧</el-checkbox-button>
                <el-checkbox-button value="Episode">集</el-checkbox-button>
            </el-checkbox-group>

            <el-input
                v-model="keyword"
                autofocus
                placeholder="在这台服务器中搜索"
                class="toolrow__query"
                :disabled="loading"
                @keyup.enter="run"
            >
                <template #append>
                    <el-button type="primary" :loading="loading" @click="run">
                        <el-icon><i-ep-Search /></el-icon>
                    </el-button>
                </template>
            </el-input>
        </div>

        <AppGridSkeleton v-if="loading" :count="12" />

        <AppEmpty
            v-else-if="error"
            title="搜索失败"
            :hint="error"
        >
            <template #actions>
                <el-button type="primary" plain @click="run">重试</el-button>
            </template>
        </AppEmpty>

        <AppEmpty
            v-else-if="searched && items.length === 0"
            :title="`没有找到「${keyword}」`"
            hint="换个片名，或把上方类型放宽到「集」。"
        />

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
            title="在这台服务器中搜索"
            hint="输入片名后回车；上方可以限定只找电影、剧或单集。"
        />
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import AppEmpty from '../../components/base/AppEmpty.vue';
import AppGridSkeleton from '../../components/base/AppGridSkeleton.vue';
import ItemPoster from '../../components/item/ItemPoster.vue';
import embyApi, { type SearchItem } from '../../api/embyApi';
import { useImage } from '../../store/image';

/**
 * 服务器内搜索。
 *
 * 与「聚合搜索」的区别：这里只查一台服务器，因此不分组，
 * 直接一张海报墙，也不做失败的局部重试。
 */
const route = useRoute();
const router = useRouter();
const image = useImage();
const serverId = route.params.embyId as string;

const keyword = ref((route.query.search as string) || '');
const itemTypes = ref<string[]>(['Movie', 'Series']);
const items = ref<SearchItem[]>([]);
const loading = ref(false);
const searched = ref(false);
const error = ref('');

async function run() {
    const query = keyword.value.trim();
    if (!query) {
        return;
    }
    loading.value = true;
    error.value = '';
    searched.value = true;
    try {
        const result = await embyApi.search(serverId, query, itemTypes.value, 0, 30);
        items.value = result.Items ?? [];
        await image.warmCovers(serverId, items.value);
    } catch (e) {
        error.value = String(e);
        items.value = [];
    } finally {
        loading.value = false;
    }
}

function openItem(item: SearchItem) {
    if (item.Type === 'Series') {
        router.push(`/nav/emby/${serverId}/series/${item.Id}`);
    } else {
        router.push(`/nav/emby/${serverId}/episodes/${item.Id}`);
    }
}

onMounted(() => {
    if (keyword.value) {
        void run();
    }
});
</script>

<style scoped>
.types {
    flex: none;
}

.toolrow__query {
    flex: 1 1 20rem;
}
</style>
