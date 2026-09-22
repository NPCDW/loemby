import { computed, ref, type Ref } from 'vue';
import type { EpisodeItem, MediaSource, MediaStream } from '../api/embyApi';
import { formatBitrate, formatBytes, formatDuration } from '../util/format';
import { resolutionLevel, resolutionLabel } from '../util/resolution';

export interface VersionOption {
    /** 1 起；-1 表示没有可用版本 */
    value: number;
    mediaSourceId: string;
    name: string;
    size: string;
    bitrate: string;
    resolution: string;
}

export interface TrackOption {
    label: string;
    value: number;
}

export interface RestoredSelection {
    version: number;
    video: number;
    audio: number;
    subtitle: number;
}

/** 中文优先、外置优先、默认优先，用于自动挑字幕 */
function scoreSubtitle(stream: MediaStream): number {
    let score = 0;
    if (stream.IsDefault) {
        score += 1;
    }
    if (stream.IsExternal) {
        score += 2;
    }
    const language = stream.DisplayLanguage || '';
    if (language.includes('Chinese') || language.includes('中')) {
        score += 3;
        if (language.includes('Simplified') || language.includes('简')) {
            score += 1;
        }
    }
    return score;
}

/**
 * 播放参数编排。
 *
 * 把一个媒体源拆成「版本 / 视频 / 音频 / 字幕」四组可选项：
 * 版本按全局策略自动挑，字幕按语言偏好自动挑，其余按 IsDefault。
 *
 * 这套编排原先与页面模板混在 500 行的单集页里，抽出来之后
 * 播放详情页只负责展示与派发，逻辑可以单独推敲与测试。
 */
