<template>
    <button class="poster clickable" type="button" :title="item.Name" @click="emit('open', item)">
        <span class="poster__cover cover cover--poster">
            <img v-if="src" v-lazy="src" :alt="item.Name" />
            <span v-else class="poster__blank" />

            <!-- 进度：贴底灯带，只有真的在播才出现 -->
            <span v-if="percent > 0 && percent < 100" class="bar">
                <span class="bar__fill" :style="{ width: percent + '%' }" />
            </span>

            <!-- 未看数 / 集号：右上角一枚小签 -->
            <span v-if="corner" class="corner" :class="{ 'corner--calm': cornerCalm }">{{ corner }}</span>

            <!-- 已看完：整块变暗，不额外加勾选徽标 -->
            <span v-if="isPlayed" class="poster__watched" />
        </span>

        <span class="poster__body">
            <span class="poster__name t-clamp-2">{{ item.Name }}</span>
            <span v-if="seriesName" class="poster__series t-clip">{{ seriesName }}</span>
            <span class="poster__facts">
                <span v-if="facts.length" class="poster__facts-text t-clip">{{ facts.join(' · ') }}</span>
            </span>
        </span>
    </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { EpisodeItem, SearchItem, SeriesItem } from '../../api/embyApi';
import { formatBytes, formatDate, formatEpisodeNo, formatYearRange, progressOf } from '../../util/format';
import { resolutionLabel } from '../../util/resolution';
import { useImage } from '../../store/image';

/**
 * 海报卡：网格里的基本单元。
 *
 * 结构固定为「封面 + 三行信息」，由父级网格决定列宽；
 * 进度只用一条琥珀灯带，未看数只用一枚小签，避免在封面上堆徽标。
 */
const props = withDefaults(
    defineProps<{
        item: SearchItem;
        embyServerId: string;
        /** 单集时在标题下补一行所属剧名 */
        showSeriesName?: boolean;
    }>(),
    { showSeriesName: false },
);

const emit = defineEmits<{ open: [item: SearchItem] }>();

const image = useImage();

const src = computed(() => image.images[`${props.embyServerId}:cover:${props.item.Id}`] ?? '');
const percent = computed(() => progressOf(props.item.UserData));
const isPlayed = computed(() => !!props.item.UserData?.Played && (props.item as EpisodeItem).Type !== 'Series');

const episode = computed(() => props.item as EpisodeItem);
const series = computed(() => props.item as SeriesItem);

/** 单集给 S01E02，剧集给未看数，其余不给角标 */
const corner = computed(() => {
    if (props.item.Type === 'Episode') {
        return formatEpisodeNo(episode.value.ParentIndexNumber, episode.value.IndexNumber);
    }
    const pending = props.item.UserData?.UnplayedItemCount;
    return pending ? String(pending) : '';
});

const cornerCalm = computed(() => props.item.Type === 'Episode');

const seriesName = computed(() => {
    if (!props.showSeriesName) {
        return '';
    }
    const name = episode.value.SeriesName;
    return name && name !== props.item.Name ? name : '';
});

/** 事实行只放等宽数字：年份 / 日期 / 分辨率 / 体积，最多三项 */
const facts = computed(() => {
    const out: string[] = [];
    if (props.item.Type === 'Series') {
        out.push(formatYearRange(props.item.ProductionYear, series.value.EndDate));
    } else if (props.item.Type === 'Episode') {
        out.push(formatDate(episode.value.PremiereDate) || formatYearRange(props.item.ProductionYear));
    } else {
        out.push(formatYearRange(props.item.ProductionYear));
    }
    const mediaSource = episode.value.MediaSources?.[0];
    if (mediaSource) {
        out.push(resolutionLabel(mediaSource));
        out.push(formatBytes(mediaSource.Size));
    }
    return out.filter(Boolean);
});
</script>

<style scoped>
.poster {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
    padding: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
}

.poster__cover {
    display: block;
    width: 100%;
}

.poster__cover img {
    width: 100%;
    height: 100%;
}

.poster__blank {
    display: block;
    width: 100%;
    height: 100%;
    background: repeating-linear-gradient(
        -45deg,
        var(--surface-card),
        var(--surface-card) 8px,
        var(--surface-hover) 8px,
        var(--surface-hover) 16px
    );
}

.poster__watched {
    position: absolute;
    inset: 0;
    background: rgba(8, 10, 14, 0.46);
}

.poster__body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
}

.poster__name {
    color: var(--text);
    font-size: var(--fs-md);
    font-weight: 500;
    line-height: 1.4;
}

.poster:hover .poster__name {
    color: var(--lamp-bright);
}

.poster__series {
    color: var(--text-faint);
    font-size: var(--fs-xs);
}

.poster__facts {
    min-height: 1.1rem;
}

.poster__facts-text {
    color: var(--text-faint);
    font-family: var(--font-num);
    font-size: var(--fs-xs);
    font-variant-numeric: tabular-nums;
}
</style>
