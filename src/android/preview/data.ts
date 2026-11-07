import { reactive } from 'vue'
import type { PlayHistory } from '../../store/db/playHistory'
import type { HistoryData } from '../data/history'

export const previewState = reactive({
    delay: 350,
    failQuery: false,
    failServers: false,
    failUpdate: false,
    empty: false,
})

export const previewServers = [
    { id: 'demo-home', server_name: '家庭影音 · 模拟服务器', order_by: 0 },
    { id: 'demo-cinema', server_name: '电影收藏 · 模拟服务器', order_by: 1 },
]
const records: PlayHistory[] = Array.from({ length: 27 }, (_, index) => ({
    id: `demo-${index}`,
    emby_server_id: previewServers[index % 2].id,
    emby_server_name: previewServers[index % 2].server_name,
    item_type: 'Episode',
    series_id: `series-${Math.floor(index / 5)}`,
    series_name: index === 0 ? '漫长的旅途：那些关于星空、海洋与远方的故事——超长剧名与多语言标题测试 The Long Journey Beyond the Horizon' : `远方的故事 ${Math.floor(index / 5) + 1}`,
    item_id: `episode-${index}`,
    item_name: index === 0 ? '第 1 集 · 当我们启程时，世界仍在缓缓苏醒 / AVeryLongUnbrokenEpisodeTitleThatMustWrapWithoutHorizontalScrolling' : `第 ${index + 1} 集 · 新的旅程`,
    played_duration: 1230 + index * 391,
    pinned: index === 0 ? 1 : index === 26 ? -1 : 0,
    update_time: new Date(Date.UTC(2026, 9, 1, 12, 0, 27 - index)).toISOString(),
}))
records[1] = {
    ...records[1], item_type: 'Movie', series_id: undefined, series_name: undefined,
    item_name: '海边的夜晚 · 电影', played_duration: 7203,
}
records[2] = {
    ...records[2], id: undefined, emby_server_id: undefined, series_id: undefined,
    item_id: undefined, series_name: '缺失 ID 的旧记录', item_name: '仅展示，不可跳转或标记', played_duration: undefined,
}

async function delay() {
    await new Promise(resolve => setTimeout(resolve, previewState.delay))
}

export const previewData: HistoryData = {
    async listServers() {
        const fail = previewState.failServers
        await delay()
        if (fail) throw new Error('模拟服务器列表失败')
        return previewServers.map(server => ({ ...server }))
    },
    async pageHistory(query) {
        const fail = previewState.failQuery
        const empty = previewState.empty
        const response = records.filter(row =>
            (!query.emby_server_id || row.emby_server_id === query.emby_server_id) &&
            (!query.series_name || row.series_name?.includes(query.series_name)) &&
            (!query.item_name || row.item_name?.includes(query.item_name))
        ).sort((a, b) => (b.pinned ?? 0) - (a.pinned ?? 0) || (b.update_time ?? '').localeCompare(a.update_time ?? ''))
        const start = (query.page_number - 1) * query.page_size
        const page = response.slice(start, start + query.page_size).map(row => ({ ...row }))
        await delay()
        if (fail) throw new Error('模拟查询失败')
        return empty ? [0, []] : [response.length, page]
    },
    async updateHistory(row) {
        const fail = previewState.failUpdate
        await delay()
        if (fail) throw new Error('模拟更新失败')
        const current = records.find(record => record.id && record.id === row.id)
        if (!current) return 0
        current.pinned = row.pinned
        current.update_time = new Date().toISOString()
        return 1
    },
}
