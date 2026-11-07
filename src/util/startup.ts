import { ref } from 'vue';
import type { RuntimeConfig } from '../store/runtimeConfig';

// 略长于后端的 30 秒 ready 等待，避免正常的后端失败被前端超时掩盖。
export const runtimeStartupTimeout = 35_000;

export function createRuntimeStartup(fetchRuntimeConfig: () => Promise<RuntimeConfig>, timeoutMs = runtimeStartupTimeout) {
    const runtimeConfig = ref<RuntimeConfig>();
    const startupStatus = ref<'pending' | 'ready' | 'failed'>('pending');
    const startupError = ref('');
    const startupFailure = ref<'backend' | 'timeout' | null>(null);
    let pending: Promise<void> | undefined;

    function getRuntimeConfig(): Promise<void> {
        if (pending) return pending;
        if (startupStatus.value === 'ready') return Promise.resolve();

        startupStatus.value = 'pending';
        startupError.value = '';
        startupFailure.value = null;
        const timeout = new Error('runtime startup timeout');
        let timer: ReturnType<typeof setTimeout>;
        const request = Promise.race([
            Promise.resolve().then(fetchRuntimeConfig),
            new Promise<never>((_resolve, reject) => {
                timer = setTimeout(() => reject(timeout), timeoutMs);
            }),
        ]);

        pending = request.then(config => {
            // 只在 race 成功后提交；超时请求的迟到响应不能写入配置。
            runtimeConfig.value = config;
            startupStatus.value = 'ready';
        }).catch(error => {
            startupStatus.value = 'failed';
            startupFailure.value = error === timeout ? 'timeout' : 'backend';
            // 后端 String 已脱敏；不展示未知异常对象或记录配置内容。
            startupError.value = error === timeout
                ? '等待应用初始化超时，请重试。'
                : typeof error === 'string' && error.trim()
                    ? error
                    : '应用初始化失败。';
        }).finally(() => {
            clearTimeout(timer);
            pending = undefined;
        });
        return pending;
    }

    return { getRuntimeConfig, runtimeConfig, startupStatus, startupError, startupFailure };
}

export function createStartupInitializer(tasks: { name: string, run: () => unknown }[]) {
    let initialized: Promise<void> | undefined;
    return function initialize() {
        // 每项任务仅尝试一次，包括失败项；runtime 重试不重复注册监听。
        if (!initialized) {
            initialized = Promise.all(tasks.map(task => Promise.resolve().then(task.run).catch(() => {
                console.error(`启动初始化失败：${task.name}，请检查配置后重启应用。`);
            }))).then(() => undefined);
        }
        return initialized;
    };
}