export function usePlaybackTracks(options: {
    policy: Ref<string>;
    /** 记住手动选择时，用 URL 参数覆盖自动选择 */
    restoredSelection?: Ref<RestoredSelection | null>;
    /** 统一记录日志，便于排查「为什么选了这个版本」 */
    onLog?: (message: string) => void;
}) {
    const log = options.onLog ?? (() => undefined);

    const mediaSources = ref<MediaSource[]>([]);
    const current = ref<MediaSource | null>(null);

    const versions = ref<VersionOption[]>([]);
    const videoOptions = ref<TrackOption[]>([]);
    const audioOptions = ref<TrackOption[]>([]);
    const subtitleOptions = ref<TrackOption[]>([]);

    const version = ref(1);
    const video = ref(-1);
    const audio = ref(-1);
    const subtitle = ref(-1);

    const durationText = computed(() =>
        current.value ? formatDuration(current.value.RunTimeTicks / 10_000_000) : '',
    );
    const sizeText = computed(() => (current.value ? formatBytes(current.value.Size) : ''));
    const bitrateText = computed(() => (current.value ? formatBitrate(current.value.Bitrate) : ''));
    const resolutionText = computed(() => resolutionLabel(current.value ?? undefined));

    /** 直链只在「远端媒体源 + 有可解析的绝对地址」时才有意义 */
    const supportsDirectLink = computed(() => {
        const source = current.value;
        return !!source?.IsRemote && !!source.Path && source.Path.includes('://');
    });

    function toOption(mediaSource: MediaSource, index: number): VersionOption {
        return {
            value: index + 1,
            mediaSourceId: mediaSource.Id,
            name: mediaSource.Name,
            size: formatBytes(mediaSource.Size),
            bitrate: formatBitrate(mediaSource.Bitrate),
            resolution: resolutionLabel(mediaSource),
        };
    }

    /** 按全局策略挑默认版本；策略未知时退回「高分辨率优先，同分辨率取大体积」 */
    function pickDefaultVersionId(): number {
        const scored = mediaSources.value.map((source, index) => ({
            value: index + 1,
            size: source.Size,
            level: resolutionLevel(source),
        }));
        if (options.policy.value === 'high-bitrate') {
            scored.sort((a, b) => b.size - a.size);
        } else {
            scored.sort((a, b) => (a.level !== b.level ? b.level - a.level : b.size - a.size));
        }
        return scored[0]?.value ?? 1;
    }

    function resetTracks(): void {
        videoOptions.value = [];
        audioOptions.value = [];
        subtitleOptions.value = [];
        video.value = -1;
        audio.value = -1;
        subtitle.value = -1;
    }

    /** 切换版本：重算四组选项，并把默认值填回去 */
    function applyVersion(versionId: number, firstTime = false): void {
        const option = versions.value.find(item => item.value === versionId);
        current.value = option ? (mediaSources.value.find(item => item.Id === option.mediaSourceId) ?? null) : null;
        version.value = versionId;
        resetTracks();

        const source = current.value;
        if (!source) {
            return;
        }

        let videoIndex = 0;
        let audioIndex = 0;
        let subtitleIndex = 0;
        let bestSubtitleScore = 0;

        for (const stream of source.MediaStreams ?? []) {
            const external = stream.IsExternal ? '（外置）' : '';
            if (stream.Type === 'Video') {
                videoIndex += 1;
                videoOptions.value.push({ label: stream.DisplayTitle + external, value: videoIndex });
                if (stream.IsDefault) {
                    video.value = videoIndex;
                }
            } else if (stream.Type === 'Audio') {
                audioIndex += 1;
                audioOptions.value.push({ label: stream.DisplayTitle + external, value: audioIndex });
                if (stream.IsDefault && audio.value === -1) {
                    audio.value = audioIndex;
                }
            } else if (stream.Type === 'Subtitle') {
                subtitleIndex += 1;
                const language = stream.DisplayLanguage ? ` / ${stream.DisplayLanguage}` : '';
                subtitleOptions.value.push({
                    label: stream.DisplayTitle + language + external,
                    value: subtitleIndex,
                });
                const score = scoreSubtitle(stream);
                if (score > bestSubtitleScore) {
                    bestSubtitleScore = score;
                    subtitle.value = subtitleIndex;
                }
            }
        }

        // 有轨道时末尾补「关闭」；没有轨道时给「自动」(0 = 交给播放器)
        videoOptions.value.push(videoOptions.value.length ? { label: '关闭', value: -1 } : { label: '自动', value: 0 });
        audioOptions.value.push(audioOptions.value.length ? { label: '关闭', value: -1 } : { label: '自动', value: 0 });
        subtitleOptions.value.push(subtitleOptions.value.length ? { label: '关闭', value: -1 } : { label: '自动', value: 0 });

        if (video.value === -1 && videoOptions.value.length > 1) {
            video.value = videoOptions.value[0].value;
        }
        if (audio.value === -1 && audioOptions.value.length > 1) {
            audio.value = audioOptions.value[0].value;
        }
        if (subtitle.value === -1 && subtitleOptions.value.length > 1) {
            subtitle.value = subtitleOptions.value[0].value;
        }

        // 记住的手动选择优先级最高（只在首次载入时套用）
        const restored = firstTime ? options.restoredSelection?.value : null;
        if (restored) {
            video.value = restored.video;
            audio.value = restored.audio;
            subtitle.value = restored.subtitle;
        }

        log(`版本 ${version.value}：${videoOptions.value.length - 1} 视频轨 / ${audioOptions.value.length - 1} 音频轨 / ${subtitleOptions.value.length - 1} 字幕轨`);
    }

    /** 载入条目：建立版本表并选中一个版本 */
    function applyItem(item: EpisodeItem, firstTime = false): void {
        mediaSources.value = item.MediaSources ?? [];
        if (mediaSources.value.length === 0) {
            versions.value = [];
            current.value = null;
            resetTracks();
            return;
        }
        versions.value = mediaSources.value.map(toOption);
        const restored = firstTime ? options.restoredSelection?.value : null;
        const target = restored?.version && restored.version > 0 ? restored.version : pickDefaultVersionId();
        applyVersion(target, firstTime);
    }

    return {
        versions,
        videoOptions,
        audioOptions,
        subtitleOptions,
        version,
        video,
        audio,
        subtitle,
        current,
        durationText,
        sizeText,
        bitrateText,
        resolutionText,
        supportsDirectLink,
        applyItem,
        applyVersion,
    };
}
