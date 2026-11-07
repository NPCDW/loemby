import { inject } from 'vue'
import type { InjectionKey } from 'vue'
import embyApi from '../../api/embyApi'
import type { BaseItem, EmbyPageList, EpisodeItem, MediaLibraryCount, MediaLibraryItem, SearchItem, SeasonItem, UserData } from '../../api/embyApi'
import { useEmbyServer } from '../../store/db/embyServer'
import type { EmbyServer } from '../../store/db/embyServer'
import { useImage } from '../../store/image'

export interface BrowseData {
    servers(): Promise<EmbyServer[]>
    resume(serverId: string, start: number, limit: number): Promise<EmbyPageList<EpisodeItem>>
    favorites(serverId: string, start: number, limit: number): Promise<EmbyPageList<SearchItem>>
    count(serverId: string): Promise<MediaLibraryCount>
    libraries(serverId: string): Promise<EmbyPageList<MediaLibraryItem>>
    latest(serverId: string, parentId: string, limit: number): Promise<SearchItem[]>
    children(serverId: string, parentId: string, start: number, limit: number): Promise<EmbyPageList<SearchItem>>
    item(serverId: string, itemId: string): Promise<EpisodeItem>
    seasons(serverId: string, seriesId: string): Promise<EmbyPageList<SeasonItem>>
    episodes(serverId: string, seriesId: string, seasonId: string, start: number, limit: number): Promise<EmbyPageList<EpisodeItem>>
    favorite(serverId: string, itemId: string, value: boolean): Promise<UserData>
    played(serverId: string, itemId: string, value: boolean): Promise<UserData>
    hideResume(serverId: string, itemId: string, hide: boolean): Promise<void>
    cover(serverId: string, item: BaseItem | MediaLibraryItem): Promise<string | undefined>
}

export const browseDataKey: InjectionKey<BrowseData> = Symbol('android-browse-data')
const parse = async <T>(response: Promise<string>): Promise<T> => JSON.parse(await response)
const liveData: BrowseData = {
    servers: () => useEmbyServer().listAllEmbyServer(),
    resume: (server, start, limit) => parse(embyApi.getContinuePlayList(server, start, limit)),
    favorites: (server, start, limit) => parse(embyApi.getFavoriteList(server, start, limit)),
    count: server => parse(embyApi.count(server)),
    libraries: server => parse(embyApi.getMediaLibraryList(server)),
    latest: (server, parent, limit) => parse(embyApi.getMediaLibraryChildLatest(server, parent, limit)),
    children: (server, parent, start, limit) => parse(embyApi.getMediaLibraryChild(server, parent, start, limit)),
    item: (server, item) => parse(embyApi.items(server, item)),
    seasons: (server, series) => parse(embyApi.seasons(server, series)),
    episodes: (server, series, season, start, limit) => parse(embyApi.episodes(server, series, season, start, limit)),
    favorite: (server, item, value) => parse(value ? embyApi.star(server, item) : embyApi.unstar(server, item)),
    played: (server, item, value) => parse(value ? embyApi.played(server, item) : embyApi.unplayed(server, item)),
    hideResume: async (server, item, hide) => { await embyApi.hideFromResume(server, item, hide) },
    cover: async (server, item) => {
        const store = useImage()
        await store.loadCover(server, item)
        return store.images[`${server}:cover:${item.Id}`]
    },
}

export function useBrowseData() {
    return inject(browseDataKey, liveData)
}
