import { EmbyServer } from '../store/db/embyServer';
import { useGlobalConfig } from '../store/db/globalConfig';
import { invoke } from '@tauri-apps/api/core';

/**
 * 获取服务器信息，无需验证
 */
async function getServerInfo(emby_server_id: string): Promise<string> {
    if (!emby_server_id) {
        return Promise.reject("参数缺失");
    }
    return invoke('emby_get_server_info', {body: {
        emby_server_id
    }});
}

/**
 * 通过用户名密码授权
 */
async function authenticateByName(emby_server_id: string): Promise<string> {
    if (!emby_server_id) {
        return Promise.reject("参数缺失");
    }
    return invoke('emby_authenticate_by_name', {body: {
        emby_server_id
    }});
}

/**
 * 登出
 */
async function logout(emby_server_id: string): Promise<string> {
    return invoke('emby_logout', {body: {
        emby_server_id
    }});
}

/**
 * 搜索，该处问题：当搜索电影时，如果传入了搜索关键字和限制条数，搜索结果的总数会返回0（搜索返回的结果正常），不传这两个值又没办法分页
 * types: Movie,Series,Episode
 * @returns EmbyPageList<SearchItems>
 */
async function search(emby_server_id: string, search_str: string, item_types: string[], start_index: number, limit: number): Promise<string> {
    return invoke('emby_search', {body: {
        emby_server_id,
        search_str,
        item_types,
        start_index,
        limit
    }});
}

/**
 * 首页继续播放列表
 * @returns EmbyPageList<EpisodeItems>
 */
async function getContinuePlayList(emby_server_id: string, start_index: number, limit: number): Promise<string> {
    return invoke('emby_get_continue_play_list', {body: {
        emby_server_id,
        start_index,
        limit
    }});
}

/**
 * 收藏列表
 * @returns EmbyPageList<EpisodeItems>
 */
async function getFavoriteList(emby_server_id: string, start_index: number, limit: number): Promise<string> {
    return invoke('emby_get_favorite_list', {body: {
        emby_server_id,
        start_index,
        limit
    }});
}

/**
 * 系列剧集 接下来 应该播放的剧集
 * @returns EmbyPageList<EpisodeItems>
 */
async function nextUp(emby_server_id: string, series_id: string, start_index: number, limit: number): Promise<string> {
    return invoke('emby_next_up', {body: {
        emby_server_id,
        series_id,
        start_index,
        limit
    }});
}

/**
 * 首页媒体库列表
 * @returns EmbyPageList<MediaLibraryItem>
 */
async function getMediaLibraryList(emby_server_id: string): Promise<string> {
    return invoke('emby_get_media_library_list', {body: {
        emby_server_id,
    }});
}

/**
 * 首页媒体库子项目最新几条
 * @returns SearchItem[]
 */
async function getMediaLibraryChildLatest(emby_server_id: string, parent_id: string, limit: number): Promise<string> {
    return invoke('emby_get_media_library_child_latest', {body: {
        emby_server_id,
        parent_id,
        limit
    }});
}

/**
 * 首页媒体库子项目
 * @returns EmbyPageList<SearchItem>
 */
async function getMediaLibraryChild(emby_server_id: string, parent_id: string, start_index: number, limit: number): Promise<string> {
    return invoke('emby_get_media_library_child', {body: {
        emby_server_id,
        parent_id,
        start_index,
        limit
    }});
}

/**
 * 剧集数量统计
 * @returns MediaLibraryCount
 */
async function count(emby_server_id: string) {
    return invoke('emby_count', {body: {
        emby_server_id,
    }});
}

/**
 * 电影详情、剧集详情、季详情、系列详情、合集详情
 * @returns EpisodeItems
 */
async function items(emby_server_id: string, item_id: string) {
    return invoke('emby_items', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 系列 下的 季列表
 * @returns EmbyPageList<SeasonItem>
 */
async function seasons(emby_server_id: string, item_id: string): Promise<string> {
    return invoke('emby_seasons', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 季 下的 剧集列表
 * @returns EmbyPageList<EpisodeItems>
 */
async function episodes(emby_server_id: string, item_id: string, season_id: string, start_index: number, limit: number): Promise<string> {
    return invoke('emby_episodes', {body: {
        emby_server_id,
        item_id,
        season_id,
        start_index,
        limit
    }});
}

/**
 * 播放流媒体详情
 * @returns PlaybackInfo
 */
async function playbackInfo(emby_server_id: string, item_id: string): Promise<string> {
    return invoke('emby_playback_info', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 开始播放
 * @param positionTicks 播放位置 / 一千万 换算成秒 
 * @returns 204
 */
async function playing(emby_server_id: string, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number): Promise<string> {
    return invoke('emby_playing', {body: {
        emby_server_id,
        item_id,
        media_source_id,
        play_session_id,
        positionTicks
    }});
}

/**
 * 播放进度（保活），只要还在播放就需要定时调用播放进度，长时间没有播放进度，服务器会认为已经停止播放
 * @param positionTicks 播放位置 / 一千万 换算成秒 
 * @returns 204
 */
async function playingProgress(emby_server_id: string, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number): Promise<string> {
    return invoke('emby_playing_progress', {body: {
        emby_server_id,
        item_id,
        media_source_id,
        play_session_id,
        positionTicks
    }});
}

/**
 * 结束播放
 * @returns 204
 */
async function playingStopped(emby_server_id: string, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number): Promise<string> {
    return invoke('emby_playing_stopped', {body: {
        emby_server_id,
        item_id,
        media_source_id,
        play_session_id,
        positionTicks
    }});
}

/**
 * 组装直连视频流地址
 * @returns
 */
function getDirectStreamUrl(embyServer: EmbyServer, directStreamUrl: string) {
    if (!directStreamUrl) {
        return null;
    }
    return embyServer.base_url + "/emby" + directStreamUrl;
}

/**
 * 组装音频流地址，请确保音频流支持外部流，否则会加载整个视频
 * @returns
 */
function getAudioStreamUrl(embyServer: EmbyServer, item: EpisodeItem, mediaSource: MediaSource, mediaStreams: MediaStream) {
    if (!mediaStreams.IsExternal) {
        return null;
    }
    return embyServer.base_url + `/emby/Audio/${mediaSource.ItemId || item.Id}/stream.${mediaStreams.Codec}?AudioStreamIndex=${mediaStreams.Index}&Static=true`;
}

/**
 * 组装字幕流地址，请确保字幕流支持外部流
 * @returns
 */
function getSubtitleStreamUrl(embyServer: EmbyServer, item: EpisodeItem, mediaSource: MediaSource, mediaStreams: MediaStream) {
    if (!mediaStreams.IsExternal) {
        return null;
    }
    return embyServer.base_url + `/emby/Videos/${mediaSource.ItemId || item.Id}/${mediaSource.Id}/Subtitles/${mediaStreams.Index}/Stream.${mediaStreams.Codec}`;
}

/**
 * 组装图片地址
 * @returns
 */
async function getImageUrl(embyServer: EmbyServer, item_id: string, imageType: string = 'Primary') {
    const disabledImage = await useGlobalConfig().getGlobalConfigValue("disabledImage") || 'off'
    if (!item_id || disabledImage == 'on') {
        return null;
    }
    return embyServer.base_url + `/emby/Items/${item_id}/Images/${imageType}`;
}

/**
 * 收藏
 * @returns
 */
async function star(emby_server_id: string, item_id: string): Promise<string> {
    return invoke('emby_star', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 取消收藏
 * @returns
 */
async function unstar(emby_server_id: string, item_id: string): Promise<string> {
    return invoke('emby_unstar', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 标记已播放
 * @returns
 */
async function played(emby_server_id: string, item_id: string): Promise<string> {
    return invoke('emby_played', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 取消已播放
 * @returns
 */
async function unplayed(emby_server_id: string, item_id: string): Promise<string> {
    return invoke('emby_unplayed', {body: {
        emby_server_id,
        item_id,
    }});
}

/**
 * 隐藏继续观看记录
 * @returns
 */
async function hideFromResume(emby_server_id: string, item_id: string, hide: boolean): Promise<string> {
    return invoke('emby_hide_from_resume', {body: {
        emby_server_id,
        item_id,
        hide,
    }});
}

export default {
    getServerInfo, authenticateByName, logout, search, items, seasons, episodes, playbackInfo, playing, playingProgress, playingStopped, getContinuePlayList, nextUp,
    getFavoriteList, getDirectStreamUrl, getAudioStreamUrl, getSubtitleStreamUrl, star, unstar, played, unplayed, getMediaLibraryList, getMediaLibraryChildLatest,
    getImageUrl, count, hideFromResume, getMediaLibraryChild, 
}


export interface EmbyPageList<T> {
    TotalRecordCount: number,
    Items: T[]
}

export interface BaseItem {
    Id: string,
    Name: string,
    Type: string,
    ProductionYear: number,
    UserData?: UserData,
    Overview: string,
    ProviderIds: {[key: string]: string},
    ExternalUrls: ExternalUrl[],
    ImageTags: BaseItemImageTags,
}

export interface BaseItemImageTags {
    Primary: string,
    Art: string,
    Banner: string,
    Logo: string,
    Thumb: string,
}

export interface SeriesItem extends BaseItem {
    EndDate: string,
}

export interface SeasonItem extends BaseItem {
    SeriesId: string,
    SeriesName: string,
    Overview: string,
    IndexNumber: number,
    ParentLogoItemId: string,
    ParentThumbItemId: string,
    SeriesPrimaryImageTag: string,
}

export interface EpisodeItem extends BaseItem {
    SeriesName: string,
    PremiereDate: string,
    ParentIndexNumber: number,
    IndexNumber: number,
    MediaSources?: MediaSource[],    // 搜索时，zdz无媒体源字段
    SeriesId: string,
    ParentLogoItemId: string,
    ParentThumbItemId: string,
    SeriesPrimaryImageTag: string,
}

export type SearchItem = SeriesItem | SeasonItem | EpisodeItem

export interface PlaybackInfo {
    PlaySessionId: string,
    MediaSources: MediaSource[],
    ErrorCode?: string,
}

export interface MediaSource {
    Id: string,
    ItemId?: string, // 电影和剧集的id，一个剧集有多个媒体源，但每个媒体源的itemid不一样，但是用不同的itemid能查询到同一个剧集   ++nya无此字段
    Name: string,
    RunTimeTicks: number,
    Size: number,
    Bitrate: number,
    DirectStreamUrl: string,
    MediaStreams: MediaStream[],
    IsRemote: boolean,
    Path: string,
}

export interface MediaStream {
    // 视频、音频、字幕编码
    Codec: string,
    // 主标题
    DisplayTitle: string,
    DisplayLanguage: string,
    // 副标题
    Title: string,
    // 码率
    BitRate: number,
    Height: number,
    Width: number,
    Type: 'Video' | 'Audio' | 'Subtitle',
    Language: string,
    Index: number,
    IsDefault: boolean,
    IsExternal: boolean,
}

export interface UserData {
    PlayedPercentage: number,
    UnplayedItemCount: number,
    PlaybackPositionTicks: number,
    PlayCount: number,
    IsFavorite: boolean,
    Played: boolean,
}

export interface ExternalUrl {
    Url: string,
    Name: string,
}

export interface MediaLibraryItem {
    Name: string,
    Id: string,
    Type: string,
    ImageTags: BaseItemImageTags,
}

export interface MediaLibraryCount {
    MovieCount: number,
    SeriesCount: number,
    EpisodeCount: number,
    GameCount: number,
    ArtistCount: number,
    ProgramCount: number,
    GameSystemCount: number,
    TrailerCount: number,
    SongCount: number,
    AlbumCount: number,
    MusicVideoCount: number,
    BoxSetCount: number,
    BookCount: number,
    ItemCount: number,
}
