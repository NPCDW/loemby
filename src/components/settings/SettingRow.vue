<template>
    <div class="srow" :class="{ 'is-stacked': stacked }">
        <div class="srow__label">
            <span class="srow__title">
                {{ label }}
                <span v-if="unit" class="t-faint mono">（{{ unit }}）</span>
            </span>
            <span v-if="hint" class="srow__hint">{{ hint }}</span>
        </div>
        <div class="srow__control">
            <slot />
        </div>
    </div>
</template>

<script setup lang="ts">
/**
 * 设置项：左侧「名称 + 说明」，右侧控件。
 *
 * 所有设置页共用同一种节奏——说明放在名称下面而不是当 placeholder，
 * 这样用户滚动时不会丢掉上下文。
 */
withDefaults(
    defineProps<{
        label: string;
        hint?: string;
        unit?: string;
        /** 控件需要占整行时（文本框、多选）改成纵向 */
        stacked?: boolean;
    }>(),
    { hint: '', unit: '', stacked: false },
);
</script>

<style scoped>
.srow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    padding: 0.875rem 0;
    border-bottom: 1px solid var(--line);
}

.srow:last-child {
    border-bottom: none;
}

.srow.is-stacked {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;
}

/* 纵向布局下标签只占内容高度，否则会被 flex-basis 撑出一条空白带 */
.srow.is-stacked .srow__label {
    flex: none;
}

.srow__label {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1 1 14rem;
    min-width: 0;
}

.srow__title {
    color: var(--text);
    font-size: var(--fs-md);
}

.srow__hint {
    max-width: 62ch;
    color: var(--text-faint);
    font-size: var(--fs-xs);
    line-height: 1.7;
}

.srow__control {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    flex: none;
}

.srow.is-stacked .srow__control {
    flex: auto;
}
</style>
