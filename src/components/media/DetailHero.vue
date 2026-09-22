<template>
    <header class="hero">
        <span v-if="!bare" class="hero__poster cover cover--poster">
            <img v-if="poster" v-lazy="poster" :alt="title" />
            <span v-else class="hero__blank" />
        </span>

        <div class="hero__main">
            <div v-if="eyebrow" class="hero__eyebrow">
                <slot name="eyebrow">{{ eyebrow }}</slot>
            </div>

            <h1 class="t-hero">{{ title }}</h1>

            <div v-if="facts.length" class="hero__facts">
                <span v-for="fact in facts" :key="fact.value" :class="fact.mono ? 'mono hero__fact-mono' : ''">
                    {{ fact.value }}
                </span>
            </div>

            <p v-if="overview" class="hero__overview">{{ overview }}</p>

            <div v-if="$slots.actions || $slots.meta" class="hero__actions">
                <slot name="actions" />
                <span v-if="$slots.meta" class="hero__meta">
                    <slot name="meta" />
                </span>
            </div>
        </div>

        <div v-if="$slots.aside" class="hero__aside">
            <slot name="aside" />
        </div>
    </header>
</template>

<script setup lang="ts">
import { computed } from 'vue';

/**
 * 详情页主视觉：海报 + 标题 + 事实行 + 简介 + 操作。
 *
 * 标题是全页唯一的大字号；事实行里只有数字用等宽体，
 * 这样「2024 · 3 季 · 1080p」不会因为位数变化而左右抖动。
 */
export interface HeroFact {
    value: string;
    mono?: boolean;
}

const props = withDefaults(
    defineProps<{
        title: string;
        poster?: string;
        /** 标题上方的一行小字，例如所属剧名（已经用 slot 时忽略） */
        eyebrow?: string;
        /** 事实行内容，按顺序渲染 */
        facts?: Array<string | HeroFact>;
        overview?: string;
        /** 没有海报时（如播放详情）用 bare */
        bare?: boolean;
    }>(),
    { poster: '', eyebrow: '', facts: () => [], overview: '', bare: false },
);

const facts = computed<HeroFact[]>(() =>
    props.facts.map(fact => (typeof fact === 'string' ? { value: fact, mono: true } : fact)),
);
</script>

<style scoped>
.hero {
    display: flex;
    align-items: flex-start;
    gap: 2rem;
}

.hero__poster {
    flex: none;
    width: 14rem;
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.5);
}

.hero__poster img {
    width: 100%;
    height: 100%;
}

.hero__blank {
    display: block;
    width: 100%;
    height: 100%;
    background: var(--surface-hover);
}

.hero__main {
    flex: auto;
    min-width: 0;
    padding-top: 0.25rem;
}

.hero__eyebrow {
    margin-bottom: 0.375rem;
    color: var(--text-faint);
    font-size: var(--fs-md);
}

.hero__facts {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.875rem;
    color: var(--text-faint);
    font-size: var(--fs-sm);
}

.hero__fact-mono {
    font-family: var(--font-num);
    font-variant-numeric: tabular-nums;
}

.hero__overview {
    max-width: 70ch;
    margin: 1rem 0 0;
    color: var(--text-dim);
    line-height: 1.8;
    white-space: pre-line;
}

.hero__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1.375rem;
}

.hero__meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-left: 0.5rem;
}

.hero__aside {
    flex: none;
    max-width: 22rem;
}
</style>
