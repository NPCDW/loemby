<template>
    <div class="eprow" :class="{ 'is-active': active }" @click="emit('open', item)">
        <span class="eprow__num">{{ indexText }}</span>

        <span class="eprow__main">
            <span class="eprow__name t-clip">{{ item.Name }}</span>
            <span class="eprow__sub">
                <span v-if="dateText" class="num">{{ dateText }}</span>
                <span v-for="tag in tags" :key="tag" class="tag">{{ tag }}</span>
            </span>
        </span>

        <span v-if="item.UserData?.UnplayedItemCount" class="corner corner--calm eprow__pending">
            {{ item.UserData.UnplayedItemCount }}
        </span>

        <span v-if="actions" class="eprow__ops" @click.stop>
            <button
                class="ghost ghost--bare"
                type="button"
                :class="{ 'is-on': item.UserData?.IsFavorite }"
                :disabled="starPending"
                :title="item.UserData?.IsFavorite ? '取消收藏' : '收藏'"
                @click="emit('star')"
            >
                <el-icon :size="15" :class="{ 'is-loading': starPending }">
                    <i-ep-StarFilled v-if="item.UserData?.IsFavorite" />
                    <i-ep-Star v-else />
                </el-icon>
            </button>
            <button
                class="ghost ghost--bare"
                type="button"
                :class="{ 'is-done': item.UserData?.Played }"
                :disabled="playedPending"
                :title="item.UserData?.Played ? '标记为未播放' : '标记为已播放'"
                @click="emit('played')"
            >
                <el-icon :size="15" :class="{ 'is-loading': playedPending }">
                    <i-ep-CircleCheckFilled v-if="item.UserData?.Played" />
                    <i-ep-CircleCheck v-else />
                </el-icon>
            </button>
        </span>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { EpisodeItem } from '../../api/embyApi';
import { formatBytes, formatDate } from '../../util/format';
import { resolutionLabel } from '../../util/resolution';

/**
 * 单集行：左侧集号 / 中间名称与参数 / 右侧标记。
 *
 * 行与行之间只有一条发丝线，不使用卡片，一屏能看完一季。
 */
const props = withDefaults(
    defineProps<{
        item: EpisodeItem;
        /** 覆盖显示用的集号；不传则用 IndexNumber */
        index?: number;
        actions?: boolean;
        starPending?: boolean;
        playedPending?: boolean;
        active?: boolean;
    }>(),
    { index: undefined, actions: true, starPending: false, playedPending: false, active: false },
);

const emit = defineEmits<{
    open: [item: EpisodeItem];
    star: [];
    played: [];
}>();

const indexText = computed(() =>
    props.index === undefined ? String(props.item.IndexNumber ?? '–') : String(props.index),
);

const dateText = computed(() => formatDate(props.item.PremiereDate));

/** 媒体源参数：体积 + 分辨率，一集可能有多个版本 */
const tags = computed(() => {
    const sources = props.item.MediaSources ?? [];
    return sources.map(source => `${formatBytes(source.Size)} · ${resolutionLabel(source)}`);
});
</script>

<style scoped>
.eprow {
    display: flex;
    align-items: center;
    gap: 0.875rem;
    padding: 0.5rem 0.625rem;
    border-radius: var(--r-sm);
    cursor: pointer;
    transition: background var(--dur-fast) var(--ease);
}

.eprow:hover {
    background: var(--surface-hover);
}

.eprow.is-active {
    background: var(--lamp-wash);
    box-shadow: inset 2px 0 0 var(--lamp);
}

.eprow__num {
    flex: none;
    width: 1.75rem;
    color: var(--text-faint);
    font-family: var(--font-num);
    font-size: var(--fs-sm);
    font-variant-numeric: tabular-nums;
    text-align: right;
}

.eprow__main {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    flex: auto;
    min-width: 0;
}

.eprow__name {
    flex: 1 1 12rem;
    min-width: 0;
    color: var(--text);
    font-size: var(--fs-md);
}

.eprow__sub {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: none;
    font-size: var(--fs-xs);
}

.eprow__pending {
    position: static;
    flex: none;
}

.eprow__ops {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
    opacity: 0;
    transition: opacity var(--dur-fast) var(--ease);
}

.eprow:hover .eprow__ops,
.eprow__ops:focus-within {
    opacity: 1;
}

.eprow.is-active .eprow__ops {
    opacity: 1;
}
</style>
