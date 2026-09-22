import { invoke } from '@tauri-apps/api/core';

/**
 * Emby 接口层。
 *
 * Rust 侧把这些接口的响应作为 JSON 字符串返回，这里统一在边界处解析成类型化对象，
 * 页面只面对对象，不再散落 JSON.parse。
 *
 * 约定：所有函数返回 Promise<T>，失败时 reject 出字符串错误。
 */

/** 在边界处把 Rust 返回的 JSON 字符串解析成对象 */
function parse<T>(raw: unknown): T {
    if (typeof raw === 'string') {
        return JSON.parse(raw) as T;
    }
    return raw as T;
}

async function getServerInfo(embyServerId: string): Promise<ServerInfo> {
    return parse<ServerInfo>(await invoke('emby_get_server_info', { body: { emby_server_id: embyServerId } }));
}

async function authenticateByName(embyServerId: string): Promise<AuthenticateResult> {
    return parse<AuthenticateResult>(await invoke('emby_authenticate_by_name', { body: { emby_server_id: embyServerId } }));
}

async function logout(embyServerId: string): Promise<string> {
    return invoke('emby_logout', { body: { emby_server_id: embyServerId } });
}

/**
 * 搜索。types: Movie / Series / Episode
 *
 * 已知上游问题：传入 search_str 与 limit 时 TotalRecordCount 可能为 0，结果本身正常。
 */
async function search(
    embyServerId: string,
    searchStr: string,
    itemTypes: string[],
    startIndex: number,
    limit: number,
): Promise<EmbyPageList<SearchItem>> {
    return parse<EmbyPageList<SearchItem>>(await invoke('emby_search', {
        body: { emby_server_id: embyServerId, search_str: searchStr, item_types: itemTypes, start_index: startIndex, limit },
    }));
}

/** 继续观看 */
async function getContinuePlayList(embyServerId: string, startIndex: number, limit: number): Promise<EmbyPageList<EpisodeItem>> {
    return parse<EmbyPageList<EpisodeItem>>(await invoke('emby_get_continue_play_list', {
        body: { emby_server_id: embyServerId, start_index: startIndex, limit },
    }));
}

/** 收藏 */
async function getFavoriteList(embyServerId: string, startIndex: number, limit: number): Promise<EmbyPageList<SearchItem>> {
    return parse<EmbyPageList<SearchItem>>(await invoke('emby_get_favorite_list', {
        body: { emby_server_id: embyServerId, start_index: startIndex, limit },
    }));
}

/** 剧集的「接下来」（按剧自己的排序，特别季不一定按序） */
async function nextUp(embyServerId: string, seriesId: string, startIndex: number, limit: number): Promise<EmbyPageList<EpisodeItem>> {
    return parse<EmbyPageList<EpisodeItem>>(await invoke('emby_next_up', {
        body: { emby_server_id: embyServerId, series_id: seriesId, start_index: startIndex, limit },
    }));
}

async function getMediaLibraryList(embyServerId: string): Promise<EmbyPageList<MediaLibraryItem>> {
    return parse<EmbyPageList<MediaLibraryItem>>(await invoke('emby_get_media_library_list', {
        body: { emby_server_id: embyServerId },
    }));
}

/** 媒体库下最新的若干条 */
async function getMediaLibraryChildLatest(embyServerId: string, parentId: string, limit: number): Promise<SearchItem[]> {
    return parse<SearchItem[]>(await invoke('emby_get_media_library_child_latest', {
        body: { emby_server_id: embyServerId, parent_id: parentId, limit },
    }));
}

async function getMediaLibraryChild(embyServerId: string, parentId: string, startIndex: number, limit: number): Promise<EmbyPageList<SearchItem>> {
    return parse<EmbyPageList<SearchItem>>(await invoke('emby_get_media_library_child', {
        body: { emby_server_id: embyServerId, parent_id: parentId, start_index: startIndex, limit },
    }));
}

async function count(embyServerId: string): Promise<MediaLibraryCount> {
    return parse<MediaLibraryCount>(await invoke('emby_count', { body: { emby_server_id: embyServerId } }));
}

/**
 * 条目详情。
 *
 * 同一个接口返回电影 / 剧 / 季 / 合集 / 单集，字段是并集，
 * 因此这里返回宽类型，由调用方按 Type 收窄（见 asSeries / asEpisode）。
 */
