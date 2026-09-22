import { computed, ref, type ComputedRef, type Ref } from 'vue';
import { ElMessage } from 'element-plus';
import { useEventBus } from '../store/eventBus';
import { useEmbyServer, type EmbyServer } from '../store/db/embyServer';

/**
 * 服务器列表的单一来源。
 *
 * 原先导航、历史、设置、聚合搜索各自维护一份 listAllEmbyServer()，
 * 排序规则也各写一遍；这里统一，并订阅 EmbyServerChanged 保持同步。
 */
export function useEmbyServers(): {
    servers: Ref<EmbyServer[]>;
    enabledServers: ComputedRef<EmbyServer[]>;
    serverMap: ComputedRef<Record<string, EmbyServer>>;
    refresh: () => Promise<void>;
    byId: (id?: string) => EmbyServer | undefined;
} {
    const servers = ref<EmbyServer[]>([]);

    function sortServers(list: EmbyServer[]): EmbyServer[] {
        return [...list].sort((a, b) => (a.order_by ?? 0) - (b.order_by ?? 0));
    }

    async function refresh(): Promise<void> {
        try {
            servers.value = sortServers(await useEmbyServer().listAllEmbyServer());
        } catch (e) {
            ElMessage.error('获取服务器列表失败 ' + e);
        }
    }

    const bus = useEventBus();
    bus.on('EmbyServerChanged', refresh);
    void refresh();

    const serverMap = computed(() => {
        const map: Record<string, EmbyServer> = {};
        for (const server of servers.value) {
            if (server.id) {
                map[server.id] = server;
            }
        }
        return map;
    });

    return {
        servers,
        enabledServers: computed(() => servers.value.filter(server => !server.disabled)),
        serverMap,
        refresh,
        byId: (id?: string) => (id ? serverMap.value[id] : undefined),
    };
}
