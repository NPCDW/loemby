import { MediaSources } from "../api/embyApi"

export const maxMediaSources = (mediaSources: MediaSources[]) => {
    if (!mediaSources || mediaSources.length == 0) {
        return null
    }
    let maxMediaSource: MediaSources = mediaSources[0]
    let max = 0
    for (let mediaSource of mediaSources) {
        if (mediaSource.Size > max) {
            max = mediaSource.Size
            maxMediaSource = mediaSource
        }
    }
    return maxMediaSource
}

/**
 * 参考 youtube 视频分辨率和宽高比 https://support.google.com/youtube/answer/6375112?hl=zh-Hans&co=GENIE.Platform%3DDesktop#
 * 4320p (8k)：7680x4320
 * 2160p (4K)：3840x2160
 * 1440p (2k)：2560x1440
 * 1080p（高清）：1920x1080
 * 720p（高清）：1280x720
 * 480p（标清）：854x480
 * 360p（标清）：640x360
 * 240p（标清）：426x240
 */
export const guessResolution = (width: number, height: number) => {
    if (!width || !height) {
        return 'Unknown'
    } else if (width >= 7680 || height >= 4320) {
        return '8K'
    } else if (width >= 3840 || height >= 2160) {
        return '4K'
    } else if (width >= 2560 || height >= 1440) {
        return '2K'
    } else if (width >= 1920 || height >= 1080) {
        return '1080p'
    } else if (width >= 1280 || height >= 720) {
        return '720p'
    } else if (width >= 854 || height >= 480) {
        return '480p'
    } else if (width >= 640 || height >= 360) {
        return '360p'
    } else {
        return '240p'
    }
}