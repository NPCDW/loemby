import { invoke } from '@tauri-apps/api/core';

/**
 * 校验代理服务器
 */
async function getProxyLocation(proxy_id: string): Promise<string> {
    return invoke('app_http_get_proxy_location', {body: {
        proxy_id
    }});
}

/**
 * 获取emby图标库
 */
async function getEmbyIconLibrary(url: string): Promise<string> {
    return invoke('app_http_get_emby_icon_library', {body: {
        url
    }});
}

export default {
    getProxyLocation, getEmbyIconLibrary
}