async function items(embyServerId: string, itemId: string): Promise<EpisodeItem> {
    return parse<EpisodeItem>(await invoke('emby_items', { body: { emby_server_id: embyServerId, item_id: itemId } }));
}

/** 把详情收窄成剧集形态（只有 Type 为 Series 时字段才完整） */
function asSeries(item: EpisodeItem): SeriesItem {
    return item as unknown as SeriesItem;
}

/** 把详情收窄成合集 / 文件夹形态 */
function asLibraryItem(item: EpisodeItem): MediaLibraryItem {
    return item as unknown as MediaLibraryItem;
}

async function seasons(embyServerId: string, seriesId: string): Promise<EmbyPageList<SeasonItem>> {
    return parse<EmbyPageList<SeasonItem>>(await invoke('emby_seasons', {
        body: { emby_server_id: embyServerId, series_id: seriesId },
    }));
}

async function episodes(
    embyServerId: string,
    seriesId: string,
    seasonId: string,
    startIndex: number,
    limit: number,
    startItemId?: string,
): Promise<EmbyPageList<EpisodeItem>> {
    return parse<EmbyPageList<EpisodeItem>>(await invoke('emby_episodes', {
        body: {
            emby_server_id: embyServerId,
            series_id: seriesId,
            season_id: seasonId,
            start_item_id: startItemId,
            start_index: startIndex,
            limit,
        },
    }));
}

async function playbackInfo(embyServerId: string, itemId: string): Promise<PlaybackInfo> {
    return parse<PlaybackInfo>(await invoke('emby_playback_info', {
        body: { emby_server_id: embyServerId, item_id: itemId },
    }));
}

async function star(embyServerId: string, itemId: string): Promise<UserData> {
    return parse<UserData>(await invoke('emby_star', { body: { emby_server_id: embyServerId, item_id: itemId } }));
}

async function unstar(embyServerId: string, itemId: string): Promise<UserData> {
    return parse<UserData>(await invoke('emby_unstar', { body: { emby_server_id: embyServerId, item_id: itemId } }));
}

async function played(embyServerId: string, itemId: string): Promise<UserData> {
    return parse<UserData>(await invoke('emby_played', { body: { emby_server_id: embyServerId, item_id: itemId } }));
}

async function unplayed(embyServerId: string, itemId: string): Promise<UserData> {
    return parse<UserData>(await invoke('emby_unplayed', { body: { emby_server_id: embyServerId, item_id: itemId } }));
}

/** 从继续观看中移除 */
async function hideFromResume(embyServerId: string, itemId: string, hide: boolean): Promise<string> {
    return invoke('emby_hide_from_resume', { body: { emby_server_id: embyServerId, item_id: itemId, hide } });
}

/**
 * 组装直连视频流地址
 */
function getDirectStreamUrl(directStreamUrl: string): string | null {
    if (!directStreamUrl) {
        return null;
    }
    return '/emby' + directStreamUrl;
}

/**
 * 组装视频流地址
 */
function getVideoStreamUrl(item: BaseItem, mediaSource: MediaSource, playSessionId: string): string | null {
    if (!item || !mediaSource) {
        return null;
    }
    return `/emby/Videos/${item.Id}/stream.${mediaSource.Container}?Static=true&mediaSourceId=${mediaSource.Id}&playSessionId=${playSessionId}`;
}

/**
 * 组装音频流地址。请确保音频流支持外部流，否则会加载整个视频。
 */
function getAudioStreamUrl(item: BaseItem, mediaSource: MediaSource, mediaStream: MediaStream): string | null {
    if (!mediaStream.IsExternal) {
        return null;
    }
    return `/emby/Audio/${mediaSource.ItemId || item.Id}/stream.${mediaStream.Codec}?AudioStreamIndex=${mediaStream.Index}&Static=true`;
}

/**
 * 组装字幕流地址。请确保字幕流支持外部流。
 */
function getSubtitleStreamUrl(item: BaseItem, mediaSource: MediaSource, mediaStream: MediaStream): string | null {
    if (!mediaStream.IsExternal) {
        return null;
    }
    return `/emby/Videos/${mediaSource.ItemId || item.Id}/${mediaSource.Id}/Subtitles/${mediaStream.Index}/Stream.${mediaStream.Codec}`;
}

