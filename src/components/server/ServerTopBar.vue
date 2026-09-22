<template>
    <header class="topbar">
        <div class="topbar__left">
            <h1 class="topbar__title">{{ title }}</h1>
            <div v-if="server" class="topbar__ctx">
                <span class="topbar__server t-clip">{{ server.server_name }}</span>
                <span class="topbar__rule" />
                <ServerBadge :disabled="server.disabled" :authenticated="!!server.auth_token" />
            </div>
        </div>

        <div class="topbar__right">
            <slot />
        </div>
    </header>
</template>

<script setup lang="ts">
import ServerBadge from './ServerBadge.vue';
import type { EmbyServer } from '../../store/db/embyServer';

/**
 * 顶栏：页面标题 + 当前服务器上下文 + 页面自己的工具。
 *
 * 页面标题由这里统一渲染，页面本身不再重复一个大标题，
 * 滚动时用户始终知道自己在哪台服务器的哪个页面。
 */
withDefaults(
    defineProps<{
        title: string;
        server?: EmbyServer | null;
    }>(),
    { server: null },
);
</script>

<style scoped>
.topbar {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    height: var(--h-topbar);
    padding: 0 var(--gutter);
    border-bottom: 1px solid var(--line);
    background: var(--surface);
}

.topbar__left {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    min-width: 0;
}

.topbar__title {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: 600;
    letter-spacing: -0.005em;
}

.topbar__ctx {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    color: var(--text-faint);
    font-size: var(--fs-sm);
}

.topbar__server {
    max-width: 18rem;
}

.topbar__rule {
    width: 1px;
    height: 0.625rem;
    background: var(--line);
}

.topbar__right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: none;
}
</style>
