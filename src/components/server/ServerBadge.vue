<template>
    <span class="badge" :title="stateLabel">
        <span class="badge__dot" :data-state="state" />
        <span class="badge__text">{{ stateLabel }}</span>
    </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

/**
 * 服务器状态：一枚圆点 + 一个词。
 *
 * 三种状态：在线 / 未登录 / 已停用。列表里不用堆标签。
 */
const props = defineProps<{
    disabled?: number;
    authenticated?: boolean;
    /** 只显示圆点，文字由外部决定是否出现 */
    dotOnly?: boolean;
}>();

const state = computed(() => {
    if (props.disabled) {
        return 'off';
    }
    return props.authenticated ? 'on' : 'unsigned';
});

const stateLabel = computed(() => {
    if (props.disabled) {
        return '已停用';
    }
    return props.authenticated ? '在线' : '未登录';
});
</script>

<style scoped>
.badge {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: var(--fs-xs);
}

.badge__dot {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-ghost);
}

.badge__dot[data-state='on'] {
    background: var(--ok);
}

.badge__dot[data-state='unsigned'] {
    background: var(--lamp);
}

.badge__dot[data-state='off'] {
    background: var(--text-ghost);
}

.badge__text {
    color: var(--text-faint);
}
</style>
