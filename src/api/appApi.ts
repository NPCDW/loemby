import { useProxyServer } from '../store/db/proxyServer';
import invokeApi from './invokeApi';

const USER_AGENT = 'loemby/' + import.meta.env.VITE_APP_VERSION

/**
 * 校验代理服务器
 */
async function getProxyLocation(proxy_id: string) {
    if (!proxy_id) {
        return Promise.reject("参数缺失");
    }
    return invokeApi.httpForward({
        url: 'https://api.my-ip.io/v2/ip.json',
        method: 'GET',
        headers: {
            'User-Agent': USER_AGENT,
        },
        proxy: await useProxyServer().getProxyUrl(proxy_id)
    });
}

export default {
    getProxyLocation
}
