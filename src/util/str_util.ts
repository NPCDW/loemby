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