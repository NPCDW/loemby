/**
 * 展示层格式化。
 *
 * 所有数字型输出都保持等宽对齐（配合 .mono / .tag），列表不会因位数变化而跳动。
 */

const BYTE_UNITS = ['KB', 'MB', 'GB', 'TB'] as const;
const RATE_UNITS = ['Kbps', 'Mbps', 'Gbps', 'Tbps'] as const;

function scale(size: number, units: readonly string[]): string {
    if (!size || size <= 0) {
        return '0 ' + units[0];
    }
    let value = size;
    for (const unit of units) {
        value /= 1024;
        if (value < 1024) {
            return value.toFixed(2) + ' ' + unit;
        }
    }
    return value.toFixed(2) + ' ' + units[units.length - 1];
}

export function formatBytes(size?: number): string {
    return scale(size ?? 0, BYTE_UNITS);
}

export function formatBitrate(size?: number): string {
    return scale(size ?? 0, RATE_UNITS);
}

/** 1h 12m 30s，用于播放时长 */
export function formatDuration(seconds: number): string {
    const safe = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
    const hours = Math.floor(safe / 3600);
    const minutes = Math.floor((safe % 3600) / 60);
    const secs = safe % 60;
    const parts: string[] = [];
    if (hours > 0) {
        parts.push(hours + 'h');
    }
    if (minutes > 0 || hours > 0) {
        parts.push(minutes + 'm');
    }
    parts.push(secs + 's');
    return parts.join('');
}

/** 01:12:30 / 12:30，用于章节跳转点 */
export function formatClock(seconds: number): string {
    const safe = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
    const hours = Math.floor(safe / 3600);
    const minutes = Math.floor((safe % 3600) / 60);
    const secs = safe % 60;
    const pad = (value: number) => String(value).padStart(2, '0');
    return hours > 0 ? `${hours}:${pad(minutes)}:${pad(secs)}` : `${pad(minutes)}:${pad(secs)}`;
}

/** S01E02 标记 */
export function formatEpisodeNo(season?: number, episode?: number): string {
    const s = season === undefined || season === null ? '–' : String(season).padStart(2, '0');
    const e = episode === undefined || episode === null ? '–' : String(episode).padStart(2, '0');
    return `S${s}E${e}`;
}

/** 2024-05-01 → 2024-05-01；空值给空串，避免出现 null */
export function formatDate(value?: string): string {
    return value ? value.substring(0, 10) : '';
}

/** 剧集年份区间：2019 – 2024 */
export function formatYearRange(year?: number, endDate?: string): string {
    if (!year) {
        return '';
    }
    const end = endDate ? endDate.substring(0, 4) : '';
    return end && end !== String(year) ? `${year} – ${end}` : String(year);
}

/** 进度百分比：已看完 100，未开始 0 */
export function progressOf(userData?: { Played?: boolean; PlayedPercentage?: number }): number {
    if (!userData) {
        return 0;
    }
    if (userData.Played) {
        return 100;
    }
    return Math.trunc(userData.PlayedPercentage || 0);
}

export function percentLabel(userData?: { Played?: boolean; PlayedPercentage?: number }): string {
    if (!userData) {
        return '';
    }
    return userData.Played ? '已看完' : Math.trunc(userData.PlayedPercentage || 0) + '%';
}
