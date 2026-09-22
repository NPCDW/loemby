import { ref, type Ref } from 'vue';
import { ElMessage } from 'element-plus';
import { useGlobalConfig } from '../store/db/globalConfig';
import { useEventBus } from '../store/eventBus';

/**
 * 设置项句柄。
 *
 * 本身就是一个 ref，因此模板里可以 `v-model="xxx"` 直接用、
 * 脚本里 `xxx.value` 读写；附带 save / load 两个方法。
 * 这样就不存在「多包一层 value」带来的解包歧义。
 */
export type GlobalSetting<T extends string | number> = Ref<T> & {
    /** 把当前值写回配置；不传则用当前值 */
    save: (next?: T) => Promise<void>;
    /** 重新从配置读一次 */
    load: () => Promise<void>;
};

/**
 * 单个全局配置项的双向绑定。
 *
 * 设置页里最容易写歪的是一堆「读一次 / 存一次 / 失败怎么办」，
 * 这里把它们收敛成 ref + 声明式保存：
 *
 *   const level = useGlobalSetting('log_level', 'info')
 *   // 模板：v-model="level.value"（模板会自动解包）
 *   // 脚本：level.current.value
 *
 * 保存失败时回滚到落库前的值，避免界面显示一个并未生效的状态。
 */
export function useGlobalSetting<T extends string | number>(
    key: string,
    fallback: T,
    options?: {
        /** 读取时的转换，例如把字符串转成数字 */
        parse?: (raw: string) => T;
        /** 写入前的序列化 */
        serialize?: (value: T) => string;
        /** 保存成功后的副作用，例如刷新代理名缓存、广播 GlobalProxyChanged */
        after?: () => void;
        /** 是否在创建时立即读取一次 */
        immediate?: boolean;
    },
): GlobalSetting<T> {
    const value = ref(fallback) as Ref<T>;
    const config = useGlobalConfig();
    const bus = useEventBus();

    const serialize = options?.serialize ?? ((input: T) => String(input));

    async function load(): Promise<void> {
        try {
            const raw = await config.getGlobalConfigValue(key);
            value.value = raw ? (options?.parse ? options.parse(raw) : (raw as T)) : fallback;
        } catch (e) {
            ElMessage.error(`读取「${key}」失败 ` + e);
        }
    }

    async function save(next?: T): Promise<void> {
        const target = next ?? value.value;
        try {
            const existing = await config.getGlobalConfig(key);
            if (existing) {
                await config.updateGlobalConfig({ ...existing, config_value: serialize(target) });
            } else {
                await config.addGlobalConfig({ config_key: key, config_value: serialize(target) });
            }
            value.value = target;
            options?.after?.();
            bus.emit('GlobalSettingSaved', { key });
        } catch (e) {
            ElMessage.error(`保存「${key}」失败 ` + e);
            await load();
        }
    }

    if (options?.immediate !== false) {
        void load();
    }

    return Object.assign(value, { load, save });
}
