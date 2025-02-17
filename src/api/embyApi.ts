import { fetch } from '@tauri-apps/plugin-http';
import { EmbyServerConfig } from '../store/config';

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

async function search(embyServer: EmbyServerConfig, search_str: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !search_str || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Users/${embyServer.user_id}/Items?SearchTerm=${encodeURIComponent(search_str)}&IncludeItemTypes=Movie,Series&Recursive=true&Fields=MediaStreams,ProductionYear,EndDate&StartIndex=${startIndex}&Limit=${limit}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

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

async function episodes(embyServer: EmbyServerConfig, item_id: string, seasonId: string, startIndex: number, limit: number) {
    if (!embyServer.base_url || !embyServer.auth_token || !item_id || !seasonId || startIndex < 0 || !limit) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Shows/${item_id}/Episodes?StartIndex=${startIndex}&Limit=${limit}&SeasonId=${seasonId}&Fields=MediaStreams&UserId=${embyServer.user_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

async function playbackInfo(embyServer: EmbyServerConfig, item_id: string) {
    if (!embyServer.base_url || !embyServer.auth_token || !embyServer.user_id || !item_id) {
        return Promise.reject("参数缺失");
    }
    return fetch(embyServer.base_url + `/Items/${item_id}/PlaybackInfo?UserId=${embyServer.user_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': embyServer.user_agent!,
            'X-Emby-Token': embyServer.auth_token,
        }
    });
}

export default {
    getServerInfo, authenticateByName, logout, search, items, seasons, episodes, playbackInfo
}