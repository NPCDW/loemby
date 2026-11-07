import type { BaseItem, EmbyPageList, EpisodeItem, MediaLibraryCount, MediaLibraryItem, MediaSource, MediaStream, SearchItem, SeasonItem, UserData } from '../../api/embyApi'
import type { BrowseData } from '../data/browse'
import { previewServers, previewState } from './data'

// 模拟目录仅用于开发预览，不调用真实 API、数据库或网络。
const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value))
const imageTags = () => ({ Primary: 'preview', Art: '', Banner: '', Logo: '', Thumb: '' })
const userData = (index = 0): UserData => ({
    PlayedPercentage: index % 4 === 0 ? 38 : 0, UnplayedItemCount: 0,
    PlaybackPositionTicks: index % 4 === 0 ? 1230 * 10_000_000 : 0,
    PlayCount: index % 5 === 0 ? 1 : 0, IsFavorite: index % 3 === 0, Played: index % 5 === 0,
})
const longSeriesName = '漫长的旅途：那些关于星空、海洋与远方的故事——超长剧名与多语言标题测试 The Long Journey Beyond the Horizon'
const seriesName = (index: number) => index === 0 ? longSeriesName : `远方的故事 ${index + 1}`

function streams(uhd: boolean): MediaStream[] {
    return [
        { Codec: uhd ? 'hevc' : 'h264', DisplayTitle: uhd ? '4K HDR · 模拟视频' : '1080p · 模拟视频', DisplayLanguage: '', Title: '', BitRate: uhd ? 18_000_000 : 6_000_000, Height: uhd ? 2160 : 1080, Width: uhd ? 3840 : 1920, Type: 'Video', Language: 'und', Index: 0, IsDefault: true, IsExternal: false },
        { Codec: 'aac', DisplayTitle: '中文 · 立体声', DisplayLanguage: '中文', Title: '', BitRate: 192_000, Height: 0, Width: 0, Type: 'Audio', Language: 'zho', Index: 1, IsDefault: true, IsExternal: false },
        { Codec: 'aac', DisplayTitle: '英文 · 原声', DisplayLanguage: 'English', Title: '', BitRate: 192_000, Height: 0, Width: 0, Type: 'Audio', Language: 'eng', Index: 4, IsDefault: false, IsExternal: false },
        { Codec: 'srt', DisplayTitle: '英文 · 内封字幕', DisplayLanguage: 'English', Title: '', BitRate: 0, Height: 0, Width: 0, Type: 'Subtitle', Language: 'eng', Index: 7, IsDefault: true, IsExternal: false },
        { Codec: 'srt', DisplayTitle: '简体中文 · 外挂字幕', DisplayLanguage: 'Chinese Simplified / 简体中文', Title: '', BitRate: 0, Height: 0, Width: 0, Type: 'Subtitle', Language: 'zho', Index: 9, IsDefault: false, IsExternal: true },
    ]
}
function sources(id: string, movie: boolean): MediaSource[] {
    return [false, true].map(uhd => ({
        Id: `${id}-${uhd ? '4k' : '1080p'}`, ItemId: id, Name: uhd ? '4K 修复版 · 模拟' : '1080p 原始版 · 模拟',
        RunTimeTicks: (movie ? 7203 : 3240) * 10_000_000, Size: uhd ? 9_400_000_000 : 2_400_000_000,
        Bitrate: uhd ? 18_000_000 : 6_000_000, DirectStreamUrl: '', MediaStreams: streams(uhd),
        IsRemote: uhd, Path: uhd ? 'https://media.example.com/preview.mkv' : '', Container: 'mkv',
    }))
}
function item(id: string, name: string, type: string, index = 0): EpisodeItem {
    return {
        Id: id, Name: name, Type: type, ProductionYear: 2020 + index % 7, UserData: userData(index),
        Overview: `${name}\n这是完全离线的模拟媒体，用于验证 Android 浏览、标记、分页与长文本。不会连接真实服务器或启动播放。\n${'远方的故事从海岸开始，穿越星空与山川，记录平凡日子中的勇气与相遇。'.repeat(6)}`,
        ProviderIds: { Imdb: 'tt1234567', Tmdb: '98765' }, ExternalUrls: [{ Name: 'IMDb', Url: 'https://www.imdb.com/title/tt1234567/' }, { Name: 'TMDB', Url: 'https://www.themoviedb.org/movie/98765' }], ImageTags: imageTags(),
        Chapters: ['片头', '新的旅程', '片尾'].map((Name, ChapterIndex) => ({ Name, ChapterIndex, StartPositionTicks: ChapterIndex * 900 * 10_000_000, MarkerType: ChapterIndex === 0 ? 'IntroStart' : '' })),
        SeriesName: '', PremiereDate: '2026-10-01T00:00:00Z', ParentIndexNumber: 0, IndexNumber: 0,
        MediaSources: ['Movie', 'Episode'].includes(type) ? sources(id, type === 'Movie') : [],
        SeriesId: '', SeasonId: '', SeasonName: '', ParentLogoItemId: '', ParentThumbItemId: '', SeriesPrimaryImageTag: 'preview',
    }
}
interface Catalog {
    items: Map<string, EpisodeItem>
    seasons: Map<string, SeasonItem[]>
    libraries: MediaLibraryItem[]
    children: Map<string, string[]>
    hidden: Set<string>
}
function catalog(serverId: string): Catalog {
    const items = new Map<string, EpisodeItem>()
    const seasons = new Map<string, SeasonItem[]>()
    for (let series = 0; series < 6; series++) {
        const seriesId = `series-${series}`
        items.set(seriesId, item(seriesId, seriesName(series), 'Series', series))
        seasons.set(seriesId, [1, 0].map(IndexNumber => ({
            Id: `${seriesId}-season-${IndexNumber}`, Name: IndexNumber ? '第 1 季 · 启程' : '特别季 · 幕后与番外', Type: 'Season', ProductionYear: 2026,
            UserData: userData(series), Overview: IndexNumber ? '故事的开始，按集数浏览。' : '特别季使用第 0 季编号，独立浏览番外。',
            ProviderIds: {}, ExternalUrls: [], ImageTags: imageTags(), Chapters: [], SeriesId: seriesId, SeriesName: seriesName(series),
            IndexNumber, ParentLogoItemId: '', ParentThumbItemId: '', SeriesPrimaryImageTag: 'preview',
        })))
    }
    // 两个模拟服务器均能打开已有历史中的详情目标。
    for (let index = 0; index < 27; index++) {
        const series = Math.floor(index / 5)
        const media = item(`episode-${index}`, index === 1 ? '海边的夜晚 · 电影' : index === 0
            ? '第 1 集 · 当我们启程时，世界仍在缓缓苏醒 / AVeryLongUnbrokenEpisodeTitleThatMustWrapWithoutHorizontalScrolling'
            : `第 ${index + 1} 集 · 新的旅程`, index === 1 ? 'Movie' : 'Episode', index)
        if (media.Type === 'Episode') Object.assign(media, {
            SeriesId: `series-${series}`, SeriesName: seriesName(series), SeasonId: `series-${series}-season-1`,
            SeasonName: '第 1 季 · 启程', ParentIndexNumber: 1, IndexNumber: index % 5 + 1,
        })
        items.set(media.Id, media)
    }
    for (let series = 0; series < 6; series++) {
        for (let index = 0; index < (series === 0 ? 16 : 2); index++) {
            const special = index < 2
            const media = item(`series-${series}-${special ? 'special' : 'extra'}-${index}`, special ? `番外 ${index + 1} · 星空下的来信` : `第 ${index + 4} 集 · 更远的旅程`, 'Episode', index + 1)
            Object.assign(media, { SeriesId: `series-${series}`, SeriesName: seriesName(series), SeasonId: `series-${series}-season-${special ? 0 : 1}`, SeasonName: special ? '特别季 · 幕后与番外' : '第 1 季 · 启程', ParentIndexNumber: special ? 0 : 1, IndexNumber: special ? index + 1 : index + 4 })
            items.set(media.Id, media)
        }
    }
    for (let index = 0; index < 31; index++) {
        const media = item(`movie-${index}`, index === 0 ? '无媒体源的电影 · 简介仍可浏览' : index === 1 ? '缺少 UserData · 标记不可用' : `${serverId === 'demo-cinema' ? '电影收藏' : '家庭影院'} ${index + 1} · 海风与星光`, 'Movie', index)
        if (index === 0) delete media.MediaSources
        if (index === 1) delete media.UserData
        items.set(media.Id, media)
    }
    for (const [id, name, type] of [
        ['collection-journey', '远方的故事 · 嵌套合集', 'BoxSet'],
        ['collection-specials', '番外与幕后 · 子合集', 'BoxSet'],
        ['folder-archive', '珍藏目录 · 多层浏览', 'Folder'],
    ]) items.set(id, item(id, name, type))
    const libraries: MediaLibraryItem[] = [
        { Id: 'library-movies', Name: '电影库 · 分页测试（32 部）', Type: 'CollectionFolder', CollectionType: 'movies', ImageTags: imageTags() },
        { Id: 'library-series', Name: '电视剧与特别季', Type: 'CollectionFolder', CollectionType: 'tvshows', ImageTags: imageTags() },
        { Id: 'library-collections', Name: '合集与目录 · 嵌套浏览', Type: 'CollectionFolder', CollectionType: 'boxsets', ImageTags: imageTags() },
    ]
    for (const library of libraries) items.set(library.Id, item(library.Id, library.Name, library.Type))
    const values = Array.from(items.values())
    for (const values of seasons.values()) for (const season of values) {
        const episodes = Array.from(items.values()).filter(media => media.Type === 'Episode' && media.SeasonId === season.Id)
        season.UserData!.UnplayedItemCount = episodes.filter(media => !media.UserData?.Played).length
        items.set(season.Id, season as EpisodeItem)
    }
    const children = new Map<string, string[]>([
        ['library-movies', values.filter(value => value.Type === 'Movie').map(value => value.Id)],
        ['library-series', values.filter(value => value.Type === 'Series').map(value => value.Id)],
        ['library-collections', ['collection-journey', 'folder-archive']],
        ['collection-journey', ['series-0', 'episode-1', 'collection-specials']],
        ['collection-specials', ['series-0-special-0', 'series-0-special-1', 'movie-0', 'movie-1']],
        ['folder-archive', ['collection-specials', 'movie-2', 'movie-3']],
    ])
    return { items, seasons, libraries, children, hidden: new Set() }
}
const catalogs = new Map(previewServers.map(server => [server.id, catalog(server.id)]))
function getCatalog(serverId: string) {
    const result = catalogs.get(serverId)
    if (!result) throw new Error('模拟服务器不存在；预览绝不回退到真实数据')
    return result
}
async function response<T>(build: (empty: boolean) => T, failure: 'failQuery' | 'failServers' | 'failUpdate' = 'failQuery'): Promise<T> {
    const fail = previewState[failure]
    const result = fail ? undefined : clone(build(previewState.empty))
    await new Promise(resolve => setTimeout(resolve, previewState.delay))
    if (fail) throw new Error('模拟请求失败，请关闭故障开关后重试')
    return result as T
}
function page<T>(values: T[], start: number, limit: number, empty: boolean): EmbyPageList<T> {
    return { TotalRecordCount: empty ? 0 : values.length, Items: empty ? [] : values.slice(Math.max(0, start), Math.max(0, start) + Math.max(0, limit)) }
}
function counts(values: EpisodeItem[]): MediaLibraryCount {
    const count = (type: string) => values.filter(value => value.Type === type).length
    return { MovieCount: count('Movie'), SeriesCount: count('Series'), EpisodeCount: count('Episode'), GameCount: 0, ArtistCount: 0, ProgramCount: 0, GameSystemCount: 0, TrailerCount: 0, SongCount: 0, AlbumCount: 0, MusicVideoCount: 0, BoxSetCount: count('BoxSet'), BookCount: 0, ItemCount: values.length }
}
async function update(serverId: string, itemId: string, field: 'IsFavorite' | 'Played', value: boolean): Promise<UserData> {
    const media = getCatalog(serverId).items.get(itemId)
    if (!media?.UserData) throw new Error('模拟媒体没有 UserData')
    // 请求成功后才更新模拟目录，失败保留原状态。
    const next = await response(() => ({ ...media.UserData!, [field]: value }), 'failUpdate')
    media.UserData = next
    if (field === 'Played') {
        media.UserData.PlaybackPositionTicks = 0
        media.UserData.PlayedPercentage = value ? 100 : 0
        const catalog = getCatalog(serverId)
        if (media.Type === 'Season' || media.Type === 'Series') {
            for (const episode of catalog.items.values()) {
                if (episode.Type === 'Episode' && (media.Type === 'Season' ? episode.SeasonId === itemId : episode.SeriesId === itemId) && episode.UserData) {
                    episode.UserData.Played = value
                    episode.UserData.PlaybackPositionTicks = 0
                    episode.UserData.PlayedPercentage = value ? 100 : 0
                }
            }
        }
        for (const seasons of catalog.seasons.values()) for (const season of seasons) {
            season.UserData!.UnplayedItemCount = Array.from(catalog.items.values()).filter(episode => episode.Type === 'Episode' && episode.SeasonId === season.Id && !episode.UserData?.Played).length
        }
    }
    return clone(media.UserData)
}
function escapeXml(value: string) {
    return value.replace(/[<>&"']/g, character => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;', "'": '&apos;' }[character]!))
}
function poster(serverId: string, media: BaseItem | MediaLibraryItem) {
    const category = ({ Movie: '电影', Series: '系列', Episode: '剧集', Season: '季', BoxSet: '合集', Folder: '目录', CollectionFolder: '媒体库' } as Record<string, string>)[media.Type] || '媒体'
    const colors = serverId === 'demo-cinema' ? ['#722c46', '#231e48'] : ['#145d73', '#152548']
    const title = Array.from(media.Name).slice(0, 28).join('')
    const lines = [Array.from(title).slice(0, 14).join(''), Array.from(title).slice(14).join('')]
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="400" height="600" viewBox="0 0 400 600"><title>${escapeXml(media.Name)} · 模拟${category}海报</title><defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="${colors[0]}"/><stop offset="1" stop-color="${colors[1]}"/></linearGradient></defs><rect width="400" height="600" fill="url(#bg)"/><circle cx="325" cy="140" r="85" fill="#fff" opacity=".12"/><path d="M0 370L100 250L210 385L300 305L400 420V600H0Z" fill="#fff" opacity=".12"/><rect x="30" y="32" width="120" height="44" rx="10" fill="#fff" opacity=".16"/><g fill="#fff" font-family="sans-serif"><text x="44" y="62" font-size="22">${category} · 模拟</text><text x="32" y="470" font-size="25" font-weight="700">${escapeXml(lines[0])}</text><text x="32" y="508" font-size="25" font-weight="700">${escapeXml(lines[1])}</text><text x="32" y="558" font-size="16">LOEMBY · 离线开发预览</text></g></svg>`
    return `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(svg)}`
}
export const previewBrowseData: BrowseData = {
    servers: () => response(() => previewServers, 'failServers'),
    resume: (server, start, limit) => response(empty => {
        const current = getCatalog(server)
        return page(Array.from(current.items.values()).filter(media => ['Episode', 'Movie'].includes(media.Type) && media.UserData && !media.UserData.Played && media.UserData.PlayedPercentage > 0 && !current.hidden.has(media.Id)), start, limit, empty)
    }),
    favorites: (server, start, limit) => response(empty => page<SearchItem>(Array.from(getCatalog(server).items.values()).filter(media => media.UserData?.IsFavorite), start, limit, empty)),
    count: server => response(empty => counts(empty ? [] : Array.from(getCatalog(server).items.values()))),
    libraries: server => response(empty => page(getCatalog(server).libraries, 0, 100, empty)),
    latest: (server, parent, limit) => response(empty => empty ? [] : (getCatalog(server).children.get(parent) || []).slice(-limit).reverse().map(id => getCatalog(server).items.get(id)!)),
    children: (server, parent, start, limit) => response(empty => page<SearchItem>((getCatalog(server).children.get(parent) || []).map(id => getCatalog(server).items.get(id)!), start, limit, empty)),
    item: (server, id) => response(empty => {
        const result = getCatalog(server).items.get(id)
        if (empty || !result) throw new Error('模拟媒体不存在或空数据场景')
        return result
    }),
    seasons: (server, series) => response(empty => page(getCatalog(server).seasons.get(series) || [], 0, 100, empty)),
    episodes: (server, series, season, start, limit) => response(empty => page(Array.from(getCatalog(server).items.values()).filter(media => media.Type === 'Episode' && media.SeriesId === series && (!season || media.SeasonId === season)), start, limit, empty)),
    favorite: (server, id, value) => update(server, id, 'IsFavorite', value),
    played: (server, id, value) => update(server, id, 'Played', value),
    hideResume: async (server, id, hide) => {
        const current = getCatalog(server)
        if (!current.items.has(id)) throw new Error('模拟媒体不存在')
        await response(() => true, 'failUpdate')
        if (hide) current.hidden.add(id)
        else current.hidden.delete(id)
    },
    cover: async (server, media) => { getCatalog(server); return poster(server, media) },
}
