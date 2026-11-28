<template>
    <div class="emby-address-input">
        <el-select v-model="address.protocol" aria-label="协议">
            <el-option label="http" value="http" />
            <el-option label="https" value="https" />
        </el-select>
        <el-input v-model="address.hostname" aria-label="域名" placeholder="域名" @paste.prevent="pasteAddress" />
        <el-input v-model="address.port" aria-label="端口" placeholder="端口" inputmode="numeric" />
        <el-input v-model="address.path" aria-label="路径" placeholder="路径" />
    </div>
</template>

<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { parsePastedEmbyAddress, type EmbyAddress } from '../util/emby_address'

const address = defineModel<EmbyAddress>({ required: true })

function pasteAddress(event: ClipboardEvent) {
    const text = event.clipboardData?.getData('text/plain')
    if (!text?.trim()) return

    const parsed = parsePastedEmbyAddress(text, address.value.protocol)
    if (!parsed) {
        ElMessage.error('请输入有效的 HTTP 或 HTTPS 地址')
        return
    }
    address.value = parsed
}
</script>

<style scoped>
.emby-address-input {
    display: grid;
    grid-template-columns: 86px minmax(0, 2fr) 70px minmax(0, 1fr);
    gap: 6px;
    width: 100%;
}
</style>
