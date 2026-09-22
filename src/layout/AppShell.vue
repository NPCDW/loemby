<template>
    <div class="shell">
        <div class="shell__body">
            <ServerRail
                :version="appVersion"
                :servers="servers"
                :active="route.path"
                @navigate="router.push"
                @add="openAddServer"
                @config-line="openLineConfig"
                @edit-icon="openIconPicker"
                @edit-server="openEditServer"
                @toggle-enabled="toggleEnabled"
                @relogin="relogin"
                @logout="logoutServer"
                @remove="removeServer"
                @reorder="reorder"
            />

            <main class="board">
                <ServerTopBar :title="pageTitle" :server="currentServer">
                    <ServerToolbar
                        v-if="isServerContext"
                        :line-id="currentServer?.line_id"
                        :line="currentLine"
                        :lines="linesOfCurrentServer"
                        :proxies="proxies"
                        :reverse-proxies="reverseProxies"
                        :global-browse-name="globalBrowseName"
                        :global-play-name="globalPlayName"
                        @change-line="selectLineById"
                        @change-proxy="applyProxyChange"
                        @config-line="openLineConfig(currentServer!)"
                    />
                </ServerTopBar>

                <el-scrollbar class="board__scroll" ref="boardScroll">
                    <router-view v-slot="{ Component, route: current }">
                        <transition name="page" mode="out-in">
                            <keep-alive :include="KEEP_ALIVE">
                                <component :is="Component" :key="current.fullPath" />
                            </keep-alive>
                        </transition>
                    </router-view>
                </el-scrollbar>
            </main>
        </div>

        <footer class="statusbar">
            <div class="statusbar__group">
                <NotifyCenter :visible="notifyVisible" @toggle="toggleNotify" @close="notifyVisible = false" />
                <span class="num">{{ servers.length }} 台服务器</span>
            </div>
            <div class="statusbar__group">
                <span class="num">loemby v{{ appVersion }}</span>
            </div>
        </footer>

        <ServerDialogs
            v-model:add-step="addStep"
            v-model:edit-server="editServer"
            v-model:line-server="lineServer"
            v-model:line-draft="lineDraft"
            v-model:icon-server="iconServer"
            v-model:icon-library-id="iconLibraryId"
            v-model:icon-keyword="iconKeyword"
            :lines="linesOfCurrentServer"
            :proxies="proxies"
            :reverse-proxies="reverseProxies"
            :global-browse-name="globalBrowseName"
            :global-play-name="globalPlayName"
            :server-info-loading="serverInfoLoading"
            :probing="probing"
            :authenticating="authenticating"
            :icon-libraries="iconLibraries"
            :icons="iconChoices"
            :icon-loading="iconLoading"
            :server-draft="serverDraft"
            @probe="probeAndAdvance"
            @authenticate="authenticateAndFinish"
            @fetch-info="fetchServerInfo"
            @save-edit="draft => saveEditServer(draft)"
            @add-line="startAddLine"
            @edit-line="startEditLine"
            @remove-line="removeLine"
            @select-line="selectLineByObject"
            @save-line="draft => saveLine(draft)"
            @switch-icon-library="loadIcons"
            @pick-icon="applyIcon"
        />
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage, ElMessageBox, type ScrollbarInstance } from 'element-plus';
import { listen } from '@tauri-apps/api/event';

import ServerRail from '../components/server/ServerRail.vue';
import ServerTopBar from '../components/server/ServerTopBar.vue';
import ServerToolbar from '../components/server/ServerToolbar.vue';
import ServerDialogs, { type IconChoice } from '../components/server/ServerDialogs.vue';
import NotifyCenter from '../components/server/NotifyCenter.vue';

import embyApi from '../api/embyApi';
import appApi from '../api/appApi';
import invokeApi from '../api/invokeApi';
import { generateGuid } from '../util/uuid_util';
import { useEventBus } from '../store/eventBus';
import { useImage } from '../store/image';
import { useGlobalConfig } from '../store/db/globalConfig';
import { useProxyServer, type ProxyServer } from '../store/db/proxyServer';
import { useReverseProxyServer, type ReverseProxyServer } from '../store/db/reverseProxyServer';
import { useEmbyServer, type EmbyServer } from '../store/db/embyServer';
import { useEmbyLine, type EmbyLine } from '../store/db/embyLine';
import { useEmbyIconLibrary, type EmbyIconLibrary } from '../store/db/embyIconLibrary';
import { useEmbyServers } from '../composables/useEmbyServer';

