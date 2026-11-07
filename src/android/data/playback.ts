import { inject } from 'vue'
import type { InjectionKey } from 'vue'
import invokeApi from '../../api/invokeApi'
import type { PlayVideoParam } from '../../api/invokeApi'
import { useEventBus } from '../../store/eventBus'
import type { PlaybackNotifyParam } from '../../store/notifyCenter'

export type PlaybackNotification = Pick<PlaybackNotifyParam, 'emby_server_id' | 'item_id' | 'event'> & Partial<Pick<PlaybackNotifyParam, 'series_id'>>

// 与 browseDataKey 一样，预览和测试注入 adapter，不调用真实 Tauri/事件总线。
export interface PlaybackData {
    play(params: PlayVideoParam): Promise<void>
    subscribe(listener: (notification: PlaybackNotification) => void): () => void
}

export const playbackDataKey: InjectionKey<PlaybackData> = Symbol('android-playback-data')
const liveData: PlaybackData = {
    play: params => invokeApi.call_player(params),
    subscribe: listener => {
        const bus = useEventBus()
        bus.on('playingNotify', listener)
        return () => bus.remove('playingNotify', listener)
    },
}

export function usePlaybackData() {
    return inject(playbackDataKey, liveData)
}
