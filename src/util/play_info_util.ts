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