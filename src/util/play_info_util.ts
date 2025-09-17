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
    if (!mediaSources || !mediaSources.MediaStreams || mediaSources.MediaStreams.length == 0) {
        return 'Unknown'
    }
    return getResolution(mediaSources.MediaStreams[0].Width, mediaSources.MediaStreams[0].Height)
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
    if (!mediaSources || !mediaSources.MediaStreams || mediaSources.MediaStreams.length == 0) {
        return -1
    }
    const level = getResolutionLevel(mediaSources.MediaStreams[0].Width, mediaSources.MediaStreams[0].Height)
    if (level == -1) {
        const mediaSourcesName = mediaSources.Name ? mediaSources.Name.toLowerCase() : ''
        const mediaStreamsDisplayTitle = mediaSources.MediaStreams[0].DisplayTitle ? mediaSources.MediaStreams[0].DisplayTitle.toLowerCase() : ''
        if (mediaSourcesName.includes('2k') || mediaSourcesName.includes('1440p')
            || mediaStreamsDisplayTitle.includes('2k') || mediaStreamsDisplayTitle.includes('1440p')) {
            return 6
        } else if (mediaSourcesName.includes('4k') || mediaSourcesName.includes('2160p')
            || mediaStreamsDisplayTitle.includes('4k') || mediaStreamsDisplayTitle.includes('2160p')) {
            return 7
        } else if (mediaSourcesName.includes('8k') || mediaSourcesName.includes('4320p')
            || mediaStreamsDisplayTitle.includes('8k') || mediaStreamsDisplayTitle.includes('4320p')) {
            return 8
        } else {
            return 5
        }
    }
    return -1
}

const getResolutionLevel = (width: number, height: number) => {
    if (!width || !height) {
        return -1
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