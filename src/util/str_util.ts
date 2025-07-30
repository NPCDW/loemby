export const formatBytes = (size: number) => {
    if (!size) {
        return "0 KB"
    }
    const units: string[] = ['KB', 'MB', 'GB', 'TB'];
    for (let index = 0; index < units.length; index++) {
        size /= 1024;
        if (size < 1024) {
            return size.toFixed(2) + " " + units[index];
        }
    }
    return size.toFixed(2) + units[units.length - 1]
}

export const formatMbps = (size: number) => {
    if (!size) {
        return "0 Kbps"
    }
    const units: string[] = ['Kbps', 'Mbps', 'Gbps', 'Tbps'];
    for (let index = 0; index < units.length; index++) {
        size /= 1024;
        if (size < 1024) {
            return size.toFixed(2) + " " + units[index];
        }
    }
    return size.toFixed(2) + units[units.length - 1]
}

export function secondsToHMS(seconds: number) {
    // 计算小时、分钟和秒
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    // 根据条件构建时间字符串
    let timeParts = [];
    if (hours > 0) {
        timeParts.push(hours.toString() + 'h'); // 添加小时
    }
    if (minutes > 0 || hours > 0) { // 如果有小时或分钟不为0，则显示分钟
        timeParts.push(minutes.toString() + 'm'); // 添加分钟
    }
    timeParts.push(secs.toString() + 's'); // 添加秒

    // 用冒号连接时间部分
    return timeParts.join('');
}

export function isInternalUrl(urlString: string) {
    try {
        // 解析URL
        const url = new URL(urlString);
        
        // 获取主机名（可能是域名或IP）
        const hostname = url.hostname;
        
        // 如果是localhost，直接返回true
        if (hostname === 'localhost') {
            return true;
        }
        
        // 检查是否是IPv6的环回地址
        if (hostname === '[::1]') {
            return true;
        }
        
        // 检查是否是IPv4地址
        if (isPrivateIP(hostname)) {
            return true;
        }
        
        // 检查域名后缀是否是内部域名
        if (isLikelyInternalDomain(hostname)) {
            return true;
        }
        
        return false;
    } catch (e) {
        // URL解析失败，可能是无效的URL
        console.error('Invalid URL:', e);
        return false;
    }
}

function isPrivateIP(ip: string) {
    // 检查是否为IPv4地址
    const ipv4Regex = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;
    const match = ip.match(ipv4Regex);
    
    if (!match) {
        // 如果不是IPv4地址，可能是域名或IPv6
        return false;
    }
    
    // 将IP地址的四个部分转换为数字
    const part1 = parseInt(match[1], 10);
    const part2 = parseInt(match[2], 10);
    const part3 = parseInt(match[3], 10);
    const part4 = parseInt(match[4], 10);
    
    // 检查各部分是否在有效范围内
    if (part1 > 255 || part2 > 255 || part3 > 255 || part4 > 255) {
        return false;
    }
    
    // 检查私有IP范围
    return (
        // 10.0.0.0 - 10.255.255.255
        part1 === 10 ||
        // 172.16.0.0 - 172.31.255.255
        (part1 === 172 && part2 >= 16 && part2 <= 31) ||
        // 192.168.0.0 - 192.168.255.255
        (part1 === 192 && part2 === 168) ||
        // 127.0.0.0 - 127.255.255.255 (环回地址)
        part1 === 127 ||
        // 169.254.0.0 - 169.254.255.255 (链路本地地址)
        (part1 === 169 && part2 === 254)
    );
}

function isLikelyInternalDomain(hostname: string) {
    const internalTLDs = [
        '.local',
        '.localhost',
        '.internal',
        '.intranet',
        '.lan',
        '.home',
        '.corp',
        '.office',
        '.localdomain',
        '.test'
    ];
    
    return internalTLDs.some(tld => hostname.endsWith(tld));
}