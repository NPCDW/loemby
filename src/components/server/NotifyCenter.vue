<template>
    <el-popover
        :visible="visible"
        :width="400"
        transition="el-zoom-in-bottom"
        placement="top-start"
        popper-class="notify-popover"
    >
        <template #reference>
            <button class="notify__trigger" type="button" @click="emit('toggle')">
                <el-icon :size="15"><i-ep-BellFilled /></el-icon>
                <span>消息</span>
                <span v-if="messages.length" class="notify__count mono">{{ messages.length }}</span>
            </button>
        </template>

        <div class="notify">
            <header class="notify__head">
                <span>消息中心</span>
                <button class="ghost ghost--bare" type="button" title="收起" @click="emit('close')">
                    <el-icon :size="14"><i-ep-ArrowDownBold /></el-icon>
                </button>
            </header>

            <el-scrollbar ref="scrollbarRef" max-height="26rem">
                <p v-if="messages.length === 0" class="notify__empty">
                    暂时没有消息。播放进度与同步结果会出现在这里。
                </p>

                <ul v-else class="notify__list">
                    <li v-for="message in messages" :key="message.id" class="notify__item">
                        <span class="notify__source">
                            <svg-icon v-if="message.username === 'trakt'" name="trakt" size="22" />
                            <svg-icon v-else-if="message.username === 'simkl'" name="simkl" size="22" />
                            <img
                                v-else-if="message.username === 'YamTrack'"
                                src="../../icons/yamtrack.png"
                                alt="YamTrack"
                                style="width: 22px; height: 22px"
                            />
                            <ServerIcon
                                v-else-if="message.username === 'embyServer'"
                                :icon-url="iconUrlOf(message.embyServerId)"
                                :size="22"
                            />
                            <svg-icon v-else name="app-icon" size="22" color="currentColor" />
                        </span>

                        <div class="notify__body">
                            <div class="notify__meta">
                                <span class="notify__from">{{ sourceName(message) }}</span>
                                <span class="num">{{ message.datetime }}</span>
                            </div>
                            <div class="notify__content" :data-level="message.level || 'info'">
                                <component :is="message.content" v-if="isVNode(message.content)" />
                                <template v-else>{{ message.content }}</template>
                            </div>
                        </div>
                    </li>
                </ul>
            </el-scrollbar>
        </div>
    </el-popover>
</template>

<script setup lang="ts">
import { computed, isVNode, nextTick, ref, watch } from 'vue';
import type { ScrollbarInstance } from 'element-plus';
import ServerIcon from './ServerIcon.vue';
import { useNotifyCenter, type NotifyMessage } from '../../store/notifyCenter';
import type { EmbyServer } from '../../store/db/embyServer';
import { useEmbyServers } from '../../composables/useEmbyServer';

/**
 * 消息中心。
 *
 * 只做两件事：把同步/播放/下载的结果按时间列出来，并在新消息到达时自动滚到底。
 * 触发按钮住在状态栏里，所以这里只管弹层内容。
 */
const props = defineProps<{ visible: boolean }>();

const emit = defineEmits<{ toggle: []; close: [] }>();

const scrollbarRef = ref<ScrollbarInstance>();
const messages = computed<NotifyMessage[]>(() => useNotifyCenter().notifyMessages);
const { serverMap } = useEmbyServers();

function iconUrlOf(embyServerId?: string): string {
    if (!embyServerId) {
        return '';
    }
    return (serverMap.value[embyServerId] as EmbyServer | undefined)?.icon_url ?? '';
}

function sourceName(message: NotifyMessage): string {
    if (message.username === 'embyServer' && message.embyServerId) {
        return serverMap.value[message.embyServerId]?.server_name ?? '服务器';
    }
    if (message.username === 'loemby') {
        return 'loemby';
    }
    return message.username;
}

function scrollToBottom() {
    nextTick(() => {
        const wrap = scrollbarRef.value?.wrapRef as HTMLElement | undefined;
        if (props.visible && wrap?.scrollHeight) {
            scrollbarRef.value?.setScrollTop(wrap.scrollHeight);
        }
    });
}

watch(() => messages.value.length, scrollToBottom);
watch(() => props.visible, value => value && scrollToBottom());
</script>

<style scoped>
.notify__trigger {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.1875rem 0.5rem;
    border: 1px solid transparent;
    border-radius: var(--r-xs);
    background: transparent;
    color: var(--text-dim);
    font-size: var(--fs-xs);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.notify__trigger:hover {
    color: var(--text);
    border-color: var(--line);
}

.notify__count {
    color: var(--lamp);
}

.notify__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 0.625rem;
    margin-bottom: 0.375rem;
    border-bottom: 1px solid var(--line);
    font-size: var(--fs-md);
    font-weight: 600;
}

.notify__empty {
    margin: 0;
    padding: 1.75rem 0;
    color: var(--text-faint);
    font-size: var(--fs-sm);
    text-align: center;
}

.notify__list {
    margin: 0;
    padding: 0;
    list-style: none;
}

.notify__item {
    display: flex;
    gap: 0.625rem;
    padding: 0.625rem 0;
    border-bottom: 1px solid var(--line);
}

.notify__item:last-child {
    border-bottom: none;
}

.notify__source {
    flex: none;
    display: flex;
    align-items: flex-start;
    padding-top: 1px;
    color: var(--text-dim);
}

.notify__body {
    flex: auto;
    min-width: 0;
}

.notify__meta {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.625rem;
    font-size: var(--fs-xs);
}

.notify__from {
    color: var(--text-dim);
}

.notify__content {
    margin-top: 0.375rem;
    padding: 0.375rem 0.5625rem;
    border-left: 2px solid var(--line);
    border-radius: 0 var(--r-xs) var(--r-xs) 0;
    background: var(--surface);
    color: var(--text-dim);
    font-size: var(--fs-sm);
    line-height: 1.65;
    word-break: break-all;
}

.notify__content[data-level='success'] {
    border-left-color: var(--ok);
}

.notify__content[data-level='warning'] {
    border-left-color: var(--lamp);
}

.notify__content[data-level='danger'] {
    border-left-color: var(--bad);
}

.notify__content[data-level='primary'] {
    border-left-color: var(--info);
}
</style>
