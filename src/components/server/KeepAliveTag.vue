<template>
    <span v-if="days !== null" class="keepalive mono" :data-level="level" :title="title">{{ days }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import dayjs from 'dayjs';

/**
 * 「距离上次观看还有几天」提醒。
 *
 * 服务端给出 keep_alive_days 与上次播放时间，这里换算成剩余天数。
 * 剩余 <= 3 天是琥珀、已过期是红，其余是绿——只有真的需要行动时才亮色。
 */
const props = defineProps<{
    keepAliveDays?: number;
    lastPlaybackTime?: string;
}>();

const remaining = computed(() => {
    if (!props.keepAliveDays || !props.lastPlaybackTime) {
        return null;
    }
    const elapsed = dayjs().diff(props.lastPlaybackTime, 'day');
    return props.keepAliveDays - elapsed;
});

const days = computed(() => remaining.value);

const level = computed(() => {
    const value = remaining.value ?? 0;
    if (value > 7) {
        return 'ok';
    }
    if (value > 3) {
        return 'soon';
    }
    return 'over';
});

const title = computed(() => {
    const value = remaining.value ?? 0;
    return value >= 0
        ? `距离上次观看 ${dayjs().diff(props.lastPlaybackTime, 'day')} 天，还剩 ${value} 天提醒`
        : `已超过 ${-value} 天未观看`;
});
</script>

<style scoped>
.keepalive {
    flex: none;
    padding: 0 5px;
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    color: var(--text-faint);
    font-size: var(--fs-xs);
    line-height: 1.6;
}

.keepalive[data-level='ok'] {
    color: var(--ok);
    border-color: rgba(92, 178, 109, 0.4);
}

.keepalive[data-level='soon'] {
    color: var(--lamp);
    border-color: var(--lamp-edge);
}

.keepalive[data-level='over'] {
    color: var(--bad);
    border-color: rgba(216, 98, 90, 0.45);
}
</style>
