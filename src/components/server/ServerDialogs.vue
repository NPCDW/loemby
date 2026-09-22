<template>
    <!-- 添加服务器：地址 → 账号 → 完成 -->
    <el-dialog :model-value="props.addStep > 0" title="添加服务器" width="45rem" :close-on-click-modal="false" @update:model-value="(v: boolean) => !v && emit('update:addStep', 0)">
        <el-steps :active="addStep" align-center>
            <el-step title="服务器地址" />
            <el-step title="用户名密码" />
            <el-step title="完成" />
        </el-steps>

        <div class="form">
            <template v-if="addStep === 1">
                <div class="field">
                    <span class="field__label">服务器地址</span>
                    <el-input v-model="draft.base_url" placeholder="http://192.168.1.2:8096" />
                    <span class="field__hint">填 Emby / Jellyfin 的访问地址，末尾不用带 /</span>
                </div>
                <div class="field">
                    <span class="field__label">反代服务器</span>
                    <el-select v-model="draft.reverse_proxy_id" style="width: 100%">
                        <el-option label="不使用反代" value="no" />
                        <el-option v-for="item in reverseProxies" :key="item.id" :label="item.name" :value="item.id!" />
                    </el-select>
                </div>
                <div class="field">
                    <span class="field__label">媒体库浏览代理</span>
                    <el-select v-model="draft.browse_proxy_id" style="width: 100%">
                        <el-option label="不使用代理" value="no" />
                        <el-option :label="`跟随全局（${globalBrowseName}）`" value="follow" />
                        <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                    </el-select>
                </div>
                <div class="field">
                    <span class="field__label">媒体流播放代理</span>
                    <el-select v-model="draft.play_proxy_id" style="width: 100%">
                        <el-option label="不使用代理" value="no" />
                        <el-option :label="`跟随全局（${globalPlayName}）`" value="follow" />
                        <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                    </el-select>
                </div>
            </template>

            <template v-else-if="addStep === 2">
                <div class="field">
                    <span class="field__label">服务器名称</span>
                    <el-input v-model="draft.server_name" placeholder="自定义名称" />
                </div>
                <div class="field">
                    <span class="field__label">用户名</span>
                    <el-input v-model="draft.username" placeholder="Emby 用户名" />
                </div>
                <div class="field">
                    <span class="field__label">密码</span>
                    <el-input v-model="draft.password" type="password" show-password placeholder="Emby 密码" />
                </div>
            </template>

            <div v-else class="done">
                <span class="done__title">服务器已添加</span>
                <span class="done__hint">回到左侧列表即可进入媒体库并开始播放。</span>
            </div>
        </div>

        <template #footer>
            <div class="footer">
                <el-button v-if="addStep === 2" @click="emit('update:addStep', 1)">上一步</el-button>
                <span class="footer__spacer" />
                <el-button v-if="addStep === 1" type="primary" :loading="probing" @click="emit('probe', draft)">
                    下一步
                </el-button>
                <el-button v-else-if="addStep === 2" type="primary" :loading="authenticating" @click="emit('authenticate', draft)">
                    登录并完成
                </el-button>
                <el-button v-else type="primary" @click="emit('update:addStep', 0)">完成</el-button>
            </div>
        </template>
    </el-dialog>

    <!-- 编辑服务器 -->
    <el-dialog :model-value="!!props.editServer" title="编辑服务器" width="45rem" @update:model-value="(v: boolean) => !v && emit('update:editServer', null)">
        <div class="form form--two">
            <div class="field">
                <span class="field__label">服务器地址</span>
                <el-input v-model="editDraft.base_url" />
            </div>
            <div class="field">
                <span class="field__label">服务器名称</span>
                <div class="field__row">
                    <el-input v-model="editDraft.server_name" style="flex: auto" />
                    <el-button :loading="serverInfoLoading" @click="emit('fetchInfo')">从服务器获取</el-button>
                </div>
            </div>
            <div class="field">
                <span class="field__label">用户名</span>
                <el-input v-model="editDraft.username" />
            </div>
            <div class="field">
                <span class="field__label">密码</span>
                <el-input v-model="editDraft.password" type="password" show-password />
            </div>
            <div class="field">
                <span class="field__label">反代服务器</span>
                <el-select v-model="editDraft.reverse_proxy_id" style="width: 100%">
                    <el-option label="不使用反代" value="no" />
                    <el-option v-for="item in reverseProxies" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
            </div>
            <div class="field">
                <span class="field__label">媒体库代理</span>
                <el-select v-model="editDraft.browse_proxy_id" style="width: 100%">
                    <el-option label="不使用代理" value="no" />
                    <el-option :label="`跟随全局（${globalBrowseName}）`" value="follow" />
                    <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
            </div>
            <div class="field">
                <span class="field__label">媒体流代理</span>
                <el-select v-model="editDraft.play_proxy_id" style="width: 100%">
                    <el-option label="不使用代理" value="no" />
                    <el-option :label="`跟随全局（${globalPlayName}）`" value="follow" />
                    <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
            </div>
            <div class="field">
                <span class="field__label">观看提醒</span>
                <el-input-number v-model="editDraft.keep_alive_days" :min="0" :controls="false" style="width: 100%" />
                <span class="field__hint">超过这么多天没有观看，就在左侧服务器列表里提醒一次</span>
            </div>
        </div>
        <template #footer>
            <div class="footer">
                <span class="footer__spacer" />
                <el-button @click="emit('update:editServer', null)">取消</el-button>
                <el-button type="primary" @click="emit('saveEdit', editDraft)">保存</el-button>
            </div>
        </template>
    </el-dialog>

    <!-- 线路配置 -->
    <el-dialog :model-value="!!props.lineServer" title="线路配置" width="45rem" @update:model-value="(v: boolean) => !v && emit('update:lineServer', null)">
        <el-scrollbar max-height="26rem">
            <LineList
                :lines="lines"
                :active-id="lineServer?.line_id"
                @add="emit('addLine')"
                @edit="line => emit('editLine', line)"
                @remove="line => emit('removeLine', line)"
                @select="line => emit('selectLine', line)"
            />
        </el-scrollbar>
        <template #footer>
            <div class="footer">
                <span class="footer__spacer" />
                <el-button type="primary" @click="emit('update:lineServer', null)">完成</el-button>
            </div>
        </template>
    </el-dialog>

    <!-- 线路编辑 -->
    <el-dialog :model-value="!!props.lineDraft" title="线路" width="45rem" @update:model-value="(v: boolean) => !v && emit('update:lineDraft', null)">
        <div class="form">
            <div class="field">
                <span class="field__label">线路名称</span>
                <el-input v-model="lineDraft.name" placeholder="例如 直连 / 中转" />
            </div>
            <div class="field">
                <span class="field__label">线路地址</span>
                <el-input v-model="lineDraft.base_url" placeholder="http://192.168.1.2:8096" />
            </div>
            <div class="field">
                <span class="field__label">反代服务器</span>
                <el-select v-model="lineDraft.reverse_proxy_id" style="width: 100%">
                    <el-option label="不使用反代" value="no" />
                    <el-option v-for="item in reverseProxies" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
            </div>
            <div class="field">
                <span class="field__label">媒体库代理</span>
                <el-select v-model="lineDraft.browse_proxy_id" style="width: 100%">
                    <el-option label="不使用代理" value="no" />
                    <el-option :label="`跟随全局（${globalBrowseName}）`" value="follow" />
                    <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
            </div>
            <div class="field">
                <span class="field__label">媒体流代理</span>
                <el-select v-model="lineDraft.play_proxy_id" style="width: 100%">
                    <el-option label="不使用代理" value="no" />
                    <el-option :label="`跟随全局（${globalPlayName}）`" value="follow" />
                    <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
            </div>
        </div>
        <template #footer>
            <div class="footer">
                <span class="footer__spacer" />
                <el-button @click="emit('update:lineDraft', null)">取消</el-button>
                <el-button type="primary" @click="emit('saveLine', lineDraft)">保存</el-button>
            </div>
        </template>
    </el-dialog>

    <!-- 图标选择 -->
    <el-dialog :model-value="!!props.iconServer" title="选择服务器图标" width="34rem" @update:model-value="(v: boolean) => !v && emit('update:iconServer', null)">
        <div class="iconpick">
            <div class="field__row">
                <el-select
                    :model-value="iconLibraryId"
                    placeholder="图标库"
                    style="width: 12rem"
                    @update:model-value="(v: string) => { emit('update:iconLibraryId', v); emit('switchIconLibrary') }"
                >
                    <el-option v-for="item in iconLibraries" :key="item.id" :label="item.name" :value="item.id!" />
                </el-select>
                <el-input
                    :model-value="iconKeyword"
                    placeholder="搜索图标"
                    style="flex: auto"
                    @update:model-value="(v: string) => emit('update:iconKeyword', v)"
                />
            </div>
            <el-scrollbar v-loading="iconLoading" max-height="18rem">
                <div class="iconpick__grid">
                    <button
                        v-for="icon in visibleIcons"
                        :key="icon.url"
                        class="iconpick__cell"
                        type="button"
                        :title="icon.name"
                        @click="emit('pickIcon', icon.url)"
                    >
                        <img v-lazy="icon.localUrl" :alt="icon.name" />
                        <span class="t-clip">{{ icon.name }}</span>
                    </button>
                </div>
                <p v-if="!iconLoading && visibleIcons.length === 0" class="t-faint iconpick__empty">
                    没有匹配的图标。换个关键词，或先到设置里添加图标库。
                </p>
            </el-scrollbar>
        </div>
    </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import LineList from './LineList.vue';
import type { EmbyServer } from '../../store/db/embyServer';
import type { EmbyLine } from '../../store/db/embyLine';
import type { ProxyServer } from '../../store/db/proxyServer';
import type { ReverseProxyServer } from '../../store/db/reverseProxyServer';
import type { EmbyIconLibrary } from '../../store/db/embyIconLibrary';

/**
 * 服务器相关的全部弹窗。
 *
 * 这些弹窗原先挤在外壳里（1000+ 行），拆出来之后外壳只负责
 * 「谁在什么时候打开哪个弹窗」，表单各自封闭。
 *
 * 草稿策略：props 是不可变对象，这里每个弹窗维护一份本地草稿，
 * 打开时同步一次、提交时把草稿回传给父级，避免直接改 props。
 */
export interface IconChoice {
    name: string;
    url: string;
    localUrl: string;
}

const props = defineProps<{
    addStep: number;
    editServer: EmbyServer | null;
    lineServer: EmbyServer | null;
    lineDraft: EmbyLine | null;
    lines: EmbyLine[];
    proxies: ProxyServer[];
    reverseProxies: ReverseProxyServer[];
    globalBrowseName: string;
    globalPlayName: string;
    serverInfoLoading?: boolean;
    probing?: boolean;
    authenticating?: boolean;
    iconServer: EmbyServer | null;
    iconLibraries: EmbyIconLibrary[];
    iconLibraryId: string;
    iconKeyword: string;
    icons: IconChoice[];
    iconLoading?: boolean;
    /** 添加向导的服务器草稿，由父级持有 */
    serverDraft: EmbyServer;
}>();

const emit = defineEmits<{
    'update:addStep': [value: number];
    'update:editServer': [value: EmbyServer | null];
    'update:lineServer': [value: EmbyServer | null];
    'update:lineDraft': [value: EmbyLine | null];
    'update:iconServer': [value: EmbyServer | null];
    'update:iconLibraryId': [value: string];
    'update:iconKeyword': [value: string];
    probe: [draft: EmbyServer];
    authenticate: [draft: EmbyServer];
    fetchInfo: [];
    saveEdit: [draft: EmbyServer];
    addLine: [];
    editLine: [line: EmbyLine];
    removeLine: [line: EmbyLine];
    selectLine: [line: EmbyLine];
    saveLine: [draft: EmbyLine];
    switchIconLibrary: [];
    pickIcon: [url: string];
}>();

/** 添加向导直接改父级草稿（父子共享同一对象引用，改动即时可见） */
const draft = computed(() => props.serverDraft);

/* 编辑服务器：本地副本 */
const editDraft = ref<EmbyServer>({});
watch(
    () => props.editServer,
    value => {
        editDraft.value = value ? { ...value } : {};
    },
    { immediate: true },
);

/* 线路编辑：本地副本 */
const lineDraft = ref<EmbyLine>({});
watch(
    () => props.lineDraft,
    value => {
        lineDraft.value = value ? { ...value } : {};
    },
    { immediate: true },
);

const visibleIcons = computed(() => {
    const keyword = props.iconKeyword.trim().toLowerCase();
    if (!keyword) {
        return props.icons;
    }
    return props.icons.filter(icon => icon.name.toLowerCase().includes(keyword));
});
</script>

<style scoped>
.form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: min(28rem, 100%);
    margin: 1.25rem auto 0.25rem;
}

.form--two {
    width: min(38rem, 100%);
}

.done {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 2rem 0 2.5rem;
    text-align: center;
}

.done__title {
    font-size: var(--fs-xl);
}

.done__hint {
    color: var(--text-faint);
    font-size: var(--fs-sm);
}

.footer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.footer__spacer {
    flex: auto;
}

.iconpick {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.iconpick__grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
}

.iconpick__cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    width: 4.5rem;
    padding: 0.5rem 0.25rem;
    border: 1px solid transparent;
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-faint);
    font-size: var(--fs-xs);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.iconpick__cell:hover {
    border-color: var(--lamp-edge);
    background: var(--lamp-wash);
    color: var(--text);
}

.iconpick__cell img {
    width: 2.25rem;
    height: 2.25rem;
    object-fit: contain;
}

.iconpick__empty {
    margin: 0;
    padding: 1.5rem 0;
    text-align: center;
}
</style>
