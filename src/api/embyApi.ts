import { EmbyServer } from '../store/db/embyServer';
import { useProxyServer } from '../store/db/proxyServer';
import invoke from './invoke';

/**
 * 获取服务器信息，无需验证
 */
async function getServerInfo(embyServer: EmbyServer) {
    if (!embyServer.base_url) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + '/emby/System/Info/Public',
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 通过用户名密码授权
 */
async function authenticateByName(embyServer: EmbyServer) {
    if (!embyServer.base_url || !embyServer.username) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + '/emby/Users/AuthenticateByName',
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Authorization': `Emby Client="${embyServer.client}", Device="${embyServer.device}", DeviceId="${embyServer.device_id}", Version="${embyServer.client_version}"`,
        },
        body: JSON.stringify({
            "Username": embyServer.username,
            "Pw": !embyServer.password ? null : embyServer.password
        }),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 登出
 */
async function logout(embyServer: EmbyServer) {
    if (!embyServer.base_url || !embyServer.auth_token) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + '/emby/Sessions/Logout',
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({}),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 搜索，该处问题：当搜索电影时，如果传入了搜索关键字和限制条数，搜索结果的总数会返回0（搜索返回的结果正常），不传这两个值又没办法分页
 * @returns EmbyPageList<SearchItems>
 */
async function search(embyServer: EmbyServer, search_str: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !search_str || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/Items?SearchTerm=${encodeURIComponent(search_str)}&IncludeItemTypes=Movie,Series&Recursive=true&Fields=AlternateMediaSources,MediaSources,ProductionYear,EndDate&StartIndex=${startIndex}&Limit=${limit}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 首页继续播放列表
 * @returns EmbyPageList<EpisodesItems>
 */
async function getContinuePlayList(embyServer: EmbyServer, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || startIndex < 0 || !limit) {
        console.log(embyServer);
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/Items/Resume?MediaTypes=Video&Recursive=true&Fields=AlternateMediaSources,MediaSources&StartIndex=${startIndex}&Limit=${limit}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 收藏列表
 * @returns EmbyPageList<EpisodesItems>
 */
async function getFavoriteList(embyServer: EmbyServer, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/Items?Filters=IsFavorite&Recursive=true&IncludeItemTypes=Episode,Series,Movie&Fields=AlternateMediaSources,MediaSources,ProductionYear,EndDate&StartIndex=${startIndex}&Limit=${limit}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 系列剧集 接下来 应该播放的剧集
 * @returns EmbyPageList<EpisodesItems>
 */
async function nextUp(embyServer: EmbyServer, seriesId: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Shows/NextUp?UserId=${embyServer.user_id}&SeriesId=${seriesId}&StartIndex=${startIndex}&Limit=${limit}&Fields=AlternateMediaSources,MediaSources`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 首页媒体库列表
 * @returns EmbyPageList<MediaLibraryItem>
 */
async function getMediaLibraryList(embyServer: EmbyServer) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/Views`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 首页媒体库子项目
 * @returns SearchItems[]
 */
async function getMediaLibraryChildLatest(embyServer: EmbyServer, parentId: string, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !parentId || !limit) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/Items/Latest?Limit=${limit}&ParentId=${parentId}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 剧集数量统计
 * @returns MediaLibraryCount
 */
async function count(embyServer: EmbyServer) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Items/Counts?UserId=${embyServer.user_id}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 电影详情、剧集详情、季详情、系列详情、合集详情
 * @returns EpisodesItems
 */
async function items(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/Items/${item_id}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 系列 下的 季列表
 * @returns EmbyPageList<SeasonsItems>
 */
async function seasons(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !item_id || !embyServer.user_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Shows/${item_id}/Seasons?Fields=ProductionYear&UserId=${embyServer.user_id}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 季 下的 剧集列表
 * @returns EmbyPageList<EpisodesItems>
 */
async function episodes(embyServer: EmbyServer, item_id: string, seasonId: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !item_id || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Shows/${item_id}/Episodes?StartIndex=${startIndex}&Limit=${limit}&SeasonId=${seasonId}&Fields=AlternateMediaSources,MediaSources&UserId=${embyServer.user_id}`,
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 播放流媒体详情
 * @returns PlaybackInfo
 */
async function playbackInfo(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Items/${item_id}/PlaybackInfo`,
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({
            "UserId": embyServer.user_id,
            "MaxStreamingBitrate": 1400000000,
            "MaxStaticBitrate": 1400000000,
            "MusicStreamingTranscodingBitrate": 1920000,
            "DeviceProfile": {
                "DirectPlayProfiles": [
                    {
                        "Container": "",
                        "Type": "Video"
                    },
                    {
                        "Container": "",
                        "Type": "Audio"
                    }
                ]
            }
        }),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 开始播放
 * @param positionTicks 播放位置 / 一千万 换算成秒 
 * @returns 204
 */
async function playing(embyServer: EmbyServer, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id || !media_source_id || !play_session_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Sessions/Playing?ItemId=${item_id}&MediaSourceId=${media_source_id}&PlayMethod=DirectStream&PlaySessionId=${play_session_id}&PositionTicks=${positionTicks}`,
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({
            "ItemId": `${item_id}`,
            "MediaSourceId": `${media_source_id}`,
            "PlayMethod": "DirectStream",
            "PlaySessionId": `${play_session_id}`,
            "PositionTicks": `${positionTicks}`,
        }),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 播放进度（保活），只要还在播放就需要定时调用播放进度，长时间没有播放进度，服务器会认为已经停止播放
 * @param positionTicks 播放位置 / 一千万 换算成秒 
 * @returns 204
 */
async function playingProgress(embyServer: EmbyServer, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id || !media_source_id || !play_session_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Sessions/Playing/Progress?ItemId=${item_id}&MediaSourceId=${media_source_id}&PlayMethod=DirectStream&PlaySessionId=${play_session_id}&PositionTicks=${positionTicks}`,
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({
            "ItemId": `${item_id}`,
            "MediaSourceId": `${media_source_id}`,
            "PlayMethod": "DirectStream",
            "PlaySessionId": `${play_session_id}`,
            "PositionTicks": `${positionTicks}`,
        }),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 结束播放
 * @returns 204
 */
async function playingStopped(embyServer: EmbyServer, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id || !media_source_id || !play_session_id || !positionTicks) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Sessions/Playing/Stopped?ItemId=${item_id}&MediaSourceId=${media_source_id}&PlayMethod=DirectStream&PlaySessionId=${play_session_id}&PositionTicks=${positionTicks}`,
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({
            "ItemId": `${item_id}`,
            "MediaSourceId": `${media_source_id}`,
            "PlayMethod": "DirectStream",
            "PlaySessionId": `${play_session_id}`,
            "PositionTicks": `${positionTicks}`,
        }),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
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
function getAudioStreamUrl(embyServer: EmbyServer, mediaSource: MediaSources, mediaStreams: MediaStreams) {
    if (!mediaStreams.IsExternal) {
        return null;
    }
    return embyServer.base_url + `/emby/Audio/${mediaSource.ItemId}/stream.${mediaStreams.Codec}?AudioStreamIndex=${mediaStreams.Index}&Static=true`;
}

/**
 * 组装字幕流地址，请确保字幕流支持外部流
 * @returns
 */
function getSubtitleStreamUrl(embyServer: EmbyServer, mediaSource: MediaSources, mediaStreams: MediaStreams) {
    if (!mediaStreams.IsExternal) {
        return null;
    }
    return embyServer.base_url + `/emby/Videos/${mediaSource.ItemId}/${mediaSource.Id}/Subtitles/${mediaStreams.Index}/Stream.${mediaStreams.Codec}`;
}

/**
 * 组装图片地址
 * @returns
 */
function getImageUrl(embyServer: EmbyServer, item_id: string, imageType: string = 'Primary') {
    if (!item_id) {
        return null;
    }
    return embyServer.base_url + `/emby/Items/${item_id}/Images/${imageType}`;
}

/**
 * 收藏
 * @returns
 */
async function star(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/FavoriteItems/${item_id}`,
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({}),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 取消收藏
 * @returns
 */
async function unstar(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/FavoriteItems/${item_id}`,
        method: 'DELETE',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 标记已播放
 * @returns
 */
async function played(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/PlayedItems/${item_id}`,
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({}),
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

/**
 * 取消已播放
 * @returns
 */
async function unplayed(embyServer: EmbyServer, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: embyServer.base_url + `/emby/Users/${embyServer.user_id}/PlayedItems/${item_id}`,
        method: 'DELETE',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
        proxy: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id)
    });
}

export default {
    getServerInfo, authenticateByName, logout, search, items, seasons, episodes, playbackInfo, playing, playingProgress, playingStopped, getContinuePlayList, nextUp,
    getFavoriteList, getDirectStreamUrl, getAudioStreamUrl, getSubtitleStreamUrl, star, unstar, played, unplayed, getMediaLibraryList, getMediaLibraryChildLatest,
    getImageUrl, count,
}


export interface EmbyPageList<T> {
    TotalRecordCount: number,
    Items: T[]
}

export interface SearchItems {
    Name: string,
    Id: string,
    ProductionYear: number,
    EndDate: string,
    Type: string,
    MediaSources?: MediaSources[],
    UserData?: UserData
}

export interface SeasonsItems {
    Name: string,
    Id: string,
    Overview: string,
    ProductionYear: number,
    IndexNumber: number,
    UserData?: UserData
}

export interface EpisodesItems {
    Name: string,
    Id: string,
    SeriesName: string,
    PremiereDate: string,
    ParentIndexNumber: number,
    IndexNumber: number,
    MediaSources?: MediaSources[],
    SeriesId: string,
    Type: string,
    UserData?: UserData,
    Overview: string,
    ProductionYear: string,
}

export interface PlaybackInfo {
    PlaySessionId: string,
    MediaSources: MediaSources[],
}

export interface MediaSources {
    Id: string,
    ItemId: string, // 电影和剧集的id，一个剧集有多个媒体源，但每个媒体源的itemid不一样，但是用不同的itemid能查询到同一个剧集
    Name: string,
    RunTimeTicks: number,
    Size: number,
    Bitrate: number,
    DirectStreamUrl: string,
    MediaStreams: MediaStreams[]
}

export interface MediaStreams {
    // 视频、音频、字幕编码
    Codec: string,
    DisplayTitle: string,
    DisplayLanguage: string,
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

export interface MediaLibraryItem {
    Name: string,
    Id: string,
    Type: string,
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
