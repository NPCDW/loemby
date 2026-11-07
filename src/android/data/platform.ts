import { inject } from 'vue'
import type { InjectionKey } from 'vue'
import type { BaseItem, EpisodeItem } from '../../api/embyApi'
import invokeApi from '../../api/invokeApi'
import { useImage } from '../../store/image'
import { useGlobalConfig } from '../../store/db/globalConfig'

export interface PlatformData {
    openUrl(url: string): Promise<void>
    logo(serverId: string, item: BaseItem): Promise<string | undefined>
    versionPolicy(): Promise<string>
}
export const platformDataKey: InjectionKey<PlatformData> = Symbol('android-platform-data')
const liveData: PlatformData = {
    openUrl: async url => { await invokeApi.open_url(url) },
    logo: async (server, item) => {
        const parent = (item as EpisodeItem).ParentLogoItemId
        if (!parent && !item.ImageTags?.Logo) return undefined
        const store = useImage()
        const key = `${server}:logo:${item.Id}`
        return store.images[key] || await store.loadImage(key, server, parent || item.Id, 'Logo')
    },
    versionPolicy: () => useGlobalConfig().getGlobalConfigValue('play_version_auto_select_policy'),
}
export function usePlatformData() { return inject(platformDataKey, liveData) }
