<template>
    <AppPanel :title="title" :caption="caption">
        <div class="block">
            <slot />
        </div>
    </AppPanel>
</template>

<script setup lang="ts">
import AppPanel from '../base/AppPanel.vue';

/**
 * 设置分组：一个面板装一组相关设置，组与组之间靠间距分开。
 */
withDefaults(defineProps<{ title: string; caption?: string }>(), { caption: '' });
</script>

<style scoped>
.block {
    display: flex;
    flex-direction: column;
}
</style>
