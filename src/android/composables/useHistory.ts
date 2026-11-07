import { computed, onMounted, onUnmounted, ref } from 'vue'
import type { HistoryData } from '../data/history'
import type { EmbyServer } from '../../store/db/embyServer'
import type { PagePlayHistoryParam, PlayHistory } from '../../store/db/playHistory'

export function useHistory(data: HistoryData) {
    const servers = ref<EmbyServer[]>([])
    const serversLoading = ref(false)
    const serversError = ref('')
    const list = ref<PlayHistory[]>([])
    const total = ref(0)
    const loading = ref(false)
    const error = ref('')
    const notice = ref('')
    const pendingIds = ref(new Set<string>())
    const updateErrors = ref<Record<string, string>>({})
    const filters = ref({ emby_server_id: '', series_name: '', item_name: '' })
    const query = ref<PagePlayHistoryParam>({ page_number: 1, page_size: 10 })
    const pageCount = computed(() => Math.max(1, Math.ceil(total.value / query.value.page_size)))
    let requestId = 0
    let serverRequestId = 0
    let disposed = false
    let mutationVersion = 0
    const localMarks = new Map<string, { pinned: number, version: number }>()

    async function loadServers() {
        const id = ++serverRequestId
        serversLoading.value = true
        serversError.value = ''
        try {
            const response = await data.listServers()
            if (disposed || id !== serverRequestId) return
            servers.value = response.filter(server => server.id).sort((a, b) => (a.order_by ?? 0) - (b.order_by ?? 0))
        } catch {
            if (!disposed && id === serverRequestId) serversError.value = '服务器列表加载失败，可重试或继续按名称查询。'
        } finally {
            if (!disposed && id === serverRequestId) serversLoading.value = false
        }
    }

    async function loadHistory() {
        const id = ++requestId
        const snapshot = { ...query.value }
        const startedVersion = mutationVersion
        loading.value = true
        error.value = ''
        try {
            const [count, records] = await data.pageHistory(snapshot)
            if (disposed || id !== requestId) return
            total.value = count
            if (snapshot.page_number > pageCount.value) {
                query.value.page_number = pageCount.value
                await loadHistory()
                return
            }
            list.value = records.map(record => {
                const mark = record.id ? localMarks.get(record.id) : undefined
                return mark && mark.version > startedVersion ? { ...record, pinned: mark.pinned } : record
            })
            for (const [recordId, mark] of localMarks) {
                if (mark.version <= startedVersion) localMarks.delete(recordId)
            }
        } catch {
            if (!disposed && id === requestId) error.value = '播放历史加载失败，请重试。'
        } finally {
            if (!disposed && id === requestId) loading.value = false
        }
    }

    function applyFilters() {
        query.value = {
            page_number: 1,
            page_size: query.value.page_size,
            emby_server_id: filters.value.emby_server_id || undefined,
            series_name: filters.value.series_name.trim() || undefined,
            item_name: filters.value.item_name.trim() || undefined,
        }
        notice.value = ''
        return loadHistory()
    }

    function resetFilters() {
        filters.value = { emby_server_id: '', series_name: '', item_name: '' }
        return applyFilters()
    }

    function changePage(page: number) {
        if (loading.value || page < 1 || page > pageCount.value) return
        query.value.page_number = page
        return loadHistory()
    }

    async function togglePinned(row: PlayHistory) {
        if (!row.id || pendingIds.value.has(row.id) || disposed) return
        const id = row.id
        const pinned = row.pinned === 1 ? -1 : row.pinned === -1 ? 0 : 1
        pendingIds.value.add(id)
        delete updateErrors.value[id]
        notice.value = ''
        try {
            const affected = await data.updateHistory({ id, pinned })
            if (disposed) return
            if (affected < 1) throw new Error('History no longer exists')
            row.pinned = pinned
            localMarks.set(id, { pinned, version: ++mutationVersion })
            const current = list.value.find(record => record.id === id)
            if (current) current.pinned = pinned
            notice.value = `已${pinned === 1 ? '置顶' : pinned === -1 ? '标记为隐藏（未删除）' : '恢复正常'}。`
        } catch {
            if (!disposed) updateErrors.value[id] = '标记更新失败，记录未更改，请重试。'
        } finally {
            pendingIds.value.delete(id)
        }
    }

    onMounted(() => {
        void loadServers()
        void loadHistory()
    })
    onUnmounted(() => {
        disposed = true
        requestId++
        serverRequestId++
    })

    return {
        servers, serversLoading, serversError, list, total, loading, error, notice,
        pendingIds, updateErrors, filters, query, pageCount,
        loadServers, loadHistory, applyFilters, resetFilters, changePage, togglePinned,
    }
}
