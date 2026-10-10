<template>
    <div class="fatal-error">
        <el-result icon="error" title="数据库连接失败" :sub-title="summary">
            <template #extra>
                <div class="fatal-error-detail">
                    <div class="fatal-error-detail-title">
                        <el-icon><i-ep-InfoFilled /></el-icon>
                        <span>排查建议</span>
                    </div>
                    <pre class="fatal-error-detail-body">{{ reason }}</pre>
                </div>
                <div class="fatal-error-actions">
                    <el-button type="primary" @click="retry">重试连接</el-button>
                    <el-button @click="openLogFolder">打开日志目录</el-button>
                    <el-button @click="restart">重启应用</el-button>
                </div>
                <div class="fatal-error-tip">
                    数据库不可用期间，服务器列表、播放历史、设置等依赖数据库的功能将无法使用。
                </div>
            </template>
        </el-result>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessage } from 'element-plus';
import { listen } from '@tauri-apps/api/event';
import invokeApi from '../api/invokeApi';

const reason = ref('正在获取数据库连接失败的原因…')

// 只给用户看第一行结论，细节收在下方，避免长文本挤爆页面
const summary = computed(() => reason.value.split('\n').find(line => line.trim() !== '') ?? '数据库连接失败')

let unlisten: (() => void) | undefined

function applyReason(payload: { reason: string }) {
    reason.value = payload.reason
}

onMounted(async () => {
    unlisten = await listen<{ reason: string }>('db_fatal_error', event => applyReason(event.payload))
})
onUnmounted(() => unlisten?.())

const retrying = ref(false)
async function retry() {
    retrying.value = true
    ElMessage.warning('应用需要重启才能重建数据库连接，正在为你重启…')
    try {
        await invokeApi.restartApp()
    } catch (e) {
        ElMessage.error('重启失败，请手动关闭并重新打开应用：' + e)
    } finally {
        retrying.value = false
    }
}

async function restart() {
    try {
        await invokeApi.restartApp()
    } catch (e) {
        ElMessage.error('重启失败，请手动关闭并重新打开应用：' + e)
    }
}

async function openLogFolder() {
    try {
        await invokeApi.open_folder('log')
    } catch (e) {
        ElMessage.error('打开日志目录失败：' + e)
    }
}

</script>

<style scoped>
.fatal-error {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 24px;
    box-sizing: border-box;
    background-color: var(--el-bg-color-page);
}

.fatal-error-detail {
    max-width: 720px;
    margin: 0 auto 16px;
    text-align: left;
    border: 1px solid var(--el-border-color-light);
    border-radius: 6px;
    background-color: var(--el-fill-color-lighter);
}

.fatal-error-detail-title {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    font-size: 13px;
    color: var(--el-text-color-secondary);
    border-bottom: 1px solid var(--el-border-color-lighter);
}

.fatal-error-detail-body {
    margin: 0;
    padding: 12px;
    max-height: 260px;
    overflow: auto;
    font-size: 12px;
    line-height: 1.7;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--el-text-color-regular);
}

.fatal-error-actions {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-bottom: 12px;
}

.fatal-error-tip {
    font-size: 12px;
    color: var(--el-text-color-secondary);
}
</style>
