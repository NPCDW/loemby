<template>
    <div class="page">
        <div class="toolrow">
            <el-select
                v-model="query.emby_server_id"
                clearable
                placeholder="全部服务器"
                class="toolrow__server"
                @change="reload"
            >
                <el-option
                    v-for="server in servers"
                    :key="server.id"
                    :label="server.server_name"
                    :value="server.id!"
                />
            </el-select>

            <el-input
                v-model="query.series_name"
                placeholder="按剧名筛选"
                class="toolrow__text"
                clearable
                @keyup.enter="reload"
            >
                <template #prefix><span class="key">剧</span></template>
            </el-input>

            <el-input
                v-model="query.item_name"
                placeholder="按集名筛选"
                class="toolrow__text"
                clearable
                @keyup.enter="reload"
            >
                <template #prefix><span class="key">集</span></template>
            </el-input>

            <el-button type="primary" plain @click="reload">筛选</el-button>
            <span class="toolrow__spacer" />
            <span v-if="total" class="num">共 {{ total }} 条</span>
        </div>

        <AppSpinner v-if="loading" :rows="6" />

        <AppEmpty
            v-else-if="entries.length === 0"
            title="还没有播放记录"
            hint="从左侧打开一台服务器开始播放，看过的内容会自动记录在这里。"
        />

        <div v-else class="records">
            <div
                v-for="entry in entries"
                :key="entry.id"
                class="record"
                :class="{ 'is-pinned': !!entry.pinned }"
            >
                <button
                    class="record__pin"
                    type="button"
                    :class="{ 'is-on': !!entry.pinned }"
                    :title="entry.pinned ? '取消置顶' : '置顶这条记录'"
                    @click="togglePin(entry)"
                >
                    <svg-icon :name="entry.pinned ? 'pin' : 'unpin'" size="15" :color="entry.pinned ? '#F2A13B' : 'currentColor'" />
                </button>

                <span class="record__server t-clip">{{ entry.emby_server_name || '—' }}</span>

                <span class="record__main">
                    <button
                        v-if="entry.series_id"
                        class="record__link t-clip"
                        type="button"
                        @click="goSeries(entry)"
                    >
                        {{ entry.series_name || entry.item_name }}
                    </button>
                    <span v-else class="record__plain t-clip">{{ entry.series_name || '—' }}</span>
                    <el-icon class="record__arrow" :size="12"><i-ep-DArrowRight /></el-icon>
                    <button
                        v-if="entry.item_id"
                        class="record__link t-clip"
                        type="button"
                        @click="goEpisode(entry)"
                    >
                        {{ entry.item_name }}
                    </button>
                </span>

                <span class="record__duration num">{{ formatDuration(entry.played_duration ?? 0) }}</span>
            </div>

            <el-pagination
                v-model:current-page="page"
                v-model:page-size="pageSize"
                layout="total, prev, pager, next, jumper"
                :total="total"
                hide-on-single-page
                @current-change="load"
            />
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import AppEmpty from '../components/base/AppEmpty.vue';
import AppSpinner from '../components/base/AppSpinner.vue';
import { usePlayHistory, type PagePlayHistoryParam, type PlayHistory } from '../store/db/playHistory';
import { useEmbyServers } from '../composables/useEmbyServer';
import { formatDuration } from '../util/format';

/**
 * 播放历史。
 *
 * 每一行是「服务器 · 剧 → 集 · 时长」，置顶用左侧琥珀灯条而不是整行改色，
 * 这样置顶与未读的视觉语义不会打架。
 */
const router = useRouter();
const { servers } = useEmbyServers();

const entries = ref<PlayHistory[]>([]);
const total = ref(0);
const loading = ref(false);
const page = ref(1);
const pageSize = ref(30);

const query = ref<PagePlayHistoryParam>({ page_number: 1, page_size: 30 });

function cleanQuery(): PagePlayHistoryParam {
    return {
        page_number: page.value,
        page_size: pageSize.value,
        emby_server_id: query.value.emby_server_id || undefined,
        series_name: query.value.series_name || undefined,
        item_name: query.value.item_name || undefined,
    };
}

async function load(target = 1) {
    page.value = target;
    loading.value = true;
    try {
        const [count, list] = await usePlayHistory().pagePlayHistory(cleanQuery());
        total.value = count;
        entries.value = list;
    } catch (e) {
        ElMessage.error('获取播放历史失败 ' + e);
    } finally {
        loading.value = false;
    }
}

function reload() {
    return load(1);
}

async function togglePin(entry: PlayHistory) {
    const pinned = entry.pinned ? 0 : 1;
    try {
        await usePlayHistory().updatePlayHistory({ id: entry.id, pinned });
        entry.pinned = pinned;
    } catch (e) {
        ElMessage.error('更新置顶失败 ' + e);
    }
}

function goSeries(entry: PlayHistory) {
    if (entry.emby_server_id && entry.series_id) {
        router.push(`/nav/emby/${entry.emby_server_id}/series/${entry.series_id}`);
    }
}

function goEpisode(entry: PlayHistory) {
    if (entry.emby_server_id && entry.item_id) {
        router.push(`/nav/emby/${entry.emby_server_id}/episodes/${entry.item_id}`);
    }
}

onMounted(reload);
</script>

<style scoped>
.toolrow__server {
    width: 12rem;
    flex: none;
}

.toolrow__text {
    width: 12.5rem;
    flex: none;
}

.key {
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.records {
    display: flex;
    flex-direction: column;
}

.record {
    position: relative;
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.625rem 0.5rem;
    border-bottom: 1px solid var(--line);
    transition: background var(--dur-fast) var(--ease);
}

.record:hover {
    background: var(--surface-hover);
}

.record.is-pinned {
    box-shadow: inset 2px 0 0 var(--lamp);
}

.record__pin {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border: none;
    border-radius: var(--r-xs);
    background: transparent;
    color: var(--text-ghost);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
}

.record__pin:hover,
.record__pin.is-on {
    color: var(--lamp);
}

.record__server {
    flex: none;
    width: 9rem;
    color: var(--text-faint);
    font-size: var(--fs-sm);
}

.record__main {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: auto;
    min-width: 0;
}

.record__link {
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-md);
    text-align: left;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
}

.record__link:hover {
    color: var(--lamp);
}

.record__plain {
    color: var(--text-dim);
}

.record__arrow {
    flex: none;
    color: var(--text-ghost);
}

.record__duration {
    flex: none;
    width: 5.5rem;
    color: var(--text-faint);
    font-size: var(--fs-sm);
    text-align: right;
}
</style>
