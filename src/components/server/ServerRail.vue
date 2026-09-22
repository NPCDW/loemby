<template>
    <aside class="rail">
        <button class="rail__brand" type="button" title="回到播放历史" @click="emit('navigate', '/nav/history')">
            <span class="rail__mark"><svg-icon name="app-icon" size="21" color="#F2A13B" /></span>
            <span class="rail__wordmark">
                <span class="rail__name">loemby</span>
                <span class="rail__version mono">v{{ version }}</span>
            </span>
        </button>

        <nav class="rail__nav">
            <button
                v-for="entry in entries"
                :key="entry.path"
                class="rail__item"
                type="button"
                :class="{ 'is-active': active === entry.path }"
                @click="emit('navigate', entry.path)"
            >
                <el-icon :size="17"><component :is="entry.icon" /></el-icon>
                <span>{{ entry.label }}</span>
            </button>
        </nav>

        <div class="rail__section">
            <span class="rail__section-name">服务器</span>
            <button class="rail__add" type="button" title="添加服务器" @click="emit('add')">
                <el-icon :size="13"><i-ep-Plus /></el-icon>
            </button>
        </div>

        <el-scrollbar class="rail__servers">
            <Container @drop="onDrop">
                <Draggable v-for="server in servers" :key="server.id">
                    <el-dropdown trigger="contextmenu" placement="bottom-start">
                        <button
                            class="rail__item rail__item--server"
                            type="button"
                            :class="{ 'is-active': active === `/nav/emby/${server.id}`, 'is-off': !!server.disabled }"
                            @click="emit('navigate', `/nav/emby/${server.id}`)"
                        >
                            <ServerBadge :disabled="server.disabled" :authenticated="!!server.auth_token" />
                            <span class="rail__icon">
                                <ServerIcon :icon-url="server.icon_url" :size="16" :alt="server.server_name" />
                            </span>
                            <span class="rail__server-name t-clip">{{ server.server_name }}</span>
                            <KeepAliveTag
                                :keep-alive-days="server.keep_alive_days"
                                :last-playback-time="server.last_playback_time"
                            />
                        </button>

                        <template #dropdown>
                            <el-dropdown-menu>
                                <el-dropdown-item @click="emit('configLine', server)">
                                    <el-icon><i-ep-Link /></el-icon>
                                    <span>线路</span>
                                </el-dropdown-item>
                                <el-dropdown-item @click="emit('editIcon', server)">
                                    <el-icon><i-ep-PriceTag /></el-icon>
                                    <span>图标</span>
                                </el-dropdown-item>
                                <el-dropdown-item @click="emit('editServer', server)">
                                    <el-icon><i-ep-Edit /></el-icon>
                                    <span>编辑</span>
                                </el-dropdown-item>
                                <el-dropdown-item @click="emit('toggleEnabled', server)">
                                    <el-icon>
                                        <i-ep-CircleCheckFilled v-if="server.disabled" />
                                        <i-ep-CircleCloseFilled v-else />
                                    </el-icon>
                                    <span>{{ server.disabled ? '启用' : '停用' }}</span>
                                </el-dropdown-item>
                                <el-dropdown-item divided @click="emit('relogin', server)">
                                    <el-icon><i-ep-Promotion /></el-icon>
                                    <span>重新登录</span>
                                </el-dropdown-item>
                                <el-dropdown-item class="is-warn" @click="emit('logout', server)">
                                    <el-icon><i-ep-WarnTriangleFilled /></el-icon>
                                    <span>退出登录</span>
                                </el-dropdown-item>
                                <el-dropdown-item class="is-danger" @click="emit('remove', server)">
                                    <el-icon><i-ep-Delete /></el-icon>
                                    <span>删除</span>
                                </el-dropdown-item>
                            </el-dropdown-menu>
                        </template>
                    </el-dropdown>
                </Draggable>
            </Container>

            <p v-if="servers.length === 0" class="rail__hint">
                还没有服务器。点击上方 + 添加一台，填好地址与账号就能开始。
            </p>
        </el-scrollbar>
    </aside>
