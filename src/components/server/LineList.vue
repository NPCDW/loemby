<template>
    <div class="lines">
        <button class="lines__add" type="button" @click="emit('add')">
            <el-icon :size="14"><i-ep-Plus /></el-icon>
            <span>添加线路</span>
        </button>

        <div
            v-for="line in lines"
            :key="line.id"
            class="line"
            :class="{ 'is-active': line.id === activeId }"
            @click="emit('select', line)"
        >
            <span class="line__radio" />
            <span class="line__body">
                <span class="line__head">
                    <span class="line__name">{{ line.name }}</span>
                    <span class="line__ops" @click.stop>
                        <button class="ghost ghost--bare" type="button" title="编辑线路" @click="emit('edit', line)">
                            <el-icon :size="14"><i-ep-Edit /></el-icon>
                        </button>
                        <button
                            class="ghost ghost--bare"
                            type="button"
                            title="删除线路"
                            :disabled="line.id === activeId"
                            @click="emit('remove', line)"
                        >
                            <el-icon :size="14"><i-ep-Delete /></el-icon>
                        </button>
                    </span>
                </span>
                <span class="line__url mono t-clip">{{ line.base_url }}</span>
                <span class="line__meta">
                    <span class="t-faint">反代</span>
                    <span class="t-clip">{{ reverseProxyName(line.reverse_proxy_id) }}</span>
                    <span class="sep">·</span>
                    <span class="t-faint">浏览</span>
                    <span class="t-clip">{{ browseProxyName(line.browse_proxy_id) }}</span>
                    <span class="sep">·</span>
                    <span class="t-faint">播放</span>
                    <span class="t-clip">{{ playProxyName(line.play_proxy_id) }}</span>
                </span>
            </span>
        </div>

        <p v-if="lines.length === 0" class="t-faint lines__hint">
            还没有线路。至少添加一条，客户端才知道去哪里找这台服务器。
        </p>
    </div>
</template>

<script setup lang="ts">
import type { EmbyLine } from '../../store/db/embyLine';
import { useProxyServer } from '../../store/db/proxyServer';
import { useReverseProxyServer } from '../../store/db/reverseProxyServer';

/**
 * 线路列表：一条线路一行，选中项用左侧灯条标记。
 *
 * 每行直接标出「反代 / 浏览代理 / 播放代理」，不用打开详情才能看到走了哪条路。
 */
defineProps<{
    lines: EmbyLine[];
    activeId?: string;
}>();

const emit = defineEmits<{
    select: [line: EmbyLine];
    edit: [line: EmbyLine];
    remove: [line: EmbyLine];
    add: [];
}>();

function reverseProxyName(id?: string): string {
    if (!id || id === 'no') {
        return '不使用反代';
    }
    return useReverseProxyServer().cacheName(id) || '未知反代';
}

function browseProxyName(id?: string): string {
    if (!id || id === 'no') {
        return '不使用代理';
    }
    if (id === 'follow') {
        return '跟随全局';
    }
    return useProxyServer().cacheName(id) || '未知代理';
}

function playProxyName(id?: string): string {
    return browseProxyName(id);
}
</script>

<style scoped>
.lines {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.lines__add {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    align-self: flex-start;
    padding: 0.375rem 0.75rem;
    border: 1px dashed var(--line);
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-dim);
    font-size: var(--fs-sm);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.lines__add:hover {
    color: var(--lamp);
    border-color: var(--lamp-edge);
}

.lines__hint {
    margin: 0;
    line-height: 1.7;
}

.line {
    display: flex;
    gap: 0.75rem;
    padding: 0.625rem 0.75rem;
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    background: var(--surface);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.line:hover {
    border-color: var(--line-strong);
    background: var(--surface-hover);
}

.line.is-active {
    border-color: var(--lamp-edge);
    background: var(--lamp-wash);
}

.line__radio {
    flex: none;
    align-self: flex-start;
    width: 12px;
    height: 12px;
    margin-top: 5px;
    border: 1px solid var(--line-strong);
    border-radius: 50%;
    transition: border-color var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease);
}

.line.is-active .line__radio {
    border-color: var(--lamp);
    box-shadow: inset 0 0 0 3px var(--lamp);
}

.line__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: auto;
    min-width: 0;
}

.line__head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.line__name {
    flex: auto;
    min-width: 0;
    font-size: var(--fs-md);
    font-weight: 500;
}

.line__ops {
    flex: none;
    display: flex;
    gap: 2px;
}

.line__url {
    color: var(--text-faint);
    font-size: var(--fs-xs);
}

.line__meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    color: var(--text-dim);
    font-size: var(--fs-xs);
}
</style>
