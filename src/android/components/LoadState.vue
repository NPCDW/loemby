<template>
    <div v-if="loading" class="mobile-state" role="status">正在加载…</div>
    <div v-else-if="error" class="mobile-state" role="alert">
        <p>{{ error }}</p><button type="button" @click="$emit('retry')">重试</button>
    </div>
    <div v-else-if="empty" class="mobile-state"><p>{{ emptyText || '暂无内容' }}</p></div>
    <slot v-else />
</template>

<script setup lang="ts">
defineProps<{ loading?: boolean, error?: string, empty?: boolean, emptyText?: string }>()
defineEmits<{ retry: [] }>()
</script>
