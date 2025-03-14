import invoke from './invoke';
import { useConfig } from '../store/config';

/**
 * 校验代理服务器
 */
async function getProxyLocation(proxy_id: string) {
    if (!proxy_id) {
        return Promise.reject("参数缺失");
    }
    return invoke.httpForward({
        url: 'https://api.my-ip.io/v2/ip.json',
        method: 'GET',
        headers: {
            'User-Agent': 'loemby/0.5.0',
        },
        proxy: useConfig().getProxyUrl(proxy_id)
    });
}

export default {
    getProxyLocation
}
