import { ElButton, ElMessage, ElNotification, NotificationHandle } from "element-plus";
import invokeApi, { UpdateInfo } from "../api/invokeApi";
import { defineComponent, h, ref } from "vue";

let notification: NotificationHandle | undefined;
let checking: Promise<UpdateInfo | null> | undefined;
const updating = ref(false);

function showUpdate(update: UpdateInfo) {
    if (notification) {
        return;
    }

    const UpdatePrompt = defineComponent({
        setup() {
            const errorMessage = ref('');
            const installed = ref(false);

            function dismiss() {
                if (!updating.value) {
                    notification?.close();
                }
            }

            function preventClose(event: KeyboardEvent) {
                if (event.key === 'Escape') {
                    event.stopImmediatePropagation();
                }
            }

            async function installUpdate() {
                if (updating.value) {
                    return;
                }
                updating.value = true;
                errorMessage.value = '';
                // ElNotification 的 Escape 关闭不受 showClose 控制，安装期间需阻止关闭。
                document.addEventListener('keydown', preventClose, true);
                try {
                    if (!installed.value) {
                        await invokeApi.installUpdate(update.version);
                        installed.value = true;
                    }
                    await invokeApi.restartApp();
                } catch (err) {
                    errorMessage.value = installed.value
                        ? `更新已安装，重启失败，请重试或手动重启: ${err}`
                        : String(err);
                    ElMessage.error(errorMessage.value);
                } finally {
                    updating.value = false;
                    document.removeEventListener('keydown', preventClose, true);
                }
            }

            return () => h('div', null, [
                h(ElButton, {
                    text: true,
                    disabled: updating.value,
                    'aria-label': '关闭更新提示',
                    style: 'position: absolute; top: 12px; right: 12px; width: 20px; height: 20px; padding: 0;',
                    onClick: dismiss,
                }, () => '×'),
                h('div', null, `新版本 ${update.version} 已发布`),
                h('div', {style: 'margin-top: 6px;'}, updating.value
                    ? (installed.value ? '正在重启应用…' : '正在下载并安装更新，请稍候…')
                    : '更新后将重启应用'),
                errorMessage.value ? h('div', {
                    style: 'margin-top: 6px; color: var(--el-color-danger); overflow-wrap: anywhere;',
                    role: 'alert',
                }, errorMessage.value) : null,
                h('div', {style: 'display: flex; gap: 8px; margin-top: 12px;'}, [
                    h(ElButton, {
                        size: 'small',
                        type: 'primary',
                        loading: updating.value,
                        disabled: updating.value,
                        onClick: installUpdate,
                    }, () => installed.value ? '立即重启' : '立即更新并重启'),
                    h(ElButton, {
                        size: 'small',
                        disabled: updating.value,
                        style: 'margin-left: 0;',
                        onClick: dismiss,
                    }, () => '下次再说'),
                ]),
            ]);
        },
    });

    notification = ElNotification({
        title: '发现新版本',
        message: h(UpdatePrompt),
        position: 'bottom-right',
        duration: 0,
        showClose: false,
        onClose: () => {
            notification = undefined;
        },
    });
}

async function checkUpdate(manual: boolean = false) {
    if (updating.value) {
        if (manual) {
            ElMessage.info('正在更新，请稍候');
        }
        return;
    }

    try {
        if (!checking) {
            checking = invokeApi.checkUpdate().finally(() => {
                checking = undefined;
            });
        }
        const update = await checking;
        if (update) {
            showUpdate(update);
        } else if (manual) {
            ElMessage.success('已经是最新版本');
        }
    } catch (err) {
        if (manual) {
            ElMessage.error(String(err));
        } else {
            console.error('自动检查更新失败', err);
        }
    }
}

export default {
    checkUpdate,
}
