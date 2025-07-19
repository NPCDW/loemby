import { ElMessage, ElMessageBox } from 'element-plus';
import { useGlobalConfig } from '../store/db/globalConfig';
import { useProxyServer } from '../store/db/proxyServer';
import invokeApi, { HttpForwardResult } from './invokeApi';
import { sleep } from '../util/sleep';

const USER_AGENT = 'loemby/' + import.meta.env.VITE_APP_VERSION
const CLIENT_ID = import.meta.env.VITE_TRAKT_CLIENT_ID

async function saveAccessToken(token_response: TokenResult, redirect_uri: string) {
    let trakt_info = {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_in: token_response.expires_in + token_response.created_at,
        redirect_uri,
        username: '',
    };
    let config = {
        config_key: "trakt_info",
        config_value: JSON.stringify(trakt_info)
    }
    await useGlobalConfig().updateGlobalConfig(config);
    getUserInfo().then(response => {
        if (response.status_code != 200) {
            ElMessage.error(response.status_code + ' ' + response.status_text)
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
        return Promise.reject("Trakt 未授权");
    }
    let json: {access_token: string, refresh_token: string, expires_in: number, redirect_uri: string} = JSON.parse(trakt_info);
    let currentTime = new Date().getTime() / 1000;
    // 如果离过期时间还有至少6小时，直接返回缓存的token
    if (currentTime < json.expires_in - 6 * 60 * 60) {
        return json.access_token;
    }
    // 如果离过期时间大于1分钟小于6小时，后台刷新token，并立即返回旧token
    // else if (currentTime < json.expires_in - 60) {
    //     console.log("trakt token 不足6小时，后台重新获取");
    //     token({redirect_uri: json.redirect_uri, refresh_token: json.refresh_token}).then(response => {
    //         if (response.status_code != 200) {
    //             ElMessage.error("trakt token 获取失败: " + response.status_code + ' ' + response.status_text)
    //             return
    //         }
    //         let rejson: TokenResult = JSON.parse(response.body);
    //         saveAccessToken(rejson, json.redirect_uri);
    //     })
    //     return json.access_token;
    // }
    // 如果离过期时间不足1分钟，立即刷新token，并返回新token
    else {
        console.log("trakt token 过期，重新获取");
        let response = await token({redirect_uri: json.redirect_uri, refresh_token: json.refresh_token})
        if (response.status_code != 200) {
            ElMessage.error("trakt token 获取失败: " + response.status_code + ' ' + response.status_text)
            return
        }
        let rejson: TokenResult = JSON.parse(response.body);
        saveAccessToken(rejson, json.redirect_uri);
        return rejson.access_token;
    }
}

interface TokenParam {
    redirect_uri: string,
    code?: string,
    refresh_token?: string
}

interface TokenResult {
    access_token: string,
    refresh_token: string,
    expires_in: number,
    created_at: number
}

/**
 * 获取 token
 */
async function token({redirect_uri, code, refresh_token}: TokenParam) {
    if ((!code && !refresh_token)) {
        return Promise.reject("参数缺失");
    }
    return invokeApi.httpForward({
        url: import.meta.env.VITE_TRAKT_TOKEN_EXCHANGE_URL,
        method: 'POST',
        headers: {
            'User-Agent': USER_AGENT,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            "code": code,
            "refresh_token": refresh_token,
            "redirect_uri": redirect_uri,
        }),
        proxy: await useProxyServer().getTraktProxyUrl()
    }).then(response => {
        if (response.status_code == 401) {
            ElMessageBox.alert("您的 Trakt 授权好像失效了，或许应该重新授权");
        }
        return response
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
    return invokeApi.httpForward({
        url: 'https://api.trakt.tv/users/settings',
        method: 'GET',
        headers: {
            'User-Agent': USER_AGENT,
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${access_token}`,
            'trakt-api-version': '2',
            'trakt-api-key': CLIENT_ID
        },
        proxy: await useProxyServer().getTraktProxyUrl()
    }).then(response => {
        if (response.status_code == 401) {
            ElMessageBox.alert("获取 Trakt 用户信息: Trakt access token 失效");
        }
        return response
    });
}

/**
 * 开始播放
 */
async function start(param: any, retry: number = 0): Promise<HttpForwardResult> {
    let access_token = await getCacheAccessToken()
    if (!access_token) {
        return Promise.reject("参数缺失");
    }
    return invokeApi.httpForward({
        url: 'https://api.trakt.tv/scrobble/start',
        method: 'POST',
        headers: {
            'User-Agent': USER_AGENT,
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${access_token}`,
            'trakt-api-version': '2',
            'trakt-api-key': CLIENT_ID
        },
        body: JSON.stringify(param),
        proxy: await useProxyServer().getTraktProxyUrl()
    }).then(async response => {
        if (response.status_code == 401) {
            ElMessageBox.alert("开始播放: Trakt access token 失效");
        }
        if (response.status_code == 429) {
            if (retry > 0) {
                ElMessage.warning("Trakt 请求太多或太快，开始播放 Api 将在 1 秒钟后重试");
            }
            await sleep(1000)
            return start(param, retry + 1)
        }
        return response
    });
}

/**
 * 停止播放
 */
async function stop(param: any, retry: number = 0): Promise<HttpForwardResult> {
    let access_token = await getCacheAccessToken()
    if (!access_token) {
        return Promise.reject("参数缺失");
    }
    return invokeApi.httpForward({
        url: 'https://api.trakt.tv/scrobble/stop',
        method: 'POST',
        headers: {
            'User-Agent': USER_AGENT,
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${access_token}`,
            'trakt-api-version': '2',
            'trakt-api-key': CLIENT_ID
        },
        body: JSON.stringify(param),
        proxy: await useProxyServer().getTraktProxyUrl()
    }).then(async response => {
        if (response.status_code == 401) {
            ElMessageBox.alert("停止播放: Trakt access token 失效");
        }
        if (response.status_code == 429) {
            if (retry > 0) {
                ElMessage.warning("Trakt 请求太多或太快，停止播放 Api 将在 1 秒钟后重试");
            }
            await sleep(1000)
            return start(param)
        }
        return response
    });
}

export default {
    token, getUserInfo, start, stop
}
