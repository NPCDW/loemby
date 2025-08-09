import { defineStore } from 'pinia';
import { ref } from 'vue';
import embyApi, { BaseItem, EpisodeItem, MediaLibraryItem } from '../api/embyApi';
import { useGlobalConfig } from './db/globalConfig';
import invokeApi from '../api/invokeApi';
import { useProxyServer } from './db/proxyServer';
import { EmbyServer } from './db/embyServer';

export const useImage = defineStore('image', () => {
    const images = ref<{[key: string]: string}>({})

    async function loadImage(imageKey: string, embyServer: EmbyServer, itemId: string, imageType: string) {
        if (images.value[imageKey]) {
            return images.value[imageKey];
        }
        const image_url = await embyApi.getImageUrl(embyServer, itemId, imageType);
        if (image_url) {
            const disabledCache = await useGlobalConfig().getGlobalConfigValue("disabledImage") || 'off'
            images.value[imageKey] = invokeApi.loadImage({
                image_url,
                proxy_url: await useProxyServer().getBrowseProxyUrl(embyServer.browse_proxy_id),
                user_agent: embyServer.user_agent!,
                cache_prefix: ['image', embyServer.id!, imageType],
                disabled_cache: disabledCache == 'on',
            })
            return images.value[imageKey];
        }
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
        const disabledCache = await useGlobalConfig().getGlobalConfigValue("disabledImage") || 'off'
        return invokeApi.loadImage({
            image_url: icon_url,
            proxy_url: await useProxyServer().getAppProxyUrl(),
            user_agent: 'loemby/' + import.meta.env.VITE_APP_VERSION,
            cache_prefix: ['icon'],
            disabled_cache: disabledCache == 'on',
        })
    }

    return { images, loadImage, loadIcon, loadLogo, loadCover }
})