/**
 * 应用外壳。
 *
 * 职责严格限定为三件事：
 *  1) 组装骨架（左导航 / 顶栏 / 内容区 / 状态栏）
 *  2) 提供服务器上下文（当前服务器、线路、代理）
 *  3) 承载服务器级别的管理动作（增删改、登录、排序、图标、线路）
 *
 * 页面的取数与排版全部下沉到 pages/*，这里不再出现任何页面细节。
 */

const KEEP_ALIVE = ['HistoryPage', 'AggregateSearchPage', 'MediaLibraryPage', 'LibraryItemsPage', 'ServerSearchPage', 'SeriesPage'];
const appVersion = (import.meta.env.VITE_APP_VERSION as string) || '';

const route = useRoute();
const router = useRouter();
const bus = useEventBus();
const image = useImage();
const { servers, serverMap, refresh: refreshServers } = useEmbyServers();

/* ————————————————— 服务器上下文 ————————————————— */

const isServerContext = computed(() => route.path.startsWith('/nav/emby/'));
const currentServer = computed(() => (isServerContext.value ? serverMap.value[route.params.embyId as string] : null));

const pageTitles: Record<string, string> = {
    '/nav/history': '播放历史',
    '/nav/search': '聚合搜索',
    '/nav/setting': '设置',
};

const pageTitle = computed(() => {
    const path = route.path;
    if (pageTitles[path]) {
        return pageTitles[path];
    }
    if (path.includes('/mediaLibrary/items/')) return '媒体库条目';
    if (path.includes('/mediaLibrary')) return '媒体库';
    if (path.includes('/episodes/')) return '播放详情';
    if (path.includes('/series/')) return '剧集详情';
    if (path.includes('/search')) return '服务器内搜索';
    if (isServerContext.value) return '服务器首页';
    return 'loemby';
});

/* ————————————————— 线路与代理 ————————————————— */

const lines = ref<EmbyLine[]>([]);
const proxies = ref<ProxyServer[]>([]);
const reverseProxies = ref<ReverseProxyServer[]>([]);
const globalBrowseName = ref('不使用代理');
const globalPlayName = ref('不使用代理');

const linesOfCurrentServer = computed(() =>
    currentServer.value ? lines.value.filter(line => line.emby_server_id === currentServer.value!.id) : [],
);

const currentLine = computed(() =>
    linesOfCurrentServer.value.find(line => line.id === currentServer.value?.line_id),
);

async function reloadLines() {
    try {
        lines.value = await useEmbyLine().listAllEmbyLine();
    } catch (e) {
        ElMessage.error('获取线路失败 ' + e);
    }
}

async function reloadProxies() {
    try {
        proxies.value = await useProxyServer().listAllProxyServer();
        reverseProxies.value = await useReverseProxyServer().listAllReverseProxyServer();
    } catch (e) {
        ElMessage.error('获取代理列表失败 ' + e);
    }
}

async function reloadGlobalProxyNames() {
    const config = useGlobalConfig();
    try {
        const browseId = await config.getGlobalConfigValue('global_browse_proxy_id');
        const playId = await config.getGlobalConfigValue('global_play_proxy_id');
        globalBrowseName.value = await useProxyServer().getProxyServerName(browseId);
        globalPlayName.value = await useProxyServer().getProxyServerName(playId);
    } catch (e) {
        ElMessage.error('获取全局代理失败 ' + e);
    }
}

/** 顶栏切线路：把该线路的地址与代理同步到服务器记录上 */
async function selectLineById(lineId: string) {
    const line = lines.value.find(item => item.id === lineId);
    if (line) {
        await selectLineByObject(line);
    }
}

