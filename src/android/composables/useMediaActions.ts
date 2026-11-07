import { onUnmounted, ref, watch } from 'vue'
import type { BaseItem } from '../../api/embyApi'
import { useBrowseData } from '../data/browse'
import { markUserDataChanged } from '../util/userData'

export function useMediaActions(serverId: () => string) {
    const data = useBrowseData()
    const pending = ref(new Set<string>())
    const errors = ref<Record<string, string>>({})
    let disposed = false
    let session = 0
    watch(serverId, () => {
        session++
        pending.value = new Set()
        errors.value = {}
    }, { flush: 'sync' })
    onUnmounted(() => { disposed = true; session++ })

    async function update(item: BaseItem, field: 'IsFavorite' | 'Played') {
        if (!item.Id || !item.UserData || pending.value.has(item.Id) || disposed) return
        const server = serverId()
        const currentSession = session
        const id = item.Id
        const next = !item.UserData[field]
        const active = () => !disposed && currentSession === session && server === serverId()
        pending.value.add(id)
        markUserDataChanged(item)
        delete errors.value[id]
        try {
            const response = await (field === 'IsFavorite' ? data.favorite(server, id, next) : data.played(server, id, next))
            if (!active()) return
            if (typeof response[field] !== 'boolean') throw new Error('Invalid user data')
            // 收藏不回写可能过期的观看进度；已看标记则同步服务端返回的进度与计数。
            item.UserData = field === 'IsFavorite'
                ? { ...item.UserData!, IsFavorite: response.IsFavorite }
                : { ...item.UserData!, ...response, IsFavorite: item.UserData!.IsFavorite }
        } catch {
            if (active()) errors.value[id] = '标记更新失败，原状态未更改，请重试。'
        } finally {
            if (active()) { markUserDataChanged(item); pending.value.delete(id) }
        }
    }
    return { pending, errors, favorite: (item: BaseItem) => update(item, 'IsFavorite'), played: (item: BaseItem) => update(item, 'Played') }
}
