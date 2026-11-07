<template>
    <nav v-if="total > 0" class="mobile-pagination" aria-label="内容分页">
        <button type="button" :disabled="loading || page <= 1" @click="$emit('change', page - 1)">上一页</button>
        <span aria-live="polite">{{ page }} / {{ pages }}<small>共 {{ total }} 项</small></span>
        <button type="button" :disabled="loading || page >= pages" @click="$emit('change', page + 1)">下一页</button>
    </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ page: number, total: number, pageSize: number, loading?: boolean }>()
defineEmits<{ change: [page: number] }>()
const pages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)))
</script>

<style scoped>
.mobile-pagination { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin: 24px 0; }
.mobile-pagination span { text-align: center; font-variant-numeric: tabular-nums; }
.mobile-pagination small { display: block; color: var(--el-text-color-secondary); }
</style>
