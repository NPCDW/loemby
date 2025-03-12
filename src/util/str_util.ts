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