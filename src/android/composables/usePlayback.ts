import { onMounted, onUnmounted, ref, watch } from 'vue'
import type { EpisodeItem } from '../../api/embyApi'
import { useBrowseData } from '../data/browse'
import { usePlaybackData } from '../data/playback'
import type { PlaybackNotification } from '../data/playback'
import type { PlaybackSelection } from '../util/playbackSelection'
import { mergeUserData, userDataRevision } from '../util/userData'

export function usePlayback(options: {
    serverId: () => string
    itemId: () => string
    item: () => EpisodeItem | undefined
    selection: () => PlaybackSelection
    followEpisode: (notification: PlaybackNotification) => Promise<unknown>
}) {
    const data = useBrowseData()
    const playback = usePlaybackData()
    const preparing = ref(false)
    const error = ref('')
    const refreshing = ref(false)
    const refreshError = ref('')
    let session = 0
    let refreshRequest = 0
    let disposed = false
    let unsubscribe: (() => void) | undefined

    // 每次路由变化均开启新页面 session，即使 A → B → A，旧 Promise 也无权更新状态。
    watch(() => [options.serverId(), options.itemId()], () => {
        session++
        refreshRequest++
        preparing.value = refreshing.value = false
        error.value = refreshError.value = ''
    }, { flush: 'sync' })

    async function play(resume = false, startTicks?: number): Promise<void> {
        const item = options.item()
        const serverId = options.serverId()
        if (disposed || preparing.value || !serverId || !item?.Id || item.Id !== options.itemId()) return
        const currentSession = session
        const selection = options.selection()
        const index = selection.versionIndex
        const validIndex = Number.isInteger(index) && index >= 0 && index < (item.MediaSources?.length || 0) ? index : 0
        const position = startTicks ?? (resume ? item.UserData?.PlaybackPositionTicks : 0)
        preparing.value = true
        error.value = ''
        try {
            await playback.play({
                emby_server_id: serverId,
                series_id: item.SeriesId,
                item_id: item.Id,
                playback_position_ticks: typeof position === 'number' && Number.isFinite(position) && position > 0 ? Math.floor(position) : 0,
                use_direct_link: selection.useDirectLink,
                select_policy: selection.rememberSelect ? 'manual' : 'auto',
                video_select: selection.videoSelect,
                audio_select: selection.audioSelect,
                subtitle_select: selection.subtitleSelect,
                version_select: validIndex + 1,
                download: false,
            })
        } catch (reason) {
            if (!disposed && currentSession === session) {
                const detail = reason instanceof Error ? reason.message : typeof reason === 'string' ? reason : ''
                error.value = `播放准备失败${detail ? `：${detail}` : ''}，请重试。`
            }
        } finally {
            if (!disposed && currentSession === session) preparing.value = false
        }
    }

    async function refreshProgress() {
        const item = options.item()
        const serverId = options.serverId()
        const itemId = options.itemId()
        if (disposed || !serverId || item?.Id !== itemId) return
        const currentSession = session
        const request = ++refreshRequest
        const revision = userDataRevision(item)
        const active = () => !disposed && currentSession === session && request === refreshRequest && item === options.item()
        refreshing.value = true
        refreshError.value = ''
        try {
            const result = await data.item(serverId, itemId)
            if (!active()) return
            if (result.Id !== itemId) throw new Error('Unexpected media item')
            // 保留版本/轨道，标记操作期间的旧进度响应无权覆盖新状态。
            if (!mergeUserData(item, result.UserData, revision)) refreshError.value = '观看标记已更新，请重试刷新进度。'
        } catch {
            if (active()) refreshError.value = '观看进度刷新失败，请重试。'
        } finally {
            if (!disposed && currentSession === session && request === refreshRequest) refreshing.value = false
        }
    }

    async function onNotification(notification: PlaybackNotification) {
        if (disposed || notification.emby_server_id !== options.serverId()) return
        if (notification.event === 'stop' && notification.item_id === options.itemId()) await refreshProgress()
        else if (notification.event === 'start' && notification.item_id !== options.itemId() && notification.series_id && notification.series_id === options.item()?.SeriesId) {
            try { await options.followEpisode(notification) }
            catch { if (!disposed) error.value = '切换单集详情失败，请重新打开详情。' }
        }
    }

    onMounted(() => { unsubscribe = playback.subscribe(notification => { void onNotification(notification) }) })
    onUnmounted(() => {
        disposed = true
        session++
        refreshRequest++
        unsubscribe?.()
    })
    return { preparing, error, refreshing, refreshError, play, refreshProgress }
}