export default {
    getServerInfo,
    authenticateByName,
    logout,
    search,
    items,
    seasons,
    episodes,
    playbackInfo,
    getContinuePlayList,
    nextUp,
    getFavoriteList,
    getDirectStreamUrl,
    getVideoStreamUrl,
    getAudioStreamUrl,
    getSubtitleStreamUrl,
    star,
    unstar,
    played,
    unplayed,
    getMediaLibraryList,
    getMediaLibraryChildLatest,
    count,
    hideFromResume,
    getMediaLibraryChild,
    asSeries,
    asLibraryItem,
};

/* —————————————————————————— 类型 —————————————————————————— */

/** 条目类型，Emby 侧的字符串字面量 */
export type ItemType = 'Movie' | 'Series' | 'Season' | 'Episode' | 'BoxSet' | string;

export interface EmbyPageList<T> {
    TotalRecordCount: number;
    Items: T[];
}

export interface UserData {
    PlayedPercentage: number;
    UnplayedItemCount: number;
    PlaybackPositionTicks: number;
    PlayCount: number;
    IsFavorite: boolean;
    Played: boolean;
}

export interface ExternalUrl {
    Url: string;
    Name: string;
}

export interface Chapter {
    StartPositionTicks: number;
    Name: string;
    MarkerType: string;
    ChapterIndex: number;
}

export interface BaseItemImageTags {
    Primary?: string;
    Art?: string;
    Banner?: string;
    Logo?: string;
    Thumb?: string;
}

export interface BaseItem {
    Id: string;
    Name: string;
    Type: ItemType;
    ProductionYear: number;
    UserData?: UserData;
    Overview: string;
    ProviderIds: { [key: string]: string };
    ExternalUrls: ExternalUrl[];
    ImageTags: BaseItemImageTags;
    Chapters: Chapter[];
}

export interface SeriesItem extends BaseItem {
    EndDate: string;
    OfficialRating?: string;
    Genres?: string[];
}

export interface SeasonItem extends BaseItem {
    SeriesId: string;
    SeriesName: string;
    IndexNumber: number;
    ParentLogoItemId: string;
    ParentThumbItemId: string;
    SeriesPrimaryImageTag: string;
}

export interface EpisodeItem extends BaseItem {
    SeriesName: string;
    PremiereDate: string;
    ParentIndexNumber: number;
    IndexNumber: number;
    /** 搜索时可能没有媒体源字段 */
    MediaSources?: MediaSource[];
    SeriesId: string;
    SeasonId: string;
    SeasonName: string;
    ParentLogoItemId: string;
    ParentThumbItemId: string;
    SeriesPrimaryImageTag: string;
}

/** 列表里出现的条目，可能是电影 / 剧 / 季 / 单集 */
export type SearchItem = SeriesItem | SeasonItem | EpisodeItem;

export interface PlaybackInfo {
    PlaySessionId: string;
    MediaSources: MediaSource[];
    ErrorCode?: string;
}

export interface MediaSource {
    Id: string;
    /** 电影与剧集的 id；同一剧集的多个媒体源 ItemId 相同 */
    ItemId?: string;
    Name: string;
    RunTimeTicks: number;
    Size: number;
    Bitrate: number;
    DirectStreamUrl: string;
    MediaStreams: MediaStream[];
    IsRemote: boolean;
    Path: string;
    Container: string;
}

export interface MediaStream {
    Codec: string;
    DisplayTitle: string;
    DisplayLanguage: string;
    Title: string;
    BitRate: number;
    Height: number;
    Width: number;
    Type: 'Video' | 'Audio' | 'Subtitle';
    Language: string;
    Index: number;
    IsDefault: boolean;
    IsExternal: boolean;
}

export interface MediaLibraryItem {
    Name: string;
    Id: string;
    Type: ItemType;
    CollectionType: string;
    ImageTags: BaseItemImageTags;
}

export interface MediaLibraryCount {
    MovieCount: number;
    SeriesCount: number;
    EpisodeCount: number;
    GameCount: number;
    ArtistCount: number;
    ProgramCount: number;
    GameSystemCount: number;
    TrailerCount: number;
    SongCount: number;
    AlbumCount: number;
    MusicVideoCount: number;
    BoxSetCount: number;
    BookCount: number;
    ItemCount: number;
}

export interface ServerInfo {
    ServerName: string;
    Id: string;
}

export interface AuthenticateResult {
    User: { Id: string };
    AccessToken: string;
}
