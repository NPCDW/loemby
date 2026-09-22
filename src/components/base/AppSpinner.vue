<template>
    <div class="spinner" :style="{ '--rows': rows }">
        <span v-for="row in rows" :key="row" class="spinner__row" />
    </div>
</template>

<script setup lang="ts">
/**
 * 占位骨架。
 *
 * 只画内容本身的形状（行/块），不画整张卡片的轮廓，
 * 这样加载完成时不会出现「壳变了」的抖动。
 */
withDefaults(defineProps<{ rows?: number }>(), { rows: 3 });
</script>

<style scoped>
.spinner {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
}

.spinner__row {
    height: 0.75rem;
    border-radius: var(--r-xs);
    background: linear-gradient(
        90deg,
        var(--surface-card) 25%,
        var(--surface-hover) 37%,
        var(--surface-card) 63%
    );
    background-size: 400% 100%;
    animation: spinner-sweep 1.4s ease infinite;
}

.spinner__row:nth-child(2n) {
    width: 72%;
}

.spinner__row:nth-child(3n) {
    width: 88%;
}

@keyframes spinner-sweep {
    0% {
        background-position: 100% 50%;
    }
    100% {
        background-position: 0 50%;
    }
}
</style>
