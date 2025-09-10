import { ElMessage } from 'element-plus';
import { useGlobalConfig } from '../store/db/globalConfig';
import { invoke } from '@tauri-apps/api/core';

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
        let json: {user: {username: string}} = JSON.parse(response);
        trakt_info.username = json.user.username;
        let config = {
            config_key: "trakt_info",
            config_value: JSON.stringify(trakt_info)
        }
        useGlobalConfig().updateGlobalConfig(config);
    }).catch(e => ElMessage.error(e))
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
        let rejson: TokenResult = JSON.parse(response);
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
async function token({redirect_uri, code, refresh_token}: TokenParam): Promise<string> {
    if ((!code && !refresh_token)) {
        return Promise.reject("参数缺失");
    }
    return invoke('trakt_http_token', {body: {
        redirect_uri, code, refresh_token
    }});
}

/**
 * 获取用户信息
 */
async function getUserInfo(): Promise<string> {
    let access_token = await getCacheAccessToken()
    if (!access_token) {
        return Promise.reject("参数缺失");
    }
    return invoke('trakt_http_get_user_info', {body: {
        access_token
    }});
}

/**
 * 开始播放
 */
async function start(param: any): Promise<string> {
    let access_token = await getCacheAccessToken()
    if (!access_token) {
        return Promise.reject("参数缺失");
    }
    return invoke('trakt_http_start', {body: {
        access_token, body: JSON.stringify(param)
    }});
}

/**
 * 停止播放
 */
async function stop(param: any): Promise<string> {
    let access_token = await getCacheAccessToken()
    if (!access_token) {
        return Promise.reject("参数缺失");
    }
    return invoke('trakt_http_stop', {body: {
        access_token, body: JSON.stringify(param)
    }});
}

export default {
    token, getUserInfo, start, stop
}
