<template>
    <div class="tools">
        <el-select :model-value="lineId" size="small" class="tools__select" placement="bottom-end" @change="emit('changeLine', $event)">
            <template #prefix><span class="tools__key">线路</span></template>
            <el-option v-for="line in lines" :key="line.id" :label="line.name" :value="line.id!" />
        </el-select>

        <button class="tools__icon" type="button" title="管理线路" @click="emit('configLine')">
            <el-icon :size="14"><i-ep-Setting /></el-icon>
        </button>

        <el-select
            :model-value="line?.reverse_proxy_id"
            size="small"
            class="tools__select"
            placement="bottom-end"
            @change="emit('changeProxy', { key: 'reverse_proxy_id', value: $event })"
        >
            <template #prefix><span class="tools__key">反代</span></template>
            <el-option label="不使用反代" value="no" />
            <el-option v-for="item in reverseProxies" :key="item.id" :label="item.name" :value="item.id!" />
        </el-select>

        <el-select
            :model-value="line?.browse_proxy_id"
            size="small"
            class="tools__select"
            placement="bottom-end"
            @change="emit('changeProxy', { key: 'browse_proxy_id', value: $event })"
        >
            <template #prefix><span class="tools__key">浏览</span></template>
            <el-option label="不使用代理" value="no" />
            <el-option :label="`跟随全局（${globalBrowseName}）`" value="follow" />
            <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
        </el-select>

        <el-select
            :model-value="line?.play_proxy_id"
            size="small"
            class="tools__select"
            placement="bottom-end"
            @change="emit('changeProxy', { key: 'play_proxy_id', value: $event })"
        >
            <template #prefix><span class="tools__key">播放</span></template>
            <el-option label="不使用代理" value="no" />
            <el-option :label="`跟随全局（${globalPlayName}）`" value="follow" />
            <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
        </el-select>
    </div>
</template>

<script setup lang="ts">
import type { EmbyLine } from '../../store/db/embyLine';
import type { ProxyServer } from '../../store/db/proxyServer';
import type { ReverseProxyServer } from '../../store/db/reverseProxyServer';

/**
 * 顶栏里的线路与代理开关。
 *
 * 只出现在服务器上下文里：切线路、切反代、切两种代理。
 * 选项标签统一带「不使用 / 跟随全局」前缀，避免出现无标签的空选项。
 */
defineProps<{
    lineId?: string;
    line?: EmbyLine;
    lines: EmbyLine[];
    proxies: ProxyServer[];
    reverseProxies: ReverseProxyServer[];
    globalBrowseName: string;
    globalPlayName: string;
}>();

const emit = defineEmits<{
    changeLine: [lineId: string];
    changeProxy: [payload: { key: 'reverse_proxy_id' | 'browse_proxy_id' | 'play_proxy_id'; value: string }];
    configLine: [];
}>();
</script>

<style scoped>
.tools {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex: none;
}

.tools__select {
    width: 9.5rem;
}

.tools__key {
    margin-right: 2px;
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.tools__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-faint);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.tools__icon:hover {
    color: var(--lamp);
    border-color: var(--lamp-edge);
}
</style>
