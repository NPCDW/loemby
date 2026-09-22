<template>
    <img v-if="src" class="server-icon" :src="src" :alt="alt" />
    <svg-icon v-else name="emby" :size="size" color="currentColor" />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useImage } from '../../store/image';

/**
 * 服务器图标。
 *
 * 图标同样走本地 axum 的 /image/icon 端点（带代理与缓存），
 * 这里只负责在 icon_url 变化时重新解析一次本地地址。
 */
const props = withDefaults(
    defineProps<{
        iconUrl?: string;
        size?: number;
        alt?: string;
    }>(),
    { iconUrl: '', size: 16, alt: '' },
);

const image = useImage();
const src = ref('');

async function resolve() {
    src.value = props.iconUrl ? image.iconUrl(props.iconUrl) : '';
}

watch(() => props.iconUrl, resolve, { immediate: true });
</script>

<style scoped>
.server-icon {
    display: block;
    width: v-bind('props.size + "px"');
    height: v-bind('props.size + "px"');
    border-radius: var(--r-xs);
    object-fit: contain;
}
</style>
