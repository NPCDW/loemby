<template>
    <article class="resume clickable" :class="{ 'is-dismissed': dismissed }" @click="emit('open', item)">
        <span class="resume__art cover cover--wide">
            <img v-if="art" v-lazy="art" :alt="item.Name" />
            <span v-else class="resume__blank" />
            <span class="bar">
                <span class="bar__fill" :style="{ width: percent + '%' }" />
            </span>
        </span>

        <div class="resume__body">
            <div class="resume__headline">
                <span v-if="item.Type === 'Episode' && item.SeriesName" class="resume__series t-clip">
                    {{ item.SeriesName }}
                </span>
                <span class="resume__name t-clamp-2">{{ item.Name }}</span>
            </div>

            <div class="resume__foot">
                <span v-if="item.Type === 'Episode'" class="num">{{ formatEpisodeNo(item.ParentIndexNumber, item.IndexNumber) }}</span>
                <span class="resume__percent mono">{{ percentLabel(item.UserData) }}</span>
                <span class="resume__ops" @click.stop>
                    <button
                        v-if="!dismissed"
                        class="ghost ghost--bare"
                        type="button"
                        title="从继续观看中移除"
                        :disabled="pending"
                        @click="emit('dismiss')"
                    >
                        <el-icon :size="14" :class="{ 'is-loading': pending }"><i-ep-Close /></el-icon>
                    </button>
                    <button
                        v-else
                        class="ghost ghost--bare"
                        type="button"
                        title="恢复到继续观看"
                        :disabled="pending"
                        @click="emit('restore')"
                    >
                        <el-icon :size="14" :class="{ 'is-loading': pending }"><i-ep-RefreshLeft /></el-icon>
                    </button>
                </span>
            </div>
        </div>
    </article>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { EpisodeItem } from '../../api/embyApi';
import { formatEpisodeNo, percentLabel, progressOf } from '../../util/format';
import { useImage } from '../../store/image';

/**
 * 继续观看卡：宽幅横卡。
 *
 * 封面用横向比例（剧集主图），右侧一列是「剧名 / 集名 / 进度 / 操作」。
 * 移除后不消失，而是变暗并给出「恢复」，避免误操作不可逆。
 */
const props = withDefaults(
    defineProps<{
        item: EpisodeItem;
        embyServerId: string;
        dismissed?: boolean;
        pending?: boolean;
    }>(),
    { dismissed: false, pending: false },
);

const emit = defineEmits<{ open: [item: EpisodeItem]; dismiss: []; restore: [] }>();

const image = useImage();

const percent = computed(() => progressOf(props.item.UserData));

const art = computed(() => {
    const parentKey = `${props.embyServerId}:parent-cover:${props.item.Id}`;
    const coverKey = `${props.embyServerId}:cover:${props.item.Id}`;
    return image.images[parentKey] ?? image.images[coverKey] ?? '';
});
</script>

<style scoped>
.resume {
    display: flex;
    gap: 0.75rem;
    padding: 0.625rem;
    width: 100%;
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    background: var(--surface-card);
    cursor: pointer;
    transition: border-color var(--dur) var(--ease), background var(--dur) var(--ease);
}

.resume:hover {
    border-color: var(--line-strong);
    background: var(--surface-hover);
}

.resume.is-dismissed {
    opacity: 0.42;
}

.resume__art {
    flex: none;
    width: 11rem;
    align-self: center;
}

.resume__art img {
    width: 100%;
    height: 100%;
}

.resume__blank {
    display: block;
    width: 100%;
    height: 100%;
    background: var(--surface-hover);
}

.resume__body {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 0.5rem;
    flex: auto;
    min-width: 0;
    padding: 0.125rem 0.25rem 0.125rem 0;
}

.resume__headline {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.resume__series {
    color: var(--text-faint);
    font-size: var(--fs-xs);
}

.resume__name {
    color: var(--text);
    font-size: var(--fs-lg);
    font-weight: 600;
    line-height: 1.35;
}

.resume:hover .resume__name {
    color: var(--lamp-bright);
}

.resume__foot {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: var(--fs-xs);
}

.resume__percent {
    color: var(--lamp);
}

.resume__ops {
    display: flex;
    align-items: center;
    margin-left: auto;
}
</style>
