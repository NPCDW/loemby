<template>
    <div class="page">
        <div class="toolrow">
            <el-button plain @click="gotoLibraryIndex">
                <el-icon><i-ep-Film /></el-icon>
                <span>媒体库</span>
            </el-button>
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
            <span v-if="stats" class="num">
                电影 {{ stats.MovieCount }} · 剧 {{ stats.SeriesCount }} · 单集 {{ stats.EpisodeCount }}
            </span>
        </div>

        <!-- 继续观看：宽幅横卡，是这一页的主视觉 -->
        <AppSection title="继续观看" :count="resumeTotal">
            <template #actions>
                <span v-if="dismissed.size" class="t-faint">已隐藏 {{ dismissed.size }} 条</span>
            </template>

            <AppGridSkeleton v-if="resumeLoading" :count="4" col="22rem" />

            <div v-else-if="resumeList.length" class="resume-grid">
                <ResumeCard
                    v-for="item in resumeList"
                    :key="item.Id"
                    :item="item"
                    :emby-server-id="serverId"
                    :dismissed="dismissed.has(item.Id)"
                    :pending="!!dismissPending[item.Id]"
                    @open="id => router.push(`/nav/emby/${serverId}/episodes/${id.Id}`)"
                    @dismiss="setDismissed(item.Id, true)"
                    @restore="setDismissed(item.Id, false)"
                />
            </div>

            <AppEmpty
                v-else
                title="没有未看完的内容"
                hint="播放任意影片后，进度会出现在这里，方便接着看。"
            />

            <el-pagination
                v-if="resumeTotal > pageSize"
                v-model:current-page="page"
                :page-size="pageSize"
                layout="prev, pager, next"
                :total="resumeTotal"
                @current-change="loadResume"
            />
        </AppSection>

        <!-- 收藏：只展示一行，更多交给搜索 -->
        <AppSection title="收藏" :count="favoriteTotal">
            <AppGridSkeleton v-if="favoriteLoading" :count="8" />

            <div v-else-if="favorites.length" class="grid">
                <ItemPoster
                    v-for="item in favorites"
                    :key="item.Id"
                    :item="item"
                    :emby-server-id="serverId"
                    show-series-name
                    @open="openItem"
                />
            </div>

            <AppEmpty
                v-else
                title="还没有收藏"
                hint="在影片或单集上点亮星标，就会汇总到这里。"
            />

            <el-pagination
                v-if="favoriteTotal > favoritePageSize"
                v-model:current-page="favoritePage"
                :page-size="favoritePageSize"
                layout="prev, pager, next"
                :total="favoriteTotal"
                @current-change="loadFavorites"
            />
        </AppSection>
    </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import AppSection from '../../components/base/AppSection.vue';
import AppEmpty from '../../components/base/AppEmpty.vue';
import AppGridSkeleton from '../../components/base/AppGridSkeleton.vue';
import ItemPoster from '../../components/item/ItemPoster.vue';
import ResumeCard from '../../components/item/ResumeCard.vue';
import embyApi, { type EpisodeItem, type MediaLibraryCount, type SearchItem } from '../../api/embyApi';
import { useImage } from '../../store/image';

/**
 * 服务器首页。
 *
 * 三段式：继续观看（主视觉，宽幅横卡）/ 收藏 / 统计。
 * 统计不做卡片墙，就是一行等宽数字，扫一眼就有概念。
 */
const route = useRoute();
const router = useRouter();
const image = useImage();
const serverId = route.params.embyId as string;

/* —— 继续观看 —— */
const resumeList = ref<EpisodeItem[]>([]);
const resumeTotal = ref(0);
const resumeLoading = ref(false);
const page = ref(1);
const pageSize = ref(12);

/** 已隐藏的条目留在原位变暗，可撤销——避免误点后找不回来 */
const dismissed = reactive(new Set<string>());
const dismissPending = reactive<Record<string, boolean>>({});

async function loadResume(target = 1) {
    page.value = target;
    resumeLoading.value = true;
    try {
        const result = await embyApi.getContinuePlayList(serverId, (target - 1) * pageSize.value, pageSize.value);
        resumeList.value = result.Items ?? [];
        resumeTotal.value = result.TotalRecordCount ?? 0;
        for (const item of resumeList.value) {
            await image.loadCover(serverId, item);
            if (item.Type === 'Episode') {
                await image.loadParentCover(serverId, item);
            }
        }
    } catch (e) {
        ElMessage.error('获取继续观看失败 ' + e);
    } finally {
        resumeLoading.value = false;
    }
}

async function setDismissed(itemId: string, hide: boolean) {
    dismissPending[itemId] = true;
    try {
        await embyApi.hideFromResume(serverId, itemId, hide);
        if (hide) {
            dismissed.add(itemId);
        } else {
            dismissed.delete(itemId);
        }
    } catch (e) {
        ElMessage.error('更新继续观看失败 ' + e);
    } finally {
        dismissPending[itemId] = false;
    }
}

/* —— 收藏 —— */
const favorites = ref<SearchItem[]>([]);
const favoriteTotal = ref(0);
const favoriteLoading = ref(false);
const favoritePage = ref(1);
const favoritePageSize = ref(12);

async function loadFavorites(target = 1) {
    favoritePage.value = target;
    favoriteLoading.value = true;
    try {
        const result = await embyApi.getFavoriteList(serverId, (target - 1) * favoritePageSize.value, favoritePageSize.value);
        favorites.value = result.Items ?? [];
        favoriteTotal.value = result.TotalRecordCount ?? 0;
        await image.warmCovers(serverId, favorites.value);
    } catch (e) {
        ElMessage.error('获取收藏失败 ' + e);
    } finally {
        favoriteLoading.value = false;
    }
}

/* —— 统计 —— */
const stats = ref<MediaLibraryCount | null>(null);

async function loadStats() {
    try {
        stats.value = await embyApi.count(serverId);
    } catch (e) {
        ElMessage.error('获取统计失败 ' + e);
    }
}

/* —— 导航 —— */
const keyword = ref('');

function goSearch() {
    if (!keyword.value.trim()) {
        return;
    }
    router.push(`/nav/emby/${serverId}/search?search=${encodeURIComponent(keyword.value.trim())}`);
}

function gotoLibraryIndex() {
    router.push(`/nav/emby/${serverId}/mediaLibrary`);
}

function openItem(item: SearchItem) {
    if (item.Type === 'Series') {
        router.push(`/nav/emby/${serverId}/series/${item.Id}`);
    } else {
        router.push(`/nav/emby/${serverId}/episodes/${item.Id}`);
    }
}

onMounted(() => {
    void loadResume();
    void loadFavorites();
    void loadStats();
});
</script>

<style scoped>
.toolrow__query {
    flex: 1 1 20rem;
    max-width: 32rem;
}

.resume-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(22rem, 1fr));
    gap: 0.875rem;
}
</style>
