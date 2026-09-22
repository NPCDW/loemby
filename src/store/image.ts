import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { BaseItem, EpisodeItem, MediaLibraryItem } from '../api/embyApi';
import { useGlobalConfig } from './db/globalConfig';
import { useRuntimeConfig } from './runtimeConfig';

/**
 * 图片地址中心。
 *
 * 图片不由前端直接请求 Emby，而是走本地 axum 的 /image 端点：
 * 它负责带 token、走代理、落盘缓存。因此这里的返回值都是本地 URL。
 *
 * 键约定：`<serverId>:<kind>:<itemId>`，kind ∈ cover | parent-cover | logo
 */
export const useImage = defineStore('image', () => {
    const images = ref<Record<string, string>>({});

    function localUrl(embyServerId: string, itemId: string, imageType: string): string {
        const port = useRuntimeConfig().runtimeConfig!.axum_port;
        return `http://127.0.0.1:${port}/image/emby?emby_server_id=${embyServerId}&item_id=${itemId}&image_type=${imageType}`;
    }

    function iconUrl(iconUrl: string): string {
        const port = useRuntimeConfig().runtimeConfig!.axum_port;
        return `http://127.0.0.1:${port}/image/icon?image_url=${encodeURIComponent(iconUrl)}`;
    }

    async function enabled(): Promise<boolean> {
        const disabledImage = (await useGlobalConfig().getGlobalConfigValue('disabledImage')) || 'off';
        return disabledImage !== 'on';
    }

    /** 写入并返回地址。图片被全局关闭时返回空串，模板里会退化成占位块 */
    async function put(key: string, embyServerId: string, itemId: string, imageType: string): Promise<string> {
        if (!(await enabled())) {
            return '';
        }
        images.value[key] = localUrl(embyServerId, itemId, imageType);
        return images.value[key];
    }

    /**
     * 封面优先序：Primary → Thumb → Art → Banner → Logo，
     * 都没有时退回所属剧集的主图，保证海报墙不出现空洞。
     */
    async function loadCover(embyServerId: string, item: BaseItem | MediaLibraryItem): Promise<string> {
        const key = `${embyServerId}:cover:${item.Id}`;
        if (images.value[key] !== undefined) {
            return images.value[key];
        }
        const tags = item.ImageTags;
        const preferred: Array<[string | undefined, string]> = [
            [tags?.Primary, 'Primary'],
            [tags?.Thumb, 'Thumb'],
            [tags?.Art, 'Art'],
            [tags?.Banner, 'Banner'],
            [tags?.Logo, 'Logo'],
        ];
        const hit = preferred.find(([tag]) => !!tag);
        if (hit) {
            return put(key, embyServerId, item.Id, hit[1]);
        }
        const episode = item as EpisodeItem;
        if (episode.SeriesPrimaryImageTag) {
            return put(key, embyServerId, episode.SeriesId, 'Primary');
        }
        if (episode.ParentThumbItemId) {
            return put(key, embyServerId, episode.ParentThumbItemId, 'Thumb');
        }
        images.value[key] = '';
        return '';
    }

    /** 继续观看用的横图：优先所属剧集的主图，退回季缩略图 */
    async function loadParentCover(embyServerId: string, item: EpisodeItem): Promise<string> {
        const key = `${embyServerId}:parent-cover:${item.Id}`;
        if (images.value[key] !== undefined) {
            return images.value[key];
        }
        if (item.SeriesPrimaryImageTag && item.SeriesId) {
            return put(key, embyServerId, item.SeriesId, 'Primary');
        }
        if (item.ParentThumbItemId) {
            return put(key, embyServerId, item.ParentThumbItemId, 'Thumb');
        }
        return loadCover(embyServerId, item);
    }

    /** 播放详情页的标题图形，优先用父级 logo */
    async function loadLogo(embyServerId: string, item: EpisodeItem): Promise<string> {
        const key = `${embyServerId}:logo:${item.Id}`;
        if (images.value[key] !== undefined) {
            return images.value[key];
        }
        if (item.ParentLogoItemId) {
            return put(key, embyServerId, item.ParentLogoItemId, 'Logo');
        }
        if (item.ImageTags?.Logo) {
            return put(key, embyServerId, item.Id, 'Logo');
        }
        images.value[key] = '';
        return '';
    }

    /** 剧集详情页的横版背景（Thumb / Backdrop 语义） */
    async function loadBackdrop(embyServerId: string, item: BaseItem): Promise<string> {
        const key = `${embyServerId}:backdrop:${item.Id}`;
        if (images.value[key] !== undefined) {
            return images.value[key];
        }
        const tags = item.ImageTags;
        if (tags?.Thumb) {
            return put(key, embyServerId, item.Id, 'Thumb');
        }
        if (tags?.Art) {
            return put(key, embyServerId, item.Id, 'Art');
        }
        if (tags?.Primary) {
            return put(key, embyServerId, item.Id, 'Primary');
        }
        images.value[key] = '';
        return '';
    }

    /** 批量预加载封面，列表页拿到数据后调用一次即可 */
    async function warmCovers(embyServerId: string, items: Array<BaseItem | MediaLibraryItem | undefined>): Promise<void> {
        await Promise.all(items.filter(Boolean).map(item => loadCover(embyServerId, item!)));
    }

    return {
        images,
        localUrl,
        iconUrl,
        loadCover,
        loadParentCover,
        loadLogo,
        loadBackdrop,
        warmCovers,
    };
});
