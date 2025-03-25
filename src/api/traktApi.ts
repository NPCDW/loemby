import { useGlobalConfig } from '../store/db/globalConfig';
import { useProxyServer } from '../store/db/proxyServer';
import invoke from './invoke';

const userAgent = 'loemby/0.6.7'
const client_id = '05521c50a5a5ac1fb238648a15e8da57ea7c708127e49711303c9b9691913572'

async function saveAccessToken(token_response: {access_token: string, refresh_token: string, expires_in: number, created_at: number}) {
    let trakt_info = {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_in: token_response.expires_in + token_response.created_at,
        username: '',
    };
    let config = {
        config_key: "trakt_info",
        config_value: JSON.stringify(trakt_info)
    }
    await useGlobalConfig().updateGlobalConfig(config);
    getUserInfo().then(response => {
        if (response.status_code != 200) {
            console.log('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: {user: {username: string}} = JSON.parse(response.body);
        trakt_info.username = json.user.username;
        let config = {
            config_key: "trakt_info",
            config_value: JSON.stringify(trakt_info)
        }
        useGlobalConfig().updateGlobalConfig(config);
    })
}

async function getCacheAccessToken() {
    let trakt_info = await useGlobalConfig().getGlobalConfigValue('trakt_info');
    if (!trakt_info) {
        return Promise.reject("获取失败");
    }
    let json: {access_token: string, refresh_token: string, expires_in: number} = JSON.parse(trakt_info);
    let currentTime = new Date().getTime() / 1000;
    // 如果离过期时间还有至少6小时，直接返回缓存的token
    if (currentTime < json.expires_in - 6 * 60 * 60) {
        return json.access_token;
    }
    // 如果离过期时间大于1分钟小于6小时，后台刷新token，并立即返回旧token
    else if (currentTime < json.expires_in - 60 && currentTime > json.expires_in - 6 * 60 * 60) {
        console.log("trakt token 不足6小时，后台重新获取");
        token('', json.refresh_token).then(response => {
            if (response.status_code != 200) {
                console.log('response status' + response.status_code + ' ' + response.status_text)
                return
            }
            let rejson: {access_token: string, refresh_token: string, expires_in: number, created_at: number} = JSON.parse(response.body);
            saveAccessToken(rejson);
        })
        return json.access_token;
    }
    // 如果离过期时间不足1分钟，立即刷新token，并返回新token
    else {
        console.log("trakt token 过期，重新获取");
        let response = await token('', json.refresh_token)
        if (response.status_code != 200) {
            console.log('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let rejson: {access_token: string, refresh_token: string, expires_in: number, created_at: number} = JSON.parse(response.body);
        saveAccessToken(rejson);
        return rejson.access_token;
    }
}

/**
 * 获取 token
 */
async function token(code?: string, refresh_token?: string) {
    if ((!code && !refresh_token)) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: 'https://token-exchange.003021.xyz/trakt',
        method: 'POST',
        headers: {
            'User-Agent': userAgent,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            "code": code,
            "refresh_token": refresh_token,
            "redirect_uri": "urn:ietf:wg:oauth:2.0:oob",
        }),
        proxy: await useProxyServer().getTraktProxyUrl()
    });
}

/**
 * 获取用户信息
 */
async function getUserInfo() {
    let access_token = await getCacheAccessToken()
    if (!access_token) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: 'https://api.trakt.tv/users/settings',
        method: 'GET',
        headers: {
            'User-Agent': userAgent,
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${access_token}`,
            'trakt-api-version': '2',
            'trakt-api-key': client_id
        },
        proxy: await useProxyServer().getTraktProxyUrl()
    });
}

/**
 * 开始播放
 */
async function start(type: 'episode' | 'movie', trakt: number, imdb: string, tmdb: number, progress: number) {
    let access_token = await getCacheAccessToken()
    if ((!trakt && !imdb && !tmdb) || !type || progress || access_token) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: 'https://api.trakt.tv/scrobble/start',
        method: 'POST',
        headers: {
            'User-Agent': userAgent,
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${access_token}`,
            'trakt-api-version': '2',
            'trakt-api-key': client_id
        },
        body: JSON.stringify({
            [type]: {
              "ids": {
                "trakt": trakt,
                "imdb": imdb,
                "tmdb": tmdb
              }
            },
            "progress": progress
        }),
        proxy: await useProxyServer().getTraktProxyUrl()
    });
}

/**
 * 停止播放
 */
async function stop(type: 'episode' | 'movie', trakt: number, imdb: string, tmdb: number, progress: number) {
    let access_token = await getCacheAccessToken()
    if ((!trakt && !imdb && !tmdb) || !type || progress || access_token) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: 'https://api.trakt.tv/scrobble/stop',
        method: 'POST',
        headers: {
            'User-Agent': userAgent,
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${access_token}`,
            'trakt-api-version': '2',
            'trakt-api-key': client_id
        },
        body: JSON.stringify({
            [type]: {
              "ids": {
                "trakt": trakt,
                "imdb": imdb,
                "tmdb": tmdb
              }
            },
            "progress": progress
        }),
        proxy: await useProxyServer().getTraktProxyUrl()
    });
}

export default {
    token, getUserInfo, start, stop
}