async function selectLineByObject(line: EmbyLine) {
    if (lineServer.value) {
        // 弹窗内的选择：只改草稿
        lineServer.value.line_id = line.id;
        return;
    }
    if (!currentServer.value || currentServer.value.line_id === line.id) {
        return;
    }
    try {
        await useEmbyServer().updateEmbyServer({
            id: currentServer.value.id,
            line_id: line.id,
            base_url: line.base_url,
            reverse_proxy_id: line.reverse_proxy_id,
            browse_proxy_id: line.browse_proxy_id,
            play_proxy_id: line.play_proxy_id,
        });
        bus.emit('EmbyServerChanged', { event: 'update', id: currentServer.value.id });
        ElMessage.success('已切换线路');
    } catch (e) {
        ElMessage.error('切换线路失败 ' + e);
    }
}

/** 顶栏改代理：写回线路，并在该线路正在使用时就地更新服务器 */
async function applyProxyChange(payload: { key: 'reverse_proxy_id' | 'browse_proxy_id' | 'play_proxy_id'; value: string }) {
    const line = currentLine.value;
    if (!line) {
        return;
    }
    const next: EmbyLine = { ...line, [payload.key]: payload.value };
    // 选了反代就默认不再走代理，避免两条链路叠加
    if (payload.key === 'reverse_proxy_id' && payload.value !== 'no') {
        next.browse_proxy_id = 'no';
        next.play_proxy_id = 'no';
    }
    try {
        await useEmbyLine().updateEmbyLine(next);
        await reloadLines();
        const active = currentServer.value;
        if (active && active.line_id === next.id) {
            await useEmbyServer().updateEmbyServer({
                id: active.id,
                reverse_proxy_id: next.reverse_proxy_id,
                browse_proxy_id: next.browse_proxy_id,
                play_proxy_id: next.play_proxy_id,
            });
            bus.emit('EmbyServerChanged', { event: 'update', id: active.id });
        }
        ElMessage.success('已更新代理');
    } catch (e) {
        ElMessage.error('更新代理失败 ' + e);
    }
}

/* ————————————————— 服务器增删改 ————————————————— */

/**
 * 添加向导的服务器草稿。弹窗与外壳共享这一份对象，改动即时可见。
 */
const serverDraft = ref<EmbyServer>({});

const addStep = ref(0);
const probing = ref(false);
const authenticating = ref(false);
const editServer = ref<EmbyServer | null>(null);
const serverInfoLoading = ref(false);

async function openAddServer() {
    addStep.value = 1;
    try {
        const hostname = await invokeApi.getSysInfo();
        await useEmbyServer().deferOrder();
        serverDraft.value = {
            id: generateGuid(),
            server_name: '未命名服务器',
            disabled: 1,
            keep_alive_days: 0,
            client: 'loemby',
            client_version: appVersion,
            device: hostname,
            device_id: hostname,
            user_agent: `loemby/${appVersion}`,
            order_by: 1,
            reverse_proxy_id: 'no',
            browse_proxy_id: 'follow',
            play_proxy_id: 'follow',
        };
        // 向导第一步先落一条线路与服务器记录，后续凭 id 更新
        await persistDraftServer();
    } catch (e) {
        ElMessage.error('准备添加服务器失败 ' + e);
        addStep.value = 0;
    }
}

async function persistDraftServer() {
    const draft = serverDraft.value;
    const line: EmbyLine = {
        id: generateGuid(),
        name: '线路一',
        base_url: draft.base_url,
        emby_server_id: draft.id,
        emby_server_name: draft.server_name,
        reverse_proxy_id: draft.reverse_proxy_id,
        browse_proxy_id: draft.browse_proxy_id,
        play_proxy_id: draft.play_proxy_id,
    };
    await useEmbyLine().addEmbyLine(line);
    draft.line_id = line.id;
    await useEmbyServer().addEmbyServer(draft);
    await reloadLines();
}

