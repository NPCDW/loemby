import { defineStore } from 'pinia';
import { ref } from 'vue';
import { BaseItem, EpisodeItem, MediaLibraryItem } from '../api/embyApi';
import { useGlobalConfig } from './db/globalConfig';
import { EmbyServer } from './db/embyServer';
import { useRuntimeConfig } from './runtimeConfig';

export const useImage = defineStore('image', () => {
    const images = ref<{[key: string]: string}>({})

    async function loadImage(imageKey: string, embyServer: EmbyServer, itemId: string, imageType: string) {
        const disabledImage = await useGlobalConfig().getGlobalConfigValue("disabledImage") || 'off'
        if (disabledImage != 'on') {
            return;
        }
        let port = useRuntimeConfig().runtimeConfig!.axum_port;
        let url = `http://127.0.0.1:${port}/image?param.type=Emby&param.emby_server_id=${embyServer.id!}&param.item_id=${itemId}&param.image_type=${imageType}`;
        images.value[imageKey] = url
        return images.value[imageKey];
    }

    async function loadLogo(embyServer: EmbyServer, item: EpisodeItem) {
        const imageKey = embyServer.id + ':logo:' + item.Id
        if (images.value[imageKey]) {
            return images.value[imageKey];
        }
        if (item.ParentLogoItemId) {
            loadImage(imageKey, embyServer, item.ParentLogoItemId, 'Logo')
        } else if (item.ImageTags && item.ImageTags.Logo) {
            loadImage(imageKey, embyServer, item.Id, 'Logo')
        }
    }

    async function loadCover(embyServer: EmbyServer, item: BaseItem | MediaLibraryItem) {
        const imageKey = embyServer.id + ':cover:' + item.Id
        if (images.value[imageKey]) {
            return images.value[imageKey];
        }
        let loaded = true;
        if (item.ImageTags) {
            if (item.ImageTags.Primary) {
                loadImage(imageKey, embyServer, item.Id, 'Primary')
            } else if (item.ImageTags.Thumb) {
                loadImage(imageKey, embyServer, item.Id, 'Thumb')
            } else if (item.ImageTags.Art) {
                loadImage(imageKey, embyServer, item.Id, 'Art')
            } else if (item.ImageTags.Banner) {
                loadImage(imageKey, embyServer, item.Id, 'Banner')
            } else if (item.ImageTags.Logo) {
                loadImage(imageKey, embyServer, item.Id, 'Logo')
            } else {
                loaded = false;
            }
        }
        if (!loaded) {
            if ((item as EpisodeItem).SeriesPrimaryImageTag) {
                loadImage(imageKey, embyServer, (item as EpisodeItem).SeriesId, 'Primary')
            } else if ((item as EpisodeItem).ParentThumbItemId) {
                loadImage(imageKey, embyServer, (item as EpisodeItem).ParentThumbItemId, 'Thumb')
            }
        }
    }

    async function loadIcon(icon_url: string) {
        let port = useRuntimeConfig().runtimeConfig!.axum_port;
        return `http://127.0.0.1:${port}/image?param.type=Icon&param.image_url=${encodeURIComponent(icon_url)}`;
    }

    return { images, loadImage, loadIcon, loadLogo, loadCover }
})
