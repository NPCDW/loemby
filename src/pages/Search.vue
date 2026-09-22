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
                placeholder="在所有已启用的服务器中搜索"
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

            <span class="toolrow__spacer" />
            <span v-if="done" class="num">{{ hitCount }} 条结果</span>
        </div>

        <AppEmpty
            v-if="!done"
            title="一次搜索，覆盖全部服务器"
            hint="输入片名后回车。结果按服务器分组，某一台失败不影响其他服务器。"
        />

        <div v-else-if="hitCount === 0" class="results">
            <AppEmpty
                title="没有找到相关内容"
                hint="换个片名，或把上方类型放宽到「集」，有些内容只以单集形式入库。"
            />
        </div>

        <div v-else class="groups">
            <section v-for="group in orderedGroups" :key="group.server.id" class="group">
                <header class="group__head">
                    <ServerIcon :icon-url="group.server.icon_url" :size="18" />
                    <h2 class="group__name">{{ group.server.server_name }}</h2>
                    <span class="group__state mono" :data-state="stateOf(group)">
                        {{ stateLabel(group) }}
                    </span>
                </header>

                <AppGridSkeleton v-if="group.pending" :count="6" />

                <div v-else-if="!group.ok" class="group__error">
                    <span class="err">{{ group.message }}</span>
                    <el-button type="primary" plain size="small" @click="searchOne(group.server)">
                        重试这台服务器
                    </el-button>
                </div>

                <div v-else class="grid">
                    <ItemPoster
                        v-for="item in group.items"
                        :key="item.Id"
                        :item="item"
                        :emby-server-id="group.server.id!"
                        @open="openItem(group.server.id!, $event)"
                    />
                </div>
            </section>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import AppEmpty from '../components/base/AppEmpty.vue';
import AppGridSkeleton from '../components/base/AppGridSkeleton.vue';
import ItemPoster from '../components/item/ItemPoster.vue';
import ServerIcon from '../components/server/ServerIcon.vue';
import embyApi, { type SearchItem } from '../api/embyApi';
import { useEmbyServers } from '../composables/useEmbyServer';
import { useImage } from '../store/image';
import type { EmbyServer } from '../store/db/embyServer';

/**
 * 聚合搜索。
 *
 * 一次输入，并发打向所有已启用的服务器。结果按导航里的服务器顺序分组，
 * 每台服务器各自显示「搜索中 / N 条 / 无结果 / 请求失败」，一台挂了不拖累全局。
 */
interface Group {
    server: EmbyServer;
    pending: boolean;
    ok: boolean;
    items: SearchItem[];
    message?: string;
}

const router = useRouter();
const image = useImage();
const { servers, enabledServers } = useEmbyServers();

const keyword = ref('');
const itemTypes = ref<string[]>(['Movie', 'Series']);
const loading = ref(false);
const done = ref(false);
const groups = ref<Group[]>([]);

const hitCount = computed(() => groups.value.reduce((sum, group) => sum + group.items.length, 0));

function stateOf(group: Group): string {
    if (group.pending) return 'loading';
    if (!group.ok) return 'error';
    return 'ok';
}

function stateLabel(group: Group): string {
    if (group.pending) return '搜索中';
    if (!group.ok) return '失败';
    return group.items.length ? `${group.items.length} 条` : '无结果';
}

/** 按导航顺序输出，只保留本次搜索过的服务器 */
const orderedGroups = computed(() =>
    servers.value.map(server => groups.value.find(group => group.server.id === server.id)).filter(Boolean) as Group[],
);

async function run() {
    const query = keyword.value.trim();
    if (!query) {
        return;
    }
    done.value = true;
    // 先铺出全部服务器的「搜索中」骨架，再并发填充
    groups.value = enabledServers.value.map(server => ({
        server,
        pending: true,
        ok: true,
        items: [],
    }));
    loading.value = true;
    await Promise.allSettled(groups.value.map(group => searchOne(group.server)));
    loading.value = false;
}

async function searchOne(server: EmbyServer) {
    const group = groups.value.find(item => item.server.id === server.id);
    if (!group) {
        return;
    }
    group.pending = true;
    group.items = [];
    group.ok = true;
    group.message = undefined;
    try {
        const result = await embyApi.search(server.id!, keyword.value.trim(), itemTypes.value, 0, 30);
        group.items = result.Items ?? [];
        await image.warmCovers(server.id!, group.items);
    } catch (e) {
        group.ok = false;
        group.message = String(e);
    } finally {
        group.pending = false;
    }
}

function openItem(embyServerId: string, item: SearchItem) {
    if (item.Type === 'Series') {
        router.push(`/nav/emby/${embyServerId}/series/${item.Id}`);
    } else {
        router.push(`/nav/emby/${embyServerId}/episodes/${item.Id}`);
    }
}
</script>

<style scoped>
.types {
    flex: none;
}

.toolrow__query {
    flex: 1 1 20rem;
}

.results {
    flex: auto;
}

.groups {
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.group {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
}

.group__head {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding-bottom: 0.625rem;
    border-bottom: 1px solid var(--line);
}

.group__name {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
}

.group__state {
    color: var(--text-faint);
    font-size: var(--fs-xs);
}

.group__state[data-state='ok'] {
    color: var(--ok);
}

.group__state[data-state='error'] {
    color: var(--bad);
}

.group__state[data-state='loading'] {
    color: var(--lamp);
}

.group__error {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 0;
}
</style>
