import type { MediaSource } from '../api/embyApi';

/**
 * 分辨率识别。
 *
 * 优先相信视频轨的宽高；宽高缺失时退回媒体源的名称与视频轨标题里的关键词，
 * 这样即使是命名规整但元数据缺失的文件也能给出可用标签。
 */

export const RESOLUTION_LABELS = ['240p', '360p', '480p', '720p', '1080p', '2K', '4K', '8K'] as const;

const NAME_KEYWORDS: Array<[string, number]> = [
    ['8k', 8],
    ['4320p', 8],
    ['4k', 7],
    ['2160p', 7],
    ['2k', 6],
    ['1440p', 6],
    ['1080p', 5],
    ['720p', 4],
    ['480p', 3],
    ['360p', 2],
    ['240p', 1],
];

function levelFromSize(width?: number, height?: number): number {
    if (!width && !height) {
        return 0;
    }
    const w = width || 0;
    const h = height || 0;
    if (w >= 7680 || h >= 4320) return 8;
    if (w >= 3840 || h >= 2160) return 7;
    if (w >= 2560 || h >= 1440) return 6;
    if (w >= 1920 || h >= 1080) return 5;
    if (w >= 1280 || h >= 720) return 4;
    if (w >= 854 || h >= 480) return 3;
    if (w >= 640 || h >= 360) return 2;
    return 1;
}

function levelFromText(text: string): number {
    const lower = text.toLowerCase();
    for (const [keyword, level] of NAME_KEYWORDS) {
        if (lower.includes(keyword)) {
            return level;
        }
    }
    return 0;
}

/** 0 表示未知，1–8 递增。用于排序，因此未知必须返回 0 */
export function resolutionLevel(mediaSource?: MediaSource): number {
    if (!mediaSource) {
        return 0;
    }
    const videoStream = mediaSource.MediaStreams?.find(stream => stream.Type === 'Video');
    const bySize = levelFromSize(videoStream?.Width, videoStream?.Height);
    if (bySize > 0) {
        return bySize;
    }
    const byTitle = levelFromText(videoStream?.DisplayTitle || '');
    if (byTitle > 0) {
        return byTitle;
    }
    return levelFromText(mediaSource.Name || '');
}

/** 展示用标签 */
export function resolutionLabel(mediaSource?: MediaSource): string {
    const level = resolutionLevel(mediaSource);
    return level > 0 ? RESOLUTION_LABELS[level - 1] : 'Unknown';
}

/** 取体积最大的媒体源，作为默认版本 */
export function largestMediaSource(mediaSources?: MediaSource[]): MediaSource | null {
    if (!mediaSources || mediaSources.length === 0) {
        return null;
    }
    return mediaSources.reduce((best, current) => (current.Size > best.Size ? current : best));
}
