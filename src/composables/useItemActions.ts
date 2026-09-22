import { reactive } from 'vue';
import { ElMessage } from 'element-plus';
import embyApi, { type UserData } from '../api/embyApi';

/**
 * 条目的收藏 / 已播放标记。
 *
 * 原先每个页面各写一份 star()/played()，连 loading 的键都是各写各的；
 * 这里统一成一份，用 itemId 做在途标记，乐观地就地改 UserData。
 */

export interface ActionableItem {
    Id: string;
    UserData?: UserData;
}

export interface ItemActions {
    starPending: Record<string, boolean>;
    playedPending: Record<string, boolean>;
    toggleStar: (item: ActionableItem) => Promise<void>;
    togglePlayed: (item: ActionableItem) => Promise<void>;
    isStarPending: (id: string) => boolean;
    isPlayedPending: (id: string) => boolean;
}

export function useItemActions(embyServerId: string): ItemActions {
    const starPending = reactive<Record<string, boolean>>({});
    const playedPending = reactive<Record<string, boolean>>({});

    async function toggleStar(item: ActionableItem): Promise<void> {
        if (!item.UserData || starPending[item.Id]) {
            return;
        }
        starPending[item.Id] = true;
        try {
            const userData = item.UserData.IsFavorite
                ? await embyApi.unstar(embyServerId, item.Id)
                : await embyApi.star(embyServerId, item.Id);
            item.UserData.IsFavorite = userData.IsFavorite;
        } catch (e) {
            ElMessage.error('收藏操作失败 ' + e);
        } finally {
            starPending[item.Id] = false;
        }
    }

    async function togglePlayed(item: ActionableItem): Promise<void> {
        if (!item.UserData || playedPending[item.Id]) {
            return;
        }
        playedPending[item.Id] = true;
        try {
            const userData = item.UserData.Played
                ? await embyApi.unplayed(embyServerId, item.Id)
                : await embyApi.played(embyServerId, item.Id);
            item.UserData.Played = userData.Played;
        } catch (e) {
            ElMessage.error('标记播放状态失败 ' + e);
        } finally {
            playedPending[item.Id] = false;
        }
    }

    return {
        starPending,
        playedPending,
        toggleStar,
        togglePlayed,
        isStarPending: (id: string) => !!starPending[id],
        isPlayedPending: (id: string) => !!playedPending[id],
    };
}
