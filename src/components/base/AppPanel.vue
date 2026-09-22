<template>
    <section class="panel">
        <header v-if="title || $slots.actions" class="panel__head">
            <h3 v-if="title">{{ title }}</h3>
            <span v-if="caption" class="t-faint mono">{{ caption }}</span>
            <div v-if="$slots.actions" class="section__head-ops">
                <slot name="actions" />
            </div>
        </header>
        <slot />
    </section>
</template>

<script setup lang="ts">
/**
 * 面板：内容需要与背景分开时使用。
 * 层级靠边线，不靠投影。
 */
withDefaults(
    defineProps<{
        title?: string;
        caption?: string;
    }>(),
    { title: '', caption: '' },
);
</script>
