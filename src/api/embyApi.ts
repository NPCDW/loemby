import { fetch } from '@tauri-apps/plugin-http';
import { generateGuid } from '../util/uuid'
import { getOsInfo } from '../util/os'

const client = "loemby";
const version = "0.1.0";
const user_agent = client + "/" + version;

async function getServerInfo(base_url: string) {
    return fetch(base_url + '/System/Info/Public', {
        method: 'GET',
        headers: {
            'User-Agent': user_agent,
        }
    });
}

async function authenticateByName(base_url: string, username: string, password: string) {
    return fetch(base_url + '/System/Info/Public', {
        method: 'POST',
        headers: {
            'User-Agent': user_agent,
            'Content-Type': 'application/json',
            'X-Emby-Authorization': `Emby Client="${client}", Device="${getOsInfo().name}", DeviceId="${generateGuid()}", Version="${version}"`,
        },
        body: JSON.stringify({
            "Username": username,
            "Pw": password
        })
    });
}

async function logout(base_url: string, token: string) {
    return fetch(base_url + '/System/Info/Public', {
        method: 'POST',
        headers: {
            'User-Agent': user_agent,
            'X-Emby-Token': token,
        }
    });
}

async function search(base_url: string, token: string, user_id: string, search_str: string) {
    return fetch(base_url + `/Users/${user_id}/Items?SearchTerm=${encodeURIComponent(search_str)}&IncludeItemTypes=Movie,Series&Recursive=true`, {
        method: 'GET',
        headers: {
            'User-Agent': user_agent,
            'X-Emby-Token': token,
        }
    });
}

async function items(base_url: string, token: string, user_id: string, item_id: string) {
    return fetch(base_url + `/Users/${user_id}/Items/${item_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': user_agent,
            'X-Emby-Token': token,
        }
    });
}

async function seasons(base_url: string, token: string, item_id: string) {
    return fetch(base_url + `/Shows/${item_id}/Seasons`, {
        method: 'GET',
        headers: {
            'User-Agent': user_agent,
            'X-Emby-Token': token,
        }
    });
}

async function episodes(base_url: string, token: string, item_id: string) {
    return fetch(base_url + `/Shows/${item_id}/Episodes`, {
        method: 'GET',
        headers: {
            'User-Agent': user_agent,
            'X-Emby-Token': token,
        }
    });
}

async function playbackInfo(base_url: string, token: string, user_id: string, item_id: string) {
    return fetch(base_url + `/Items/${item_id}/PlaybackInfo?UserId=${user_id}`, {
        method: 'GET',
        headers: {
            'User-Agent': user_agent,
            'X-Emby-Token': token,
        }
    });
}

export default {
    getServerInfo, authenticateByName, logout, search, items, seasons, episodes, playbackInfo
}