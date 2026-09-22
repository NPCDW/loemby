<template>
    <section class="section">
        <header class="section__head">
            <h2>{{ title }}</h2>
            <span v-if="count !== undefined && count !== ''" class="num">{{ count }}</span>
            <span v-else-if="caption" class="t-faint">{{ caption }}</span>
            <div v-if="$slots.actions" class="section__head-ops">
                <slot name="actions" />
            </div>
        </header>
        <slot />
    </section>
</template>

<script setup lang="ts">
/**
 * 段落：所有页面共用的一种横排。
 *
 * 计数用等宽数字放在标题旁，滚动或翻页时不会让标题跳动。
 */
withDefaults(
    defineProps<{
        title: string;
        /** 数字型计数，等宽显示 */
        count?: number | string;
        /** 非数字说明文字 */
        caption?: string;
    }>(),
    { count: undefined, caption: '' },
);
</script>
