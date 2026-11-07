import { onUnmounted, ref, watch } from 'vue'

export function useResource<T>(fetcher: () => Promise<T>, deps: () => unknown) {
    const value = ref<T>()
    const loading = ref(false)
    const error = ref('')
    let requestId = 0
    let disposed = false

    async function load() {
        if (disposed) return
        const id = ++requestId
        loading.value = true
        error.value = ''
        value.value = undefined
        try {
            const result = await fetcher()
            if (!disposed && id === requestId) value.value = result
        } catch {
            if (!disposed && id === requestId) error.value = '加载失败，请检查服务器连接后重试。'
        } finally {
            if (!disposed && id === requestId) loading.value = false
        }
    }
    watch(deps, () => { void load() }, { immediate: true })
    onUnmounted(() => { disposed = true; requestId++ })
    return { value, loading, error, load }
}