async function probeAndAdvance(draft: EmbyServer) {
    if (!draft.base_url) {
        ElMessage.warning('先填写服务器地址');
        return;
    }
    probing.value = true;
    try {
        // 地址可能被改过，先同步到线路再探测
        const line = lines.value.find(item => item.id === draft.line_id);
        if (line && line.base_url !== draft.base_url) {
            await useEmbyLine().updateEmbyLine({
                ...line,
                base_url: draft.base_url,
                reverse_proxy_id: draft.reverse_proxy_id,
                browse_proxy_id: draft.browse_proxy_id,
                play_proxy_id: draft.play_proxy_id,
            });
            await reloadLines();
        }
        await useEmbyServer().updateEmbyServer({
            id: draft.id,
            base_url: draft.base_url,
            reverse_proxy_id: draft.reverse_proxy_id,
            browse_proxy_id: draft.browse_proxy_id,
            play_proxy_id: draft.play_proxy_id,
        });
        const info = await embyApi.getServerInfo(draft.id!);
        draft.server_name = info.ServerName;
        draft.server_id = info.Id;
        await useEmbyServer().updateEmbyServer({ id: draft.id, server_name: info.ServerName, server_id: info.Id });
        await useEmbyLine().updateEmbyServerName(draft.id!, info.ServerName);
        await reloadLines();
        addStep.value = 2;
    } catch (e) {
        ElMessage.error('连接服务器失败，请检查地址、反代与代理配置：' + e);
    } finally {
        probing.value = false;
    }
}

async function authenticateAndFinish(draft: EmbyServer) {
    if (!draft.username) {
        ElMessage.warning('至少填写用户名');
        return;
    }
    authenticating.value = true;
    try {
        await useEmbyServer().updateEmbyServer(draft);
        const auth = await embyApi.authenticateByName(draft.id!);
        await useEmbyServer().updateEmbyServer({
            id: draft.id,
            auth_token: auth.AccessToken,
            user_id: auth.User.Id,
            disabled: 0,
            server_name: draft.server_name,
        });
        await refreshServers();
        ElMessage.success('已登录');
        addStep.value = 3;
    } catch (e) {
        ElMessage.error('登录失败 ' + e);
    } finally {
        authenticating.value = false;
    }
}

function openEditServer(server: EmbyServer) {
    editServer.value = server;
}

function fetchServerInfo() {
    const target = editServer.value;
    if (!target?.id) {
        return;
    }
    serverInfoLoading.value = true;
    embyApi
        .getServerInfo(target.id)
        .then(info => {
            if (editServer.value) {
                editServer.value.server_name = info.ServerName;
                editServer.value.server_id = info.Id;
            }
        })
        .catch(e => ElMessage.error('获取服务器信息失败 ' + e))
        .finally(() => (serverInfoLoading.value = false));
}

async function saveEditServer(target: EmbyServer) {
    try {
        const line = lines.value.find(item => item.id === target.line_id);
        if (line) {
            await useEmbyLine().updateEmbyLine({
                ...line,
                base_url: target.base_url,
                reverse_proxy_id: target.reverse_proxy_id,
                browse_proxy_id: target.browse_proxy_id,
                play_proxy_id: target.play_proxy_id,
            });
            if (line.emby_server_name !== target.server_name) {
                await useEmbyLine().updateEmbyServerName(target.id!, target.server_name!);
            }
            await reloadLines();
        }
        await useEmbyServer().updateEmbyServer(target);
        bus.emit('EmbyServerChanged', { event: 'update', id: target.id });
        ElMessage.success('已保存');
        editServer.value = null;
    } catch (e) {
        ElMessage.error('保存失败 ' + e);
    }
}

async function toggleEnabled(server: EmbyServer) {
    if (!server.auth_token && server.disabled) {
        ElMessage.error('请先完成登录');
        return;
    }
    try {
        await useEmbyServer().updateEmbyServer({ id: server.id, disabled: 1 - (server.disabled ?? 0) });
        bus.emit('EmbyServerChanged', { event: 'update', id: server.id });
    } catch (e) {
        ElMessage.error('切换启用状态失败 ' + e);
    }
}

function relogin(server: EmbyServer) {
    ElMessageBox.confirm(`重新登录「${server.server_name}」？`, '重新登录', {
        confirmButtonText: '登录',
        cancelButtonText: '取消',
    })
        .then(async () => {
            try {
                await useEmbyServer().updateEmbyServer(server);
                const auth = await embyApi.authenticateByName(server.id!);
                await useEmbyServer().updateEmbyServer({
                    id: server.id,
                    auth_token: auth.AccessToken,
                    user_id: auth.User.Id,
                    disabled: 0,
                });
                bus.emit('EmbyServerChanged', { event: 'update', id: server.id });
                ElMessage.success('登录成功');
            } catch (e) {
                ElMessage.error('登录失败 ' + e);
            }
        })
        .catch(() => undefined);
}

