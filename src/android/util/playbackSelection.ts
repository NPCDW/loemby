import type { MediaSource, MediaStream } from '../../api/embyApi'
import { getResolutionLevelFromMediaSources } from '../../util/play_info_util'
import { isInternalUrl } from '../../util/str_util'

export type TrackType = MediaStream['Type']
export interface PlaybackSelection {
    rememberSelect: boolean
    useDirectLink: boolean
    versionIndex: number
    videoSelect: number
    audioSelect: number
    subtitleSelect: number
}

export function preferredVersion(sources: MediaSource[], policy: string) {
    const versions = sources.map((source, index) => ({ index, size: source.Size || 0, resolution: getResolutionLevelFromMediaSources(source) }))
    if (policy === 'high-bitrate') versions.sort((a, b) => b.size - a.size)
    else if (policy === 'high-resolution') versions.sort((a, b) => b.resolution - a.resolution || b.size - a.size)
    return versions[0]?.index || 0
}

export function trackOptions(source: MediaSource | undefined, type: TrackType) {
    const streams = (source?.MediaStreams || []).filter(stream => stream.Type === type)
    return [
        { value: 0, label: '自动' },
        ...streams.map((stream, index) => ({
            value: index + 1,
            label: `${stream.DisplayTitle || stream.Title || `轨道 ${index + 1}`}${stream.DisplayLanguage ? ` / ${stream.DisplayLanguage}` : ''}${stream.IsExternal ? '（外挂）' : ''}`,
        })),
        ...(streams.length ? [{ value: -1, label: '关闭' }] : []),
    ]
}

export function preferredTracks(source: MediaSource | undefined) {
    const video = (source?.MediaStreams || []).filter(stream => stream.Type === 'Video')
    const audio = (source?.MediaStreams || []).filter(stream => stream.Type === 'Audio')
    const subtitles = (source?.MediaStreams || []).filter(stream => stream.Type === 'Subtitle')
    let subtitleIndex = 0
    let subtitleScore = 0
    subtitles.forEach((stream, index) => {
        let score = (stream.IsDefault ? 1 : 0) + (stream.IsExternal ? 2 : 0)
        const language = stream.DisplayLanguage || ''
        if (language.includes('Chinese')) score += 3 + (language.includes('Simplified') ? 1 : 0)
        if (language.includes('中')) score += 3 + (language.includes('简') ? 1 : 0)
        if (score > subtitleScore) { subtitleScore = score; subtitleIndex = index }
    })
    // 与桌面一致：视频取最后一个默认轨，音频取第一个默认轨，无默认则取第一轨。
    return {
        videoSelect: video.length ? Math.max(0, video.reduce((selected, stream, index) => stream.IsDefault ? index : selected, 0)) + 1 : 0,
        audioSelect: audio.length ? Math.max(0, audio.findIndex(stream => stream.IsDefault)) + 1 : 0,
        subtitleSelect: subtitles.length ? subtitleIndex + 1 : 0,
    }
}

export function supportsDirectLink(source: MediaSource | undefined) {
    if (!source?.IsRemote || !source.Path || !source.Path.includes('://')) return false
    return !isInternalUrl(source.Path)
}

export function selectionQuery(selection: PlaybackSelection) {
    return {
        rememberSelect: String(selection.rememberSelect),
        useDirectLink: String(selection.useDirectLink),
        versionSelect: String(selection.versionIndex + 1),
        videoSelect: String(selection.videoSelect),
        audioSelect: String(selection.audioSelect),
        subtitleSelect: String(selection.subtitleSelect),
    }
}

export function validTrack(value: unknown, source: MediaSource | undefined, type: TrackType, fallback: number) {
    if (typeof value !== 'string' || !/^-?\d+$/.test(value)) return fallback
    const number = Number(value)
    return trackOptions(source, type).some(option => option.value === number) ? number : fallback
}
