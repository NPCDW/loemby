<template>
    <section v-if="ids.length || links.length" class="mobile-section detail-metadata" aria-label="外部资料">
        <h2>外部资料</h2>
        <dl v-if="ids.length" class="provider-ids">
            <div v-for="[name, value] in ids" :key="name"><dt>{{ name }}</dt><dd>{{ value }}</dd></div>
        </dl>
        <div v-if="links.length" class="mobile-actions">
            <button v-for="(link, index) in links" :key="`${link.Url}:${index}`" type="button" :title="link.Url" :disabled="pending" @click="open(link.Url)">{{ link.Name || link.Url }}</button>
        </div>
        <p v-if="pending" class="mobile-muted" role="status">正在打开外部资料…</p>
        <p v-if="error" class="mobile-error" role="alert">{{ error }}</p>
    </section>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import type { BaseItem } from '../../api/embyApi'
import { usePlatformData } from '../data/platform'
const props = defineProps<{ item: BaseItem }>()
const platform = usePlatformData()
const ids = computed(() => Object.entries(props.item.ProviderIds || {}))
const links = computed(() => props.item.ExternalUrls || [])
const pending = ref(false)
const error = ref('')
let session = 0
let disposed = false
watch(() => props.item, () => { session++; pending.value = false; error.value = '' })
onUnmounted(() => { disposed = true; session++ })
async function open(url: string) {
    if (disposed || pending.value) return
    const current = session
    pending.value = true
    error.value = ''
    try {
        if (!['http:', 'https:'].includes(new URL(url).protocol)) throw new Error('不支持此链接类型')
        await platform.openUrl(url)
    } catch {
        if (!disposed && current === session) error.value = '无法打开外部资料。Android 系统浏览器桥接待后续适配，请重试或手动打开链接。'
    } finally { if (!disposed && current === session) pending.value = false }
}
</script>

<style scoped>
.provider-ids { display: flex; flex-wrap: wrap; gap: 10px; }
.provider-ids > div { border: 1px solid var(--el-border-color-lighter); border-radius: 8px; padding: 8px 12px; max-width: 100%; }
.provider-ids dt { color: var(--el-text-color-secondary); font-size: 13px; }
.provider-ids dd { margin: 4px 0 0; overflow-wrap: anywhere; }
.detail-metadata button { min-height: 44px; max-width: 100%; overflow-wrap: anywhere; }
</style>