function logoutServer(server: EmbyServer) {
    ElMessageBox.confirm(`退出登录「${server.server_name}」？媒体库将不可用，直到重新登录。`, '退出登录', {
        confirmButtonText: '退出',
        cancelButtonText: '取消',
        type: 'warning',
    })
        .then(async () => {
            try {
                await embyApi.logout(server.id!);
                await useEmbyServer().updateEmbyServer({ id: server.id, auth_token: '', disabled: 1 });
                bus.emit('EmbyServerChanged', { event: 'update', id: server.id });
                ElMessage.success('已退出登录');
            } catch (e) {
                ElMessage.error('退出登录失败 ' + e);
            }
        })
        .catch(() => undefined);
}

function removeServer(server: EmbyServer) {
    ElMessageBox.confirm(`删除「${server.server_name}」？该服务器的线路与缓存记录会一并清理。`, '删除服务器', {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
    })
        .then(async () => {
            try {
                await useEmbyServer().delEmbyServer(server.id!);
                await useEmbyLine().delEmbyServer(server.id!).catch(() => bus.emit('EmbyLineChanged', {}));
                bus.emit('EmbyServerChanged', { event: 'del', id: server.id });
                if (route.path.includes(server.id!)) {
                    await router.push('/nav/history');
                }
                ElMessage.success('已删除');
            } catch (e) {
                ElMessage.error('删除失败 ' + e);
            }
        })
        .catch(() => undefined);
}

async function reorder({ removedIndex, addedIndex }: { removedIndex: number; addedIndex: number }) {
    const list = [...servers.value];
    const [moved] = list.splice(removedIndex, 1);
    list.splice(addedIndex, 0, moved);
    // 就地更新，避免列表闪烁
    servers.value = list;
    try {
        await useEmbyServer().updateOrder(moved.id!, moved.order_by ?? 0, list[addedIndex].order_by ?? 0);
        await refreshServers();
    } catch (e) {
        ElMessage.error('排序失败 ' + e);
        await refreshServers();
    }
}

/* ————————————————— 线路管理 ————————————————— */

const lineServer = ref<EmbyServer | null>(null);
const lineDraft = ref<EmbyLine | null>(null);

function openLineConfig(server: EmbyServer) {
    lineServer.value = server;
    void reloadLines();
}

function startAddLine() {
    lineDraft.value = {
        id: '',
        emby_server_id: lineServer.value?.id,
        emby_server_name: lineServer.value?.server_name,
        reverse_proxy_id: 'no',
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow',
    };
}

function startEditLine(line: EmbyLine) {
    lineDraft.value = line;
}

function removeLine(line: EmbyLine) {
    if (line.id === lineServer.value?.line_id) {
        ElMessage.error('不能删除正在使用的线路');
        return;
    }
    ElMessageBox.confirm(`删除线路「${line.name}」？`, '删除线路', {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
    })
        .then(async () => {
            try {
                await useEmbyLine().delEmbyLine(line.id!);
                await reloadLines();
                ElMessage.success('已删除');
            } catch (e) {
                ElMessage.error('删除失败 ' + e);
            }
        })
        .catch(() => undefined);
}

async function saveLine(draft: EmbyLine) {
    try {
        if (draft.id) {
            await useEmbyLine().updateEmbyLine(draft);
        } else {
            await useEmbyLine().addEmbyLine({ ...draft, id: generateGuid() });
        }
        await reloadLines();
        // 正在使用的线路被改动时，同步到服务器
        const active = lineServer.value;
        if (active && draft.id === active.line_id) {
            await useEmbyServer().updateEmbyServer({
                id: active.id,
                base_url: draft.base_url,
                reverse_proxy_id: draft.reverse_proxy_id,
                browse_proxy_id: draft.browse_proxy_id,
                play_proxy_id: draft.play_proxy_id,
            });
            bus.emit('EmbyServerChanged', { event: 'update', id: active.id });
        }
        ElMessage.success('已保存');
        lineDraft.value = null;
    } catch (e) {
        ElMessage.error('保存线路失败 ' + e);
    }
}

