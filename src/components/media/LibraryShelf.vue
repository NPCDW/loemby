<template>
    <AppSection :title="title" :caption="caption">
        <template v-if="$slots.actions" #actions>
            <slot name="actions" />
        </template>

        <div v-if="loading" class="shelf">
            <span v-for="cell in 6" :key="cell" class="shelf__skeleton" :class="wide ? 'shelf__skeleton--wide' : ''" />
        </div>

        <div v-else-if="items.length" class="shelf">
            <button
                v-for="item in items"
                :key="item.Id"
                class="shelf__cell clickable"
                type="button"
                :title="item.Name"
                @click="emit('open', item)"
            >
                <span class="cover" :class="wide ? 'cover--wide shelf__wide' : 'cover--poster shelf__poster'">
                    <img v-if="coverOf(item)" v-lazy="coverOf(item)" :alt="item.Name" />
                    <span v-else class="shelf__blank" />
                    <span v-if="wide" class="shelf__label t-clip">{{ item.Name }}</span>
                </span>
                <span v-if="!wide" class="shelf__caption t-clip">{{ item.Name }}</span>
            </button>
        </div>

        <AppEmpty v-else :title="emptyTitle" :hint="emptyHint" />
    </AppSection>
</template>

<script setup lang="ts">
import AppSection from '../base/AppSection.vue';
import AppEmpty from '../base/AppEmpty.vue';
import type { MediaLibraryItem, SearchItem } from '../../api/embyApi';
import { useImage } from '../../store/image';

/**
 * 横向天桥。
 *
 * 两种形态：
 *  - wide   —— 媒体库本身，16:9 大图 + 图上标题
 *  - poster —— 库内最新条目，2:3 海报 + 图下标题
 * 超出宽度时只横向滚动，不换行，避免变成砖墙。
 */
const props = withDefaults(
    defineProps<{
        title: string;
        caption?: string;
        items: Array<SearchItem | MediaLibraryItem>;
        embyServerId: string;
        wide?: boolean;
        loading?: boolean;
        emptyTitle?: string;
        emptyHint?: string;
    }>(),
    {
        caption: '',
        wide: false,
        loading: false,
        emptyTitle: '这里还是空的',
        emptyHint: '稍后再来，或先在服务器上补全媒体库。',
    },
);

const emit = defineEmits<{ open: [item: SearchItem | MediaLibraryItem] }>();

const image = useImage();

function coverOf(item: SearchItem | MediaLibraryItem): string {
    return image.images[`${props.embyServerId}:cover:${item.Id}`] ?? '';
}
</script>

<style scoped>
.shelf__cell {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
}

.shelf__wide {
    width: 14.5rem;
}

.shelf__poster {
    width: 7.25rem;
}

.shelf__wide img,
.shelf__poster img {
    width: 100%;
    height: 100%;
}

.shelf__blank {
    display: block;
    width: 100%;
    height: 100%;
    background: var(--surface-hover);
}

/* 大图上直接压标题，加一层底部渐隐保证可读 */
.shelf__label {
    position: absolute;
    right: 0.75rem;
    bottom: 0.625rem;
    left: 0.75rem;
    color: #fff;
    font-size: var(--fs-lg);
    font-weight: 600;
    text-shadow: 0 1px 10px rgba(0, 0, 0, 0.92);
}

.shelf__caption {
    color: var(--text-dim);
    font-size: var(--fs-sm);
}

.shelf__cell:hover .shelf__caption {
    color: var(--text);
}

.shelf__skeleton {
    flex: none;
    width: 7.25rem;
    aspect-ratio: 2 / 3;
    border-radius: var(--r-md);
    background: var(--surface-card);
}

.shelf__skeleton--wide {
    width: 14.5rem;
    aspect-ratio: 16 / 9;
}
</style>
