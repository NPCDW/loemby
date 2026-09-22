/**
 * 判断是否是内网地址。
 *
 * 用于决定媒体源的直链能否直接交给播放器：内网地址走直链会绕过代理，
 * 在反代/代理场景下会失败。
 */

const INTERNAL_TLDS = [
    '.local',
    '.localhost',
    '.internal',
    '.intranet',
    '.lan',
    '.home',
    '.corp',
    '.office',
    '.localdomain',
    '.test',
];

function isPrivateIpv4(hostname: string): boolean {
    const match = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/.exec(hostname);
    if (!match) {
        return false;
    }
    const [a, b, c, d] = match.slice(1).map(part => Number(part));
    if ([a, b, c, d].some(part => part > 255)) {
        return false;
    }
    return (
        a === 10 ||
        (a === 172 && b >= 16 && b <= 31) ||
        (a === 192 && b === 168) ||
        a === 127 ||
        (a === 169 && b === 254)
    );
}

export function isInternalUrl(rawUrl: string): boolean {
    try {
        const url = new URL(rawUrl);
        const hostname = url.hostname;
        if (hostname === 'localhost' || hostname === '[::1]') {
            return true;
        }
        if (isPrivateIpv4(hostname)) {
            return true;
        }
        return INTERNAL_TLDS.some(tld => hostname.endsWith(tld));
    } catch {
        return false;
    }
}
