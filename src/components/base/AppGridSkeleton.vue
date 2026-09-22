<template>
    <div class="grid" :style="{ '--col': col }">
        <span v-for="cell in count" :key="cell" class="grid-skeleton" />
    </div>
</template>

<script setup lang="ts">
/**
 * 网格占位：与真实网格同列宽，避免加载完成后的布局重排。
 */
withDefaults(defineProps<{ count?: number; col?: string }>(), { count: 12, col: '11.5rem' });
</script>

<style scoped>
.grid-skeleton {
    display: block;
    aspect-ratio: 2 / 3;
    border-radius: var(--r-md);
    background: linear-gradient(
        90deg,
        var(--surface-card) 25%,
        var(--surface-hover) 37%,
        var(--surface-card) 63%
    );
    background-size: 400% 100%;
    animation: grid-sweep 1.4s ease infinite;
}

@keyframes grid-sweep {
    0% {
        background-position: 100% 50%;
    }
    100% {
        background-position: 0 50%;
    }
}
</style>
