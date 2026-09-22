<template>
    <div class="rail">
        <button
            v-for="chapter in chapters"
            :key="chapter.ChapterIndex"
            class="rail__item"
            type="button"
            :title="chapter.Name || chapter.MarkerType"
            @click="emit('jump', chapter)"
        >
            <span class="rail__time mono">{{ formatClock(chapter.StartPositionTicks / TICKS_PER_SECOND) }}</span>
            <span class="rail__name t-clip">{{ chapter.Name || chapter.MarkerType || '未命名' }}</span>
        </button>
    </div>
</template>

<script setup lang="ts">
import type { Chapter } from '../../api/embyApi';
import { formatClock } from '../../util/format';

/**
 * 章节条：可点击跳转的时间点。
 * 章节名常为空，此时退回 MarkerType，再退回「未命名」，不留空白按钮。
 */
const TICKS_PER_SECOND = 10_000_000;

defineProps<{ chapters: Chapter[] }>();

const emit = defineEmits<{ jump: [chapter: Chapter] }>();
</script>

<style scoped>
.rail {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.rail__item {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    max-width: 24rem;
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-dim);
    font-size: var(--fs-sm);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.rail__item:hover {
    color: var(--text);
    border-color: var(--lamp-edge);
}

.rail__time {
    flex: none;
    color: var(--lamp);
    font-size: var(--fs-xs);
}

.rail__name {
    min-width: 0;
}
</style>
