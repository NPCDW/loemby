<template>
    <main class="preview-detail">
        <h1>详情路由预览</h1>
        <p>此处仅验证名称跳转的路由，不模拟详情或播放。真实应用继续使用原有桌面详情页。</p>
        <code>{{ route.path }}</code>
        <RouterLink to="/nav/history">返回播放历史</RouterLink>
    </main>
</template>

<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
const route = useRoute()
</script>