</template>

<script setup lang="ts">
import { Container, Draggable } from 'vue3-smooth-dnd';
import ServerBadge from './ServerBadge.vue';
import ServerIcon from './ServerIcon.vue';
import KeepAliveTag from './KeepAliveTag.vue';
import type { EmbyServer } from '../../store/db/embyServer';

/**
 * 左侧导航栏。
 *
 * 三块自上而下：品牌 / 主入口 / 服务器列表（可拖拽排序、右键管理）。
 * 服务器是一等公民，所以它占掉剩余全部高度，而不是塞进一个子菜单。
 */
defineProps<{
    version: string;
    servers: EmbyServer[];
    active: string;
}>();

const emit = defineEmits<{
    navigate: [path: string];
    add: [];
    configLine: [server: EmbyServer];
    editIcon: [server: EmbyServer];
    editServer: [server: EmbyServer];
    toggleEnabled: [server: EmbyServer];
    relogin: [server: EmbyServer];
    logout: [server: EmbyServer];
    remove: [server: EmbyServer];
    reorder: [payload: { removedIndex: number; addedIndex: number }];
}>();

const entries = [
    { path: '/nav/history', label: '播放历史', icon: 'i-ep-Clock' },
    { path: '/nav/search', label: '聚合搜索', icon: 'i-ep-Search' },
    { path: '/nav/setting', label: '设置', icon: 'i-ep-Setting' },
];

function onDrop(payload: { removedIndex: number; addedIndex: number }) {
    if (payload.removedIndex === null || payload.addedIndex === null) {
        return;
    }
    emit('reorder', payload);
}
</script>

<style scoped>
.rail {
    flex: none;
    display: flex;
    flex-direction: column;
    width: var(--w-rail);
    min-height: 0;
    padding: 0.75rem 0.5rem 0.5rem;
    border-right: 1px solid var(--line);
    background: var(--surface-raised);
}

.rail__brand {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.25rem 0.625rem 1rem;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
}

.rail__mark {
    display: flex;
    color: var(--lamp);
}

.rail__wordmark {
    display: flex;
    align-items: baseline;
    gap: 0.375rem;
    min-width: 0;
}

.rail__name {
    font-size: var(--fs-lg);
    font-weight: 600;
    letter-spacing: 0.01em;
}

.rail__version {
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.rail__nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 0.5rem;
    margin-bottom: 0.75rem;
    border-bottom: 1px solid var(--line);
}

.rail__item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.625rem;
    width: 100%;
    padding: 0.4375rem 0.625rem;
    border: 1px solid transparent;
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-dim);
    font-size: var(--fs-md);
    text-align: left;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease),
        border-color var(--dur-fast) var(--ease);
}

.rail__item:hover {
    background: var(--surface-hover);
    color: var(--text);
}

.rail__item.is-active {
    background: var(--lamp-wash);
    color: var(--text);
    box-shadow: inset 2px 0 0 var(--lamp);
}

.rail__item--server {
    padding-right: 0.5rem;
    margin-bottom: 1px;
}

.rail__item--server.is-off {
    opacity: 0.5;
}

.rail__section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 0.625rem 0.375rem;
}

.rail__section-name {
    color: var(--text-ghost);
    font-size: var(--fs-xs);
    letter-spacing: 0.04em;
}

.rail__add {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.rail__add:hover {
    color: var(--lamp);
    border-color: var(--lamp-edge);
}

.rail__servers {
    flex: auto;
    min-height: 0;
}

.rail__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: 1.125rem;
    color: inherit;
}

.rail__server-name {
    flex: auto;
    min-width: 0;
}

.rail__hint {
    margin: 0;
    padding: 0.625rem;
    color: var(--text-ghost);
    font-size: var(--fs-sm);
    line-height: 1.7;
}

:deep(.el-dropdown-menu__item .el-icon) {
    margin-right: 0.5rem;
}

:deep(.el-dropdown-menu__item.is-warn) {
    color: var(--warn);
}

:deep(.el-dropdown-menu__item.is-danger) {
    color: var(--bad);
}
</style>
