<template>
    <aside class="preview-tools" aria-label="模拟预览工具">
        <details>
            <summary>开发预览 · 全部为模拟数据</summary>
            <p>预览播放历史与浏览主链路，不连接数据库，不播放媒体，也不会修改真实记录。</p>
            <div class="preview-controls">
                <label><input v-model="previewState.empty" type="checkbox" />空记录</label>
                <label><input v-model="previewState.failQuery" type="checkbox" />查询失败</label>
                <label><input v-model="previewState.failServers" type="checkbox" />服务器列表失败</label>
                <label><input v-model="previewState.failUpdate" type="checkbox" />标记更新失败</label>
                <label><input v-model="slow" type="checkbox" />慢请求（2秒）</label>
                <label><input v-model="dark" type="checkbox" />深色主题</label>
            </div>
            <p>切换场景后，点击页面中的查询、重试或标记按钮触发。</p>
            <button type="button" @click="pageKey++">重新挂载当前页</button>
            <button type="button" @click="reload">返回播放历史</button>
        </details>
    </aside>
    <RouterView :key="pageKey" />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { RouterView, useRouter } from 'vue-router'
import { previewState } from './data'

const dark = ref(true)
const slow = ref(false)
const pageKey = ref(0)
const router = useRouter()
watch(dark, value => document.documentElement.classList.toggle('dark', value))
watch(slow, value => previewState.delay = value ? 2000 : 350)
async function reload() {
    await router.push('/nav/history')
    pageKey.value++
}
</script>
