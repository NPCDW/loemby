import { fetch } from '@tauri-apps/plugin-http';
import { EmbyServerConfig } from '../store/config';

/**
 * 获取服务器信息，无需验证
 */
async function getServerInfo(embyServer: EmbyServerConfig) {
    if (!embyServer.base_url) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + '/System/Info/Public', {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
        }
    });
}

/**
 * 通过用户名密码授权
 */
async function authenticateByName(embyServer: EmbyServerConfig) {
    if (!embyServer.base_url || !embyServer.username) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + '/Users/AuthenticateByName', {
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Authorization': `Emby Client="${embyServer.client}", Device="${embyServer.device}", DeviceId="${embyServer.device_id}", Version="${embyServer.client_version}"`,
        },
        body: JSON.stringify({
            "Username": embyServer.username,
            "Pw": !embyServer.password ? null : embyServer.password
        })
    });
}

/**
 * 登出
 */
async function logout(embyServer: EmbyServerConfig) {
    if (!embyServer.base_url || !embyServer.auth_token) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + '/Sessions/Logout', {
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 搜索，该处问题：当搜索电影时，如果传入了搜索关键字和限制条数，搜索结果的总数会返回0（搜索返回的结果正常），不传这两个值又没办法分页
 * @returns EmbyPageList<SearchItems>
 */
async function search(embyServer: EmbyServerConfig, search_str: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !search_str || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Users/${embyServer.user_id}/Items?SearchTerm=${encodeURIComponent(search_str)}&IncludeItemTypes=Movie,Series&Recursive=true&Fields=MediaSources,ProductionYear,EndDate&StartIndex=${startIndex}&Limit=${limit}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 首页继续播放列表
 * @returns EmbyPageList<EpisodesItems>
 */
async function continuePlay(embyServer: EmbyServerConfig, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Users/${embyServer.user_id}/Items/Resume?MediaTypes=Video&Recursive=true&Fields=MediaSources&StartIndex=${startIndex}&Limit=${limit}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 系列剧集 接下来 应该播放的剧集
 * @returns EmbyPageList<EpisodesItems>
 */
async function nextUp(embyServer: EmbyServerConfig, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Users/${embyServer.user_id}/Items/Resume?MediaTypes=Video&Recursive=true&Fields=MediaSources&StartIndex=${startIndex}&Limit=${limit}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 电影详情、剧集详情、季详情、系列详情、合集详情
 * @returns EpisodesItems
 */
async function items(embyServer: EmbyServerConfig, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Users/${embyServer.user_id}/Items/${item_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 系列 下的 季列表
 * @returns EmbyPageList<SeasonsItems>
 */
async function seasons(embyServer: EmbyServerConfig, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !item_id || !embyServer.user_id) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Shows/${item_id}/Seasons?Fields=ProductionYear&UserId=${embyServer.user_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 季 下的 剧集列表
 * @returns EmbyPageList<EpisodesItems>
 */
async function episodes(embyServer: EmbyServerConfig, item_id: string, seasonId: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !item_id || !seasonId || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Shows/${item_id}/Episodes?StartIndex=${startIndex}&Limit=${limit}&SeasonId=${seasonId}&Fields=MediaSources&UserId=${embyServer.user_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

/**
 * 播放流媒体详情
 * @returns PlaybackInfo
 */
async function playbackInfo(embyServer: EmbyServerConfig, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Items/${item_id}/PlaybackInfo?UserId=${embyServer.user_id}`, {
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'Content-Type': 'application/json',
            'X-Emby-Token': embyServer.auth_token,
        },
        body: JSON.stringify({
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
        })
    });
}

/**
 * 开始播放
 * @param positionTicks 播放位置 / 一千万 换算成秒 
 * @returns 204
 */
async function playing(embyServer: EmbyServerConfig, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id || !media_source_id || !play_session_id) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Sessions/Playing?ItemId=${item_id}&CanSeek=true&MediaSourceId=${media_source_id}&PlayMethod=DirectStream&PlaySessionId=${play_session_id}&PositionTicks=${positionTicks}&VolumeLevel=100`, {
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
    });
}

/**
 * 结束播放
 * @param embyServer 
 * @returns 204
 */
async function playingStopped(embyServer: EmbyServerConfig, item_id: string, media_source_id: string, play_session_id: string, positionTicks: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id || !media_source_id || !play_session_id || !positionTicks) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Sessions/Playing/Stopped?ItemId=${item_id}&CanSeek=true&MediaSourceId=${media_source_id}&PlayMethod=DirectStream&PlaySessionId=${play_session_id}&PositionTicks=${positionTicks}&VolumeLevel=100`, {
        method: 'POST',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        },
    });
}

export default {
    getServerInfo, authenticateByName, logout, search, items, seasons, episodes, playbackInfo, playing, playingStopped, continuePlay, nextUp,
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
    UserData?: {
        UnplayedItemCount: number,
        PlaybackPositionTicks: number,
        PlayCount: number,
        IsFavorite: number,
        Played: number,
    }
}

export interface SeasonsItems {
    Name: string,
    Id: string,
    ProductionYear: number,
    IndexNumber: number,
    UserData?: {
        UnplayedItemCount: number,
        PlaybackPositionTicks: number,
        PlayCount: number,
        IsFavorite: number,
        Played: number,
    }
}

export interface EpisodesItems {
    Name: string,
    Id: string,
    PremiereDate: string,
    ParentIndexNumber: number,
    IndexNumber: number,
    MediaSources?: MediaSources[],
    UserData?: {
        PlayedPercentage: number,
        PlaybackPositionTicks: number,
        IsFavorite: number,
        Played: number,
    },
}

export interface PlaybackInfo {
    PlaySessionId: string,
    MediaSources: MediaSources[],
}

export interface MediaSources {
    Id: string,
    Size: number,
    DirectStreamUrl: string,
    MediaStreams: {
        // 视频编码
        Codec: string,
        DisplayTitle: string,
        // 码率
        BitRate: number,
        Type: 'Video' | 'Audio' | 'Subtitle',
        Height: number,
        Width: number,
        Index: number,
    }[]
}
