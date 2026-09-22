import { invoke } from '@tauri-apps/api/core';

/**
 * 应用级 HTTP 辅助接口。
 *
 * 这些请求由 Rust 侧发起，因此天然走「应用数据代理」配置，
 * 前端不需要也不应该自己发网络请求。
 */

export interface GeoLocation {
    ip: string;
    country_code: string;
}

export interface EmbyIconLibraryPayload {
    name: string;
    icons: Array<{ name: string; url: string }>;
}

function parse<T>(raw: unknown): T {
    return typeof raw === 'string' ? (JSON.parse(raw) as T) : (raw as T);
}

/** 校验代理服务器：拿它出口的 IP 与地区 */
async function getProxyLocation(proxyId: string): Promise<GeoLocation> {
    return parse<GeoLocation>(await invoke('app_http_get_proxy_location', { body: { proxy_id: proxyId } }));
}

/** 校验反代服务器 */
async function getReverseProxyLocation(reverseProxyId: string): Promise<GeoLocation> {
    return parse<GeoLocation>(await invoke('app_http_get_reverse_proxy_location', {
        body: { reverse_proxy_id: reverseProxyId },
    }));
}

/** 拉取图标库清单 */
async function getEmbyIconLibrary(url: string): Promise<EmbyIconLibraryPayload> {
    return parse<EmbyIconLibraryPayload>(await invoke('app_http_get_emby_icon_library', { body: { url } }));
}

export default { getProxyLocation, getReverseProxyLocation, getEmbyIconLibrary };
