import { MediaSource } from "../api/embyApi"

export const maxMediaSources = (mediaSources?: MediaSource[]) => {
    if (!mediaSources || mediaSources.length == 0) {
        return null
    }
    let maxMediaSource: MediaSource = mediaSources[0]
    let max = 0
    for (let mediaSource of mediaSources) {
        if (mediaSource.Size > max) {
            max = mediaSource.Size
            maxMediaSource = mediaSource
        }
    }
    return maxMediaSource
}

export const getResolutionFromMediaSources = (mediaSources?: MediaSource) => {
    if (!mediaSources) {
        return 'Unknown'
    }
    let mediaStreamsDisplayTitle = ''
    if (mediaSources.MediaStreams && mediaSources.MediaStreams.length > 0) {
        const videoMediaStream = mediaSources.MediaStreams.find(item => item.Type == 'Video')
        if (videoMediaStream) {
            const resolution = getResolution(videoMediaStream.Width, videoMediaStream.Height)
            if (resolution != 'Unknown') {
                return resolution
            }
            mediaStreamsDisplayTitle = videoMediaStream.DisplayTitle ? videoMediaStream.DisplayTitle.toLowerCase() : ''
        }
    }
    const mediaSourcesName = mediaStreamsDisplayTitle + ' ' + mediaSources.Name ? mediaSources.Name.toLowerCase() : ''
    if (mediaSourcesName.includes('8k') || mediaSourcesName.includes('4320p')) {
        return '8k'
    } else if (mediaSourcesName.includes('4k') || mediaSourcesName.includes('2160p')) {
        return '4k'
    } else if (mediaSourcesName.includes('2k') || mediaSourcesName.includes('1440p')) {
        return '2k'
    } else if (mediaSourcesName.includes('1080p')) {
        return '1080p'
    } else if (mediaSourcesName.includes('720p')) {
        return '720p'
    } else if (mediaSourcesName.includes('480p')) {
        return '480p'
    } else if (mediaSourcesName.includes('360p')) {
        return '360p'
    } else if (mediaSourcesName.includes('240p')) {
        return '240p'
    }
    return 'Unknown'
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
export const getResolution = (width: number, height: number) => {
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

export const getResolutionLevelFromMediaSources = (mediaSources?: MediaSource) => {
    // 需要返回0，否则排序时相减会导致错误
    if (!mediaSources) {
        return 0
    }
    let mediaStreamsDisplayTitle = ''
    if (mediaSources.MediaStreams && mediaSources.MediaStreams.length > 0) {
        const videoMediaStream = mediaSources.MediaStreams.find(item => item.Type == 'Video')
        if (videoMediaStream) {
            const level = getResolutionLevel(videoMediaStream.Width, videoMediaStream.Height)
            if (level != 0) {
                return level
            }
            mediaStreamsDisplayTitle = videoMediaStream.DisplayTitle ? videoMediaStream.DisplayTitle.toLowerCase() : ''
        }
    }
    const mediaSourcesName = mediaStreamsDisplayTitle + ' ' + mediaSources.Name ? mediaSources.Name.toLowerCase() : ''
    if (mediaSourcesName.includes('8k') || mediaSourcesName.includes('4320p')) {
        return 8
    } else if (mediaSourcesName.includes('4k') || mediaSourcesName.includes('2160p')) {
        return 7
    } else if (mediaSourcesName.includes('2k') || mediaSourcesName.includes('1440p')) {
        return 6
    } else if (mediaSourcesName.includes('1080p')) {
        return 5
    } else if (mediaSourcesName.includes('720p')) {
        return 4
    } else if (mediaSourcesName.includes('480p')) {
        return 3
    } else if (mediaSourcesName.includes('360p')) {
        return 2
    } else if (mediaSourcesName.includes('240p')) {
        return 1
    }
    return 0
}

const getResolutionLevel = (width: number, height: number) => {
    if (!width || !height) {
        return 0
    } else if (width >= 7680 || height >= 4320) {
        return 8
    } else if (width >= 3840 || height >= 2160) {
        return 7
    } else if (width >= 2560 || height >= 1440) {
        return 6
    } else if (width >= 1920 || height >= 1080) {
        return 5
    } else if (width >= 1280 || height >= 720) {
        return 4
    } else if (width >= 854 || height >= 480) {
        return 3
    } else if (width >= 640 || height >= 360) {
        return 2
    } else {
        return 1
    }
}