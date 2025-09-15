import { defineStore } from 'pinia';
import { ref } from 'vue';
import { BaseItem, EpisodeItem, MediaLibraryItem } from '../api/embyApi';
import { useGlobalConfig } from './db/globalConfig';
import { useRuntimeConfig } from './runtimeConfig';

export const useImage = defineStore('image', () => {
    const images = ref<{[key: string]: string}>({})

    async function loadImage(imageKey: string, embyServerId: string, itemId: string, imageType: string) {
        const disabledImage = await useGlobalConfig().getGlobalConfigValue("disabledImage") || 'off'
        if (disabledImage != 'on') {
            return;
        }
        let port = useRuntimeConfig().runtimeConfig!.axum_port;
        let url = `http://127.0.0.1:${port}/image?param.type=Emby&param.emby_server_id=${embyServerId}&param.item_id=${itemId}&param.image_type=${imageType}`;
        images.value[imageKey] = url
        return images.value[imageKey];
    }

    async function loadLogo(embyServerId: string, item: EpisodeItem) {
        const imageKey = embyServerId + ':logo:' + item.Id
        if (images.value[imageKey]) {
            return images.value[imageKey];
        }
        if (item.ParentLogoItemId) {
            loadImage(imageKey, embyServerId, item.ParentLogoItemId, 'Logo')
        } else if (item.ImageTags && item.ImageTags.Logo) {
            loadImage(imageKey, embyServerId, item.Id, 'Logo')
        }
    }

    async function loadCover(embyServerId: string, item: BaseItem | MediaLibraryItem) {
        const imageKey = embyServerId + ':cover:' + item.Id
        if (images.value[imageKey]) {
            return images.value[imageKey];
        }
        let loaded = true;
        if (item.ImageTags) {
            if (item.ImageTags.Primary) {
                loadImage(imageKey, embyServerId, item.Id, 'Primary')
            } else if (item.ImageTags.Thumb) {
                loadImage(imageKey, embyServerId, item.Id, 'Thumb')
            } else if (item.ImageTags.Art) {
                loadImage(imageKey, embyServerId, item.Id, 'Art')
            } else if (item.ImageTags.Banner) {
                loadImage(imageKey, embyServerId, item.Id, 'Banner')
            } else if (item.ImageTags.Logo) {
                loadImage(imageKey, embyServerId, item.Id, 'Logo')
            } else {
                loaded = false;
            }
        }
        if (!loaded) {
            if ((item as EpisodeItem).SeriesPrimaryImageTag) {
                loadImage(imageKey, embyServerId, (item as EpisodeItem).SeriesId, 'Primary')
            } else if ((item as EpisodeItem).ParentThumbItemId) {
                loadImage(imageKey, embyServerId, (item as EpisodeItem).ParentThumbItemId, 'Thumb')
            }
        }
    }

    async function loadIcon(icon_url: string) {
        let port = useRuntimeConfig().runtimeConfig!.axum_port;
        return `http://127.0.0.1:${port}/image?param.type=Icon&param.image_url=${encodeURIComponent(icon_url)}`;
    }

    return { images, loadImage, loadIcon, loadLogo, loadCover }
})
