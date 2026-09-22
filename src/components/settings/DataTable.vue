<template>
    <div class="dt">
        <header class="dt__head">
            <span v-if="title" class="dt__title">{{ title }}</span>
            <span v-if="caption" class="t-faint">{{ caption }}</span>
            <div class="dt__tools">
                <slot name="tools" />
            </div>
        </header>

        <div v-if="rows.length" class="dt__body">
            <div v-for="(row, index) in rows" :key="rowKey(row, index)" class="dt__row">
                <div class="dt__cells">
                    <slot :row="row" :index="index" />
                </div>
                <div class="dt__ops">
                    <slot name="ops" :row="row" :index="index" />
                </div>
            </div>
        </div>

        <AppEmpty v-else :title="emptyTitle" :hint="emptyHint" />
    </div>
</template>

<script setup lang="ts" generic="T extends Record<string, unknown>">
import AppEmpty from '../base/AppEmpty.vue';

/**
 * 轻量列表：每一行是一条记录。
 *
 * 用行而不是表格，是因为这些记录的字段宽度差异很大（地址、备注），
 * 表格会把它们强行等宽反而难读；行布局在窄窗口下还能自然换行。
 *
 * 泛型是为了让 slot 里的 row 拿到真实类型，而不是 unknown。
 */
const props = withDefaults(
    defineProps<{
        rows: T[];
        title?: string;
        caption?: string;
        emptyTitle?: string;
        emptyHint?: string;
        /** 取哪一个字段作为列表 key，默认 id */
        rowKeyField?: string;
    }>(),
    { title: '', caption: '', emptyTitle: '这里是空的', emptyHint: '还没有任何记录。', rowKeyField: 'id' },
);

function rowKey(row: T, index: number): string {
    const value = row[props.rowKeyField as keyof T];
    return value === undefined || value === null ? String(index) : String(value);
}
</script>

<style scoped>
.dt {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.dt__head {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    min-height: 1.75rem;
}

.dt__title {
    font-size: var(--fs-lg);
    font-weight: 600;
}

.dt__tools {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-left: auto;
}

.dt__body {
    display: flex;
    flex-direction: column;
}

.dt__row {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 0.25rem;
    border-bottom: 1px solid var(--line);
    transition: background var(--dur-fast) var(--ease);
}

.dt__row:hover {
    background: var(--surface-hover);
}

.dt__row:last-child {
    border-bottom: none;
}

.dt__cells {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem 1.25rem;
    flex: auto;
    min-width: 0;
}

.dt__ops {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex: none;
}
</style>