/* ————————————————— 图标 ————————————————— */

const iconServer = ref<EmbyServer | null>(null);
const iconLibraries = ref<EmbyIconLibrary[]>([]);
const iconLibraryId = ref('');
const iconKeyword = ref('');
const iconLoading = ref(false);
const iconChoices = ref<IconChoice[]>([]);

async function openIconPicker(server: EmbyServer) {
    iconServer.value = server;
    iconKeyword.value = '';
    try {
        iconLibraries.value = await useEmbyIconLibrary().listAllEmbyIconLibrary();
        iconLibraryId.value = iconLibraries.value[0]?.id ?? '';
        await loadIcons();
    } catch (e) {
        ElMessage.error('获取图标库失败 ' + e);
    }
}

async function loadIcons() {
    const library = iconLibraries.value.find(item => item.id === iconLibraryId.value);
    if (!library?.url) {
        iconChoices.value = [];
        return;
    }
    iconLoading.value = true;
    iconChoices.value = [];
    try {
        const payload = await appApi.getEmbyIconLibrary(library.url);
        iconChoices.value = (payload.icons ?? []).map(icon => ({
            name: icon.name,
            url: icon.url,
            localUrl: image.iconUrl(icon.url),
        }));
    } catch (e) {
        ElMessage.error('加载图标库失败，检查图标库地址与应用数据代理：' + e);
    } finally {
        iconLoading.value = false;
    }
}

async function applyIcon(url: string) {
    const target = iconServer.value;
    if (!target?.id) {
        return;
    }
    try {
        await useEmbyServer().updateEmbyServer({ id: target.id, icon_url: url });
        bus.emit('EmbyServerChanged', { event: 'update', id: target.id });
        iconServer.value = null;
        ElMessage.success('图标已更新');
    } catch (e) {
        ElMessage.error('更新图标失败 ' + e);
    }
}

/* ————————————————— 消息中心 ————————————————— */

const notifyVisible = ref(false);
const boardScroll = ref<ScrollbarInstance>();

function toggleNotify() {
    notifyVisible.value = !notifyVisible.value;
}

function onNotifyMessageChange(param: { force_open: boolean }) {
    if (param.force_open) {
        notifyVisible.value = true;
    }
}

/* ————————————————— 生命周期 ————————————————— */

onMounted(() => {
    bus.on('EmbyLineChanged', reloadLines);
    bus.on('ProxyServerChanged', reloadProxies);
    bus.on('ReverseProxyServerChanged', reloadProxies);
    bus.on('GlobalProxyChanged', reloadGlobalProxyNames);
    bus.on('notifyMessageChange', onNotifyMessageChange);

    listen<{ event: string; id?: string }>('EmbyServerChange', event => {
        bus.emit('EmbyServerChanged', { event: event.payload.event, id: event.payload.id });
    }).catch(() => undefined);
});

onUnmounted(() => {
    bus.remove('EmbyLineChanged', reloadLines);
    bus.remove('ProxyServerChanged', reloadProxies);
    bus.remove('ReverseProxyServerChanged', reloadProxies);
    bus.remove('GlobalProxyChanged', reloadGlobalProxyNames);
    bus.remove('notifyMessageChange', onNotifyMessageChange);
});

void reloadLines();
void reloadProxies();
void reloadGlobalProxyNames();

// 换服务器或换页面时，内容区回到顶部
watch(
    () => route.fullPath,
    () => boardScroll.value?.setScrollTop(0),
);
</script>

<style scoped>
.shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--surface);
}

.shell__body {
    display: flex;
    flex: auto;
    min-height: 0;
}

.board {
    display: flex;
    flex-direction: column;
    flex: auto;
    min-width: 0;
}

.board__scroll {
    flex: auto;
    min-height: 0;
}

.statusbar {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    height: var(--h-statusbar);
    padding: 0 0.75rem;
    border-top: 1px solid var(--line);
    background: var(--surface-raised);
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.statusbar__group {
    display: flex;
    align-items: center;
    gap: 1rem;
}
</style>
