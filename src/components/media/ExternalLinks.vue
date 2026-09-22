<template>
    <div class="ext">
        <button
            v-for="link in links"
            :key="link.Url"
            class="ext__item"
            type="button"
            :title="link.Url"
            @click="invokeApi.open_url(link.Url)"
        >
            <svg-icon v-if="providerOf(link.Url) === 'imdb'" name="imdb" size="17" />
            <svg-icon v-else-if="providerOf(link.Url) === 'tmdb'" name="tmdb" size="17" />
            <svg-icon v-else-if="providerOf(link.Url) === 'tvdb'" name="tvdb" size="17" />
            <svg-icon v-else-if="providerOf(link.Url) === 'trakt'" name="trakt" size="17" />
            <svg-icon v-else-if="providerOf(link.Url) === 'myanimelist'" name="myanimelist" size="17" />
            <img
                v-else-if="providerOf(link.Url) === 'anidb'"
                src="../../icons/anidb.png"
                alt="anidb"
                style="width: 17px; height: 17px"
            />
            <el-icon v-else :size="15"><i-ep-Link /></el-icon>
            <span>{{ link.Name }}</span>
        </button>

        <span v-for="(value, key) in providerIds" :key="key" class="tag">{{ key }}:{{ value }}</span>
    </div>
</template>

<script setup lang="ts">
import type { ExternalUrl } from '../../api/embyApi';
import invokeApi from '../../api/invokeApi';

/**
 * 外部信息：站外链接用各自的品牌图形，站内 id 用等宽小签。
 * 链接一律交给系统浏览器打开，不在应用内嵌套网页。
 */
withDefaults(
    defineProps<{
        links?: ExternalUrl[];
        providerIds?: Record<string, string>;
    }>(),
    { links: () => [], providerIds: () => ({}) },
);

const PROVIDERS: Array<[string, string]> = [
    ['imdb.com', 'imdb'],
    ['themoviedb.org', 'tmdb'],
    ['thetvdb.com', 'tvdb'],
    ['trakt.tv', 'trakt'],
    ['myanimelist.net', 'myanimelist'],
    ['anidb.net', 'anidb'],
];

function providerOf(url: string): string {
    return PROVIDERS.find(([needle]) => url.includes(needle))?.[1] ?? '';
}
</script>

<style scoped>
.ext {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
}

.ext__item {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.3125rem 0.625rem;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-faint);
    font-size: var(--fs-xs);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.ext__item:hover {
    color: var(--text);
    border-color: var(--line-strong);
}
</style>
