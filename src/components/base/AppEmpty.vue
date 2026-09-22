<template>
    <div class="empty">
        <span class="empty__title">{{ title }}</span>
        <span v-if="hint" class="empty__hint">{{ hint }}</span>
        <div v-if="$slots.actions" class="empty__actions">
            <slot name="actions" />
        </div>
    </div>
</template>

<script setup lang="ts">
/**
 * 空态与失败态。
 *
 * 约定：永远给一句「下一步做什么」，而不是只留一片空白或一个图标。
 */
withDefaults(
    defineProps<{
        title: string;
        hint?: string;
    }>(),
    { hint: '' },
);
</script>
