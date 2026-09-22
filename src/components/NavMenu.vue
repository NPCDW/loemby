<template>
    <div class="shell">
        <div class="shell__body">
            <!-- 导航栏：单列，服务器是一等公民，可拖拽排序 -->
            <aside class="rail">
                <div class="rail__brand" @click="jumpRoute('/nav/history')">
                    <span class="rail__mark">
                        <svg-icon name="app-icon" size="22" color="#F2A13B" />
                    </span>
                    <span class="rail__name">loemby</span>
                    <span class="rail__version">v{{ appVersion }}</span>
                </div>

                <nav class="rail__group">
                    <button
                        v-for="entry in railEntries"
                        :key="entry.path"
                        class="rail__item"
                        :class="{ 'is-active': active === entry.path }"
                        @click="jumpRoute(entry.path)"
                    >
                        <el-icon :size="18"><component :is="entry.icon" /></el-icon>
                        <span>{{ entry.label }}</span>
                    </button>
                </nav>

                <div class="rail__label">
                    <span>服务器</span>
                    <button class="rail__add" title="添加服务器" @click="addEmbyServer()">
                        <el-icon :size="14"><i-ep-Plus /></el-icon>
                    </button>
                </div>

                <el-scrollbar class="rail__servers">
                    <Container @drop="onDrop" style="height: 100%; width: 100%;">
                        <Draggable v-for="embyServer in embyServers" :key="embyServer.id" style="width: 100%;">
                            <el-dropdown trigger="contextmenu" style="width: 100%;">
                                <button
                                    class="rail__item rail__item--server"
                                    :class="{ 'is-active': active === '/nav/emby/' + embyServer.id, 'is-off': embyServer.disabled }"
                                    @click="jumpRoute('/nav/emby/' + embyServer.id)"
                                >
                                    <span class="rail__dot" :data-state="serverState(embyServer)"></span>
                                    <span class="rail__icon">
                                        <img v-if="embyServer.icon_url" v-lazy="embyIconLocalUrl[embyServer.id!]">
                                        <svg-icon v-else name="emby" size="16" color="currentColor" />
                                    </span>
                                    <span class="rail__server-name">{{ embyServer.server_name }}</span>
                                    <span
                                        v-if="embyServer.keep_alive_days"
                                        class="rail__keepalive"
                                        :data-level="keep_alive_days[embyServer.id!] > 7 ? 'ok' : keep_alive_days[embyServer.id!] > 3 ? 'soon' : 'over'"
                                        :title="'距离上次观看 ' + keep_alive_days[embyServer.id!] + ' 天'"
                                    >{{ keep_alive_days[embyServer.id!] }}</span>
                                </button>
                                <template #dropdown>
                                    <el-dropdown-menu>
                                        <el-dropdown-item @click="configLine(embyServer)">
                                            <i-ep-Link style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">线路</span>
                                        </el-dropdown-item>
                                        <el-dropdown-item @click="editEmbyIcon(embyServer)">
                                            <i-ep-PriceTag style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">图标</span>
                                        </el-dropdown-item>
                                        <el-dropdown-item @click="editEmbyServer(embyServer)">
                                            <i-ep-Edit style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">编辑</span>
                                        </el-dropdown-item>
                                        <el-dropdown-item @click="enabledEmbyServer(embyServer)">
                                            <template v-if="embyServer.disabled">
                                                <i-ep-CircleCheckFilled style="position: absolute; left: 10;" />
                                                <span style="margin-left: 15px;">启用</span>
                                            </template>
                                            <template v-else>
                                                <i-ep-CircleCloseFilled style="position: absolute; left: 10;" />
                                                <span style="margin-left: 15px;">禁用</span>
                                            </template>
                                        </el-dropdown-item>
                                        <el-dropdown-item divided @click="reLogin(embyServer)">
                                            <i-ep-Promotion style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">重新登录</span>
                                        </el-dropdown-item>
                                        <el-dropdown-item style="color: #D9973F" @click="logoutEmbyServer(embyServer)">
                                            <i-ep-WarnTriangleFilled style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">退出登录</span>
                                        </el-dropdown-item>
                                        <el-dropdown-item style="color: #CF5B54" @click="delEmbyServer(embyServer)">
                                            <i-ep-Delete style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">删除</span>
                                        </el-dropdown-item>
                                    </el-dropdown-menu>
                                </template>
                            </el-dropdown>
                        </Draggable>
                    </Container>
                    <div v-if="embyServers.length === 0" class="rail__hint">
                        还没有服务器，点右上角 + 添加
                    </div>
                </el-scrollbar>
            </aside>

            <!-- 内容区：顶栏负责搜索与线路，页面自己负责排版 -->
            <main class="board">
                <header class="topbar">
                    <div class="topbar__left">
                        <div class="topbar__title">{{ currentTitle }}</div>
                        <div v-if="showEmbyServer.server_name" class="topbar__sub">
                            <span class="topbar__server">{{ showEmbyServer.server_name }}</span>
                            <span class="topbar__rule"></span>
                            <span class="topbar__status" :data-state="serverState(showEmbyServer)">
                                {{ showEmbyServer.disabled ? '已停用' : '在线' }}
                            </span>
                        </div>
                    </div>

                    <div class="topbar__tools" v-if="$route.path.startsWith('/nav/emby/')">
                        <el-select v-model="showEmbyServer.line_id" @change="configLineChange" size="small" class="topbar__select topbar__select--line">
                            <template #prefix><span class="topbar__key">线路</span></template>
                            <el-option v-for='line in embyLines[showEmbyServer.id!]' :key="line.id" :label="line.name" :value="line.id"/>
                            <template #footer>
                                <el-button size="small" @click="configLine(showEmbyServer)">配置线路</el-button>
                            </template>
                        </el-select>
                        <el-select v-model="showServerLine.reverse_proxy_id" @change="proxyChange(showServerLine)" size="small" class="topbar__select">
                            <template #prefix><span class="topbar__key">反代</span></template>
                            <el-option key="no" label="不使用反代" value="no"/>
                            <el-option v-for="reverseProxyServer in reverseProxyServers" :key="reverseProxyServer.id" :label="reverseProxyServer.name" :value="reverseProxyServer.id"/>
                        </el-select>
                        <el-select v-model="showServerLine.browse_proxy_id" @change="proxyChange(showServerLine)" size="small" class="topbar__select">
                            <template #prefix><span class="topbar__key">浏览</span></template>
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                        <el-select v-model="showServerLine.play_proxy_id" @change="proxyChange(showServerLine)" size="small" class="topbar__select">
                            <template #prefix><span class="topbar__key">播放</span></template>
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </div>
                </header>

                <el-scrollbar class="board__scroll">
                    <router-view v-slot="{ Component }">
                        <keep-alive>
                            <component :is="Component" :key="$route.fullPath" v-if="$route.meta.keepAlive" />
                        </keep-alive>
                        <component :is="Component" :key="$route.fullPath" v-if="!$route.meta.keepAlive" />
                    </router-view>
                </el-scrollbar>
            </main>
        </div>

        <!-- 状态栏：消息与全局状态，保持一行 -->
        <footer class="status">
            <div class="status__left">
                <button class="status__notify" @click="() => {dialogNotifyCenterVisible = !dialogNotifyCenterVisible;notifyScrollBottom()}">
                    <el-icon :size="15"><i-ep-BellFilled /></el-icon>
                    <span>消息</span>
                    <span v-if="notifyMessages && notifyMessages.length" class="status__badge">{{ notifyMessages.length }}</span>
                </button>
                <span class="status__meta">{{ embyServers.length }} 台服务器</span>
            </div>
            <div class="status__right">
                <span class="status__meta">loemby v{{ appVersion }}</span>
            </div>
        </footer>

        <el-popover
            :visible="dialogNotifyCenterVisible"
            :width="420"
            transition="el-zoom-in-bottom"
            placement="top-start"
            popper-class="roe-notify"
        >
            <template #reference>
                <span class="status__anchor"></span>
            </template>
            <div class="notify">
                <div class="notify__head">
                    <span>消息中心</span>
                    <button class="notify__close" @click="dialogNotifyCenterVisible = false">
                        <el-icon><i-ep-ArrowDownBold /></el-icon>
                    </button>
                </div>
                <el-scrollbar max-height="440px" ref="notifyScrollbarRef">
                    <div v-if="!notifyMessages || notifyMessages.length <= 0" class="notify__empty">
                        暂无消息，播放与同步结果会出现在这里。
                    </div>
                    <template v-else>
                        <transition-group name="el-zoom-in-left">
                            <div v-for="message in notifyMessages" :key="message.id" class="notify__item">
                                <span class="notify__source">
                                    <svg-icon v-if="message.username == 'trakt'" name="trakt" size="24" />
                                    <svg-icon v-else-if="message.username == 'simkl'" name="simkl" size="24" />
                                    <img style="width: 24px; height: 24px;" v-else-if="message.username == 'YamTrack'" src="../icons/yamtrack.png" />
                                    <template v-else-if="message.username == 'embyServer'">
                                        <img v-if="message.embyServerId && embyServerMap[message.embyServerId] && embyServerMap[message.embyServerId].icon_url" v-lazy="embyIconLocalUrl[embyServerMap[message.embyServerId].id!]" style="max-width: 24px; max-height: 24px;">
                                        <svg-icon v-else name="emby" size="24" color="currentColor" />
                                    </template>
                                    <svg-icon v-else name="app-icon" size="24" color="currentColor" />
                                </span>
                                <div class="notify__body">
                                    <div class="notify__meta">
                                        <span class="notify__from">
                                            <template v-if="message.username == 'embyServer' && message.embyServerId && embyServerMap[message.embyServerId]">
                                                {{ embyServerMap[message.embyServerId].server_name }}
                                            </template>
                                            <template v-else>{{ message.username }}</template>
                                        </span>
                                        <span class="notify__time">{{ message.datetime }}</span>
                                    </div>
                                    <div class="notify__content" :data-level="message.level || 'info'">
                                        <component v-if="isVNode(message.content)" :is="message.content"></component>
                                        <template v-else>{{ message.content }}</template>
                                    </div>
                                </div>
                            </div>
                        </transition-group>
                    </template>
                </el-scrollbar>
            </div>
        </el-popover>

        <el-dialog v-model="dialogAddEmbyServerVisible" title="添加服务器" width="720">
            <el-steps :active="stepActive" align-center>
                <el-step title="服务器地址" />
                <el-step title="用户名密码" />
                <el-step title="完成" />
            </el-steps>
            <div v-if="stepActive == 1" class="dlg-form">
                <el-form label-position="top">
                    <el-form-item label="服务器地址">
                        <el-input v-model="dialogEmbyServer.base_url" placeholder="例如 http://192.168.1.2:8096" />
                    </el-form-item>
                    <el-form-item label="反代服务器">
                        <el-select v-model="dialogEmbyServer.reverse_proxy_id" @change="reverseProxyChange(dialogEmbyServer)">
                            <el-option key="no" label="不使用反代" value="no"/>
                            <el-option v-for="reverseProxyServer in reverseProxyServers" :key="reverseProxyServer.id" :label="reverseProxyServer.name" :value="reverseProxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="媒体库浏览代理">
                        <el-select v-model="dialogEmbyServer.browse_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="媒体流播放代理">
                        <el-select v-model="dialogEmbyServer.play_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <div class="dlg-actions">
                        <el-button :loading="addEmbyServerAddrLoading" @click="addEmbyServerAddr" type="primary">下一步</el-button>
                    </div>
                </el-form>
            </div>
            <div v-if="stepActive == 2" class="dlg-form">
                <el-form label-position="top">
                    <el-form-item label="服务器名称">
                        <el-input v-model="dialogEmbyServer.server_name" placeholder="自定义名称" />
                    </el-form-item>
                    <el-form-item label="用户名">
                        <el-input v-model="dialogEmbyServer.username" placeholder="Emby 用户名" />
                    </el-form-item>
                    <el-form-item label="密码">
                        <el-input v-model="dialogEmbyServer.password" placeholder="Emby 密码" show-password />
                    </el-form-item>
                    <div class="dlg-actions dlg-actions--split">
                        <el-button :loading="addEmbyServerAuthLoading" @click="addEmbyServerPrevStep">上一步</el-button>
                        <el-button :loading="addEmbyServerAuthLoading" @click="addEmbyServerAuth" type="primary">下一步</el-button>
                    </div>
                </el-form>
            </div>
            <div v-if="stepActive == 3" class="dlg-done">
                <div class="dlg-done__title">服务器已添加</div>
                <div class="dlg-done__text">现在可以从左侧列表进入媒体库并开始播放。</div>
                <el-button type="primary" @click="dialogAddEmbyServerVisible = false">完成</el-button>
            </div>
        </el-dialog>

        <el-dialog v-model="dialogEditEmbyServerVisible" title="编辑服务器" width="720">
            <div class="dlg-form">
                <el-form label-position="top">
                    <el-form-item label="服务器地址">
                        <el-input v-model="dialogEmbyServer.base_url" placeholder="例如 http://192.168.1.2:8096" />
                    </el-form-item>
                    <el-form-item label="服务器名称">
                        <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                            <el-input v-model="dialogEmbyServer.server_name" placeholder="自定义名称" />
                            <el-button :loading="serverInfoLoading" @click="getServerInfo(dialogEmbyServer)" style="margin-left: 8px;">从服务器获取</el-button>
                        </div>
                    </el-form-item>
                    <el-form-item label="用户名">
                        <el-input v-model="dialogEmbyServer.username" placeholder="Emby 用户名" />
                    </el-form-item>
                    <el-form-item label="密码">
                        <el-input v-model="dialogEmbyServer.password" placeholder="Emby 密码" show-password />
                    </el-form-item>
                    <el-form-item label="反代服务器">
                        <el-select v-model="dialogEmbyServer.reverse_proxy_id" @change="reverseProxyChange(dialogEmbyServer)">
                            <el-option key="no" label="不使用反代" value="no"/>
                            <el-option v-for="reverseProxyServer in reverseProxyServers" :key="reverseProxyServer.id" :label="reverseProxyServer.name" :value="reverseProxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="媒体库代理">
                        <el-select v-model="dialogEmbyServer.browse_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="媒体流代理">
                        <el-select v-model="dialogEmbyServer.play_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="观看提醒（超过 N 天未观看会在左侧列表提醒）">
                        <el-input-number v-model="dialogEmbyServer.keep_alive_days" />
                    </el-form-item>
                    <div class="dlg-actions dlg-actions--split">
                        <span></span>
                        <span>
                            <el-button type="primary" @click="saveEditEmbyServer">保存</el-button>
                            <el-button @click="dialogEditEmbyServerVisible = false">取消</el-button>
                        </span>
                    </div>
                </el-form>
            </div>
        </el-dialog>

        <el-dialog v-model="dialogConfigLineVisible" title="线路配置" width="720">
            <el-scrollbar style="height: 420px; padding-right: 6px;">
                <button class="ghost-add" @click="addLine">
                    <el-icon><i-ep-Plus /></el-icon>
                    <span>添加线路</span>
                </button>
                <el-radio-group v-model="dialogEmbyServer.line_id" @change="configLineChange" class="line-list">
                    <el-radio v-for="line in dialogEmbyServerLines" :value="line.id" size="large" border class="line-item">
                        <div class="line-item__body">
                            <div class="line-item__head">
                                <span class="line-item__name">{{ line.name }}</span>
                                <span class="line-item__ops">
                                    <el-button type="primary" text size="small" @click="editLine(line)"><i-ep-Edit /></el-button>
                                    <el-button type="danger" :disabled="line.id === dialogEmbyServer.line_id ? true : false" text size="small" @click="delLine(line)"><i-ep-Delete /></el-button>
                                </span>
                            </div>
                            <span class="line-item__url">{{ line.base_url }}</span>
                        </div>
                    </el-radio>
                </el-radio-group>
            </el-scrollbar>
        </el-dialog>

        <el-dialog v-model="dialogAddLineVisible" title="线路" width="720">
            <div class="dlg-form">
                <el-form label-position="top">
                    <el-form-item label="线路名称">
                        <el-input v-model="dialogEmbyServerAddLine.name" placeholder="例如 直连 / 中转" />
                    </el-form-item>
                    <el-form-item label="线路地址">
                        <el-input v-model="dialogEmbyServerAddLine.base_url" placeholder="例如 http://192.168.1.2:8096" />
                    </el-form-item>
                    <el-form-item label="反代服务器">
                        <el-select v-model="dialogEmbyServerAddLine.reverse_proxy_id" @change="reverseProxyChange(dialogEmbyServerAddLine)">
                            <el-option key="no" label="不使用反代" value="no"/>
                            <el-option v-for="reverseProxyServer in reverseProxyServers" :key="reverseProxyServer.id" :label="reverseProxyServer.name" :value="reverseProxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="媒体库代理">
                        <el-select v-model="dialogEmbyServerAddLine.browse_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="媒体流代理">
                        <el-select v-model="dialogEmbyServerAddLine.play_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <div class="dlg-actions dlg-actions--split">
                        <span></span>
                        <span>
                            <el-button type="primary" @click="savedialogEmbyServerAddLine">保存</el-button>
                            <el-button @click="dialogAddLineVisible = false">取消</el-button>
                        </span>
                    </div>
                </el-form>
            </div>
        </el-dialog>

        <el-dialog v-model="dialogEditEmbyIconVisible" title="选择图标" width="520">
            <div class="iconpick">
                <div class="iconpick__tools">
                    <el-select v-model="selectedEmbyIconLibrary" @change="embyIconLibraryChange" placeholder="图标库">
                        <el-option v-for="item in embyIconLibrary" :key="item.id" :label="item.name" :value="item.id" />
                    </el-select>
                    <el-input v-model="searchEmbyIconName" placeholder="搜索图标" />
                </div>
                <el-scrollbar style="height: 300px;">
                    <div class="iconpick__grid" v-loading="embyIconListLoading">
                        <button
                            v-for="embyIcon in embyIconList"
                            :key="embyIcon.url"
                            v-show="embyIcon.name.toLowerCase().includes(searchEmbyIconName.toLowerCase())"
                            class="iconpick__cell"
                            @click="updateEmbyIcon(embyIcon.url)"
                        >
                            <img v-lazy="embyIcon.local_url" />
                            <span>{{ embyIcon.name }}</span>
                        </button>
                    </div>
                </el-scrollbar>
            </div>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, watchEffect, isVNode, nextTick } from "vue";
import { useRoute, useRouter } from 'vue-router'
import embyApi from '../api/embyApi'
import { ElMessage, ElMessageBox, ScrollbarInstance } from "element-plus";
import { generateGuid } from "../util/uuid_util";
import _ from "lodash";
import { Container, Draggable } from "vue3-smooth-dnd";
import invokeApi from "../api/invokeApi";
import { ProxyServer, useProxyServer } from "../store/db/proxyServer";
import { ReverseProxyServer, useReverseProxyServer } from "../store/db/reverseProxyServer";
import { EmbyServer, useEmbyServer } from "../store/db/embyServer";
import { EmbyLine, useEmbyLine } from "../store/db/embyLine";
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'
import { useEventBus } from "../store/eventBus";
import { EmbyIconLibrary, useEmbyIconLibrary } from "../store/db/embyIconLibrary";
import appApi from "../api/appApi";
import { useGlobalConfig } from "../store/db/globalConfig";
import { useImage } from "../store/image";
import { useNotifyCenter } from "../store/notifyCenter";

const appVersion = import.meta.env.VITE_APP_VERSION || ''

const railEntries = [
    { path: '/nav/history', label: '播放历史', icon: 'i-ep-Clock' },
    { path: '/nav/search', label: '聚合搜索', icon: 'i-ep-Search' },
    { path: '/nav/setting', label: '设置', icon: 'i-ep-Setting' },
]

const pageTitles: {[key: string]: string} = {
    '/nav/history': '播放历史',
    '/nav/search': '聚合搜索',
    '/nav/setting': '设置',
}

const active = ref("/nav/search");
const route = useRoute();
const router = useRouter()

const currentTitle = computed(() => {
    const path = route.path
    if (pageTitles[path]) {
        return pageTitles[path]
    }
    if (path.includes('/mediaLibrary')) return '媒体库'
    if (path.includes('/episodes/')) return '播放详情'
    if (path.includes('/series/')) return '剧集'
    if (path.includes('/search')) return '搜索'
    return 'loemby'
})

watchEffect(() => {
    active.value = route.path;
})

/** 三种状态用一枚圆点表达，避免在列表里堆标签 */
function serverState(embyServer: EmbyServer): string {
    if (embyServer.disabled) return 'off'
    if (!embyServer.auth_token) return 'unsigned'
    return 'on'
}

const proxyServers = ref<ProxyServer[]>([]);
function listAllProxyServer() {
    useProxyServer().listAllProxyServer().then(list => {
        proxyServers.value = list;
    })
}
listAllProxyServer()
onMounted(() => useEventBus().on('ProxyServerChanged', listAllProxyServer))
onUnmounted(() => useEventBus().remove('ProxyServerChanged', listAllProxyServer))

const reverseProxyServers = ref<ReverseProxyServer[]>([]);
function listAllReverseProxyServer() {
    useReverseProxyServer().listAllReverseProxyServer().then(list => {
        reverseProxyServers.value = list;
    })
}
listAllReverseProxyServer()
onMounted(() => useEventBus().on('ReverseProxyServerChanged', listAllReverseProxyServer))
onUnmounted(() => useEventBus().remove('ReverseProxyServerChanged', listAllReverseProxyServer))

const embyServers = ref<EmbyServer[]>([])
const embyServerMap = ref<{[key: string]: EmbyServer}>({})
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
        for (const emby of embyServers.value) {
            getEmbyIconLocalUrl(emby.id!, emby.icon_url)
            embyServerMap.value[emby.id!] = emby
        }
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
listAllEmbyServer()
async function embyServerChanged({event, id}: {event: string, id?: string}) {
    if (id && event == 'update') {
        const index = embyServers.value.findIndex(emby => emby.id === id)
        if (index !== -1) {
            embyServers.value[index] = await useEmbyServer().getEmbyServer(id)
            getEmbyIconLocalUrl(embyServers.value[index].id!, embyServers.value[index].icon_url)
        }
    } else {
        listAllEmbyServer()
    }
}
onMounted(() => useEventBus().on('EmbyServerChanged', embyServerChanged))
onUnmounted(() => useEventBus().remove('EmbyServerChanged', embyServerChanged))

const embyIconLocalUrl = ref<{[key: string]: string}>({})
function getEmbyIconLocalUrl(emby_server_id: string, icon_url?: string) {
    if (icon_url) {
        useImage().loadIcon(icon_url!).then(local_url => embyIconLocalUrl.value[emby_server_id] = local_url)
    }
}

const keep_alive_days = computed(() => {
    let days: {[key: string]: number} = {}
    for (let embyServer of embyServers.value) {
        if (embyServer.keep_alive_days) {
            days[embyServer.id!] = embyServer.keep_alive_days - dayjs().locale('zh-cn').diff(embyServer.last_playback_time, 'day')
        }
    }
    return days
})

const embyLines = ref<{[key: string]: EmbyLine[]}>({});
function listAllEmbyLine() {
    useEmbyLine().listAllEmbyLine().then(list => {
        embyLines.value = {}
        for (let line of list) {
            if (!embyLines.value[line.emby_server_id!]) {
                embyLines.value[line.emby_server_id!] = []
            }
            embyLines.value[line.emby_server_id!].push(line)
        }
        dialogEmbyServerLines.value = embyLines.value[dialogEmbyServer.value.id!]
    })
}
listAllEmbyLine()
onMounted(() => useEventBus().on('EmbyLineChanged', listAllEmbyLine))
onUnmounted(() => useEventBus().remove('EmbyLineChanged', listAllEmbyLine))

async function addEmbyServerDb(tmp: EmbyServer) {
    return useEmbyServer().addEmbyServer(tmp).then(() => {
        useEventBus().emit('EmbyServerChanged', {event: 'add', id: tmp.id})
    }).catch(e => ElMessage.error('添加Emby服务器失败' + e))
}

async function updateEmbyServerDb(tmp: EmbyServer) {
    return useEmbyServer().updateEmbyServer(tmp).then(() => {
        useEventBus().emit('EmbyServerChanged', {event: 'update', id: tmp.id})
    }).catch(e => ElMessage.error('更新Emby服务器失败' + e))
}

async function addEmbyLineDb(line: EmbyLine) {
    return useEmbyLine().addEmbyLine(line).then(() => {
        useEventBus().emit('EmbyLineChanged', {})
    }).catch(e => ElMessage.error('添加Emby线路失败' + e))
}

async function updateEmbyLineDb(line: EmbyLine) {
    return useEmbyLine().updateEmbyLine(line).then(() => {
        useEventBus().emit('EmbyLineChanged', {})
    }).catch(e => ElMessage.error('更新Emby线路失败' + e))
}

async function updateEmbyLineServerName(embyId: string, embyName: string) {
    return useEmbyLine().updateEmbyServerName(embyId, embyName).then(() => {
        useEventBus().emit('EmbyLineChanged', {})
    }).catch(e => ElMessage.error('更新Emby线路服务器名称失败' + e))
}

const stepActive = ref(1)
const dialogAddEmbyServerVisible = ref(false)
const dialogEmbyServer = ref<EmbyServer>({})
function addEmbyServer() {
    stepActive.value = 1;
    dialogAddEmbyServerVisible.value = true
    invokeApi.getSysInfo().then(hostname => {
        const client = "loemby";
        const client_version = import.meta.env.VITE_APP_VERSION;
        const user_agent = client + "/" + client_version;
        useEmbyServer().deferOrder().then(() => {
            dialogEmbyServer.value = {
                id: generateGuid(),
                server_name: '未完成',
                disabled: 1,
                keep_alive_days: 0,
                user_agent: user_agent,
                client: client,
                client_version: client_version,
                device: hostname,
                device_id: hostname,
                order_by: 1,
                reverse_proxy_id: 'no',
                browse_proxy_id: 'follow',
                play_proxy_id: 'follow',
            }
        }).catch(e => ElMessage.error('获取最大排序失败' + e))
    }).catch(e => {
        ElMessage.error({
            message: '获取主机名失败' + e
        })
    })
}
const dialogEditEmbyServerVisible = ref(false)
function editEmbyServer(embyServer: EmbyServer) {
    dialogEditEmbyServerVisible.value = true
    dialogEmbyServer.value = _.clone(embyServer)
}
async function enabledEmbyServer(embyServer: EmbyServer) {
    if (!embyServer.auth_token && embyServer.disabled) {
        ElMessage.error('请先登录')
        return
    }
    embyServer.disabled = 1 - embyServer.disabled!
    await updateEmbyServerDb({id: embyServer.id, disabled: embyServer.disabled})
}
function logoutEmbyServer(embyServer: EmbyServer) {
  ElMessageBox.confirm(
    `确认退出登录服务器「${embyServer.server_name}」吗`,
    'Warning',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
        await embyApi.logout(embyServer.id!)
        embyServer.disabled = 1
        embyServer.auth_token = ''
        await updateEmbyServerDb({id: embyServer.id, auth_token: '', disabled: embyServer.disabled})
    })
}
function delEmbyServer(tmp: EmbyServer) {
  ElMessageBox.confirm(
    `确认删除服务器「${tmp.server_name}」吗`,
    'Warning',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
        useEmbyServer().delEmbyServer(tmp.id!).then(() => {
            useEventBus().emit('EmbyServerChanged', {event: 'del', id: tmp.id})
            ElMessage.success('删除成功')
            useEmbyLine().delEmbyServer(tmp.id!).catch(e => {
                useEventBus().emit('EmbyLineChanged', {})
                ElMessage.error('删除Emby线路失败' + e)
            })
            // 如果当前页面 URL 包含被删除的服务器 ID，重定向到根路径
            if (route.path.includes(tmp.id!)) {
                router.push('/')
            }
        }).catch(e => {
            ElMessage.error('删除Emby服务器失败' + e)
        })
    })
}

const addEmbyServerAddrLoading = ref(false)
async function addEmbyServerAddr() {
    addEmbyServerAddrLoading.value = true
    if (!dialogEmbyServer || !dialogEmbyServer.value?.base_url) {
        addEmbyServerAddrLoading.value = false
        return
    }
    let embyServer = await useEmbyServer().getEmbyServer(dialogEmbyServer.value.id!)
    if (!embyServer) {
        let line = {
            id: generateGuid(),
            name: '线路一',
            base_url: dialogEmbyServer.value!.base_url,
            emby_server_id: dialogEmbyServer.value.id!,
            emby_server_name: dialogEmbyServer.value.server_name,
            in_use: 1,
            reverse_proxy_id: dialogEmbyServer.value!.reverse_proxy_id,
            browse_proxy_id: dialogEmbyServer.value!.browse_proxy_id,
            play_proxy_id: dialogEmbyServer.value!.play_proxy_id
        }
        await addEmbyLineDb(line)
        dialogEmbyServer.value.line_id = line.id
        await addEmbyServerDb(dialogEmbyServer.value);
    }
    embyApi.getServerInfo(dialogEmbyServer.value!.id!).then(async response => {
        let json: {ServerName: string, Id: string} = JSON.parse(response);
        dialogEmbyServer.value!.server_name = json['ServerName']
        dialogEmbyServer.value!.server_id = json['Id']
        await updateEmbyServerDb(dialogEmbyServer.value);
        updateEmbyLineServerName(dialogEmbyServer.value!.id!, dialogEmbyServer.value!.server_name!);
        stepActive.value = stepActive.value + 1;
    }).catch(e => ElMessage.error(e)).finally(() => addEmbyServerAddrLoading.value = false)
}
const serverInfoLoading = ref(false)
function getServerInfo(embyServer: EmbyServer) {
    serverInfoLoading.value = true
    embyApi.getServerInfo(embyServer.id!).then(async response => {
        let json: {ServerName: string, Id: string} = JSON.parse(response);
        embyServer.server_name = json['ServerName']
        embyServer.server_id = json['Id']
    }).catch(e => ElMessage.error(e)).finally(() => serverInfoLoading.value = false)
}
const addEmbyServerAuthLoading = ref(false)
function addEmbyServerPrevStep() {
    addEmbyServerAuthLoading.value = true
    stepActive.value = stepActive.value - 1;
    addEmbyServerAuthLoading.value = false
}
async function addEmbyServerAuth() {
    if (!dialogEmbyServer || !dialogEmbyServer.value?.username) {
        ElMessage.error('请至少填写用户名')
        return
    }
    updateEmbyLineServerName(dialogEmbyServer.value.id!, dialogEmbyServer.value.server_name!);
    addEmbyServerAuthLoading.value = true
    login(dialogEmbyServer.value).then(() => {
        stepActive.value = stepActive.value + 1;
    }).catch(e => ElMessage.error(e)).finally(() => addEmbyServerAuthLoading.value = false)
}
async function reLogin(embyServerConfig: EmbyServer) {
  ElMessageBox.confirm(
    `确认重新登录服务器「${embyServerConfig.server_name}」吗`,
    'Warning',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    login(embyServerConfig)
    })
}
async function login(embyServerConfig: EmbyServer) {
    await updateEmbyServerDb(embyServerConfig);
    return embyApi.authenticateByName(embyServerConfig.id!).then(async response => {
        let json: {User: {Id: string}, AccessToken: string} = JSON.parse(response);
        embyServerConfig.auth_token = json['AccessToken']
        embyServerConfig.user_id = json["User"]['Id']
        embyServerConfig.disabled = 0
        await updateEmbyServerDb(embyServerConfig);
        ElMessage.success({
            message: "登录成功"
        })
    }).catch(e => ElMessage.error('登录失败 ' + e))
}
async function saveEditEmbyServer() {
    await useEmbyLine().getEmbyLine(dialogEmbyServer.value!.line_id!).then(async line => {
        if (!line) {
            ElMessage.error('获取正在使用的线路失败')
            return
        }
        line.base_url = dialogEmbyServer.value!.base_url
        line.reverse_proxy_id = dialogEmbyServer.value!.reverse_proxy_id
        line.browse_proxy_id = dialogEmbyServer.value!.browse_proxy_id
        line.play_proxy_id = dialogEmbyServer.value!.play_proxy_id
        await updateEmbyLineDb(line)
        if (line.emby_server_name != dialogEmbyServer.value!.server_name) {
            await updateEmbyLineServerName(dialogEmbyServer.value!.id!, dialogEmbyServer.value!.server_name!);
        }
        if (line.id === showServerLine.value.id) {
            showServerLine.value = line
        }
    }).catch(e => ElMessage.error('获取正在使用的线路失败' + e))
    await updateEmbyServerDb(dialogEmbyServer.value!);
    ElMessage.success({
        message: "保存成功"
    })
    dialogEditEmbyServerVisible.value = false
}
async function jumpRoute(route: string) {
    router.push(route)
}

async function onDrop({removedIndex, addedIndex}: {removedIndex: number, addedIndex: number}) {
    useEmbyServer().updateOrder(embyServers.value[removedIndex].id!, embyServers.value[removedIndex].order_by!, embyServers.value[addedIndex].order_by!).then(() => {
        useEventBus().emit('EmbyServerChanged', {event: 'order'})
    }).catch(e => ElMessage.error("排序失败" + e))
    // 页面操作，防止刷新Emby列表闪烁
    let element = embyServers.value.splice(removedIndex, 1);
    embyServers.value.splice(addedIndex, 0, element[0]);
}

const dialogConfigLineVisible = ref(false)
const dialogEmbyServerLines = ref<EmbyLine[]>([])
function configLine(embyServer: EmbyServer) {
    dialogEmbyServer.value = _.clone(embyServer)
    dialogEmbyServerLines.value = embyLines.value[embyServer.id!]
    dialogConfigLineVisible.value = true
}
const dialogAddLineVisible = ref(false)
const dialogEmbyServerAddLine = ref<EmbyLine>({})
function addLine() {
    dialogEmbyServerAddLine.value = {
        emby_server_id: dialogEmbyServer.value.id!,
        emby_server_name: dialogEmbyServer.value.server_name,
        reverse_proxy_id: 'no',
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow'
    }
    dialogAddLineVisible.value = true
}
function editLine(line: EmbyLine) {
    dialogEmbyServerAddLine.value = _.clone(line)
    dialogAddLineVisible.value = true
}
function delLine(line: EmbyLine) {
    if (line.id === dialogEmbyServer.value.line_id) {
        ElMessage.error('不能删除正在使用的服务器线路')
        return
    }
    ElMessageBox.confirm(
        `确认删除服务器「${dialogEmbyServer.value.server_name}」的线路「${line.name}」吗`,
        'Warning',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning',
        }
    ).then(async () => {
        useEmbyLine().delEmbyLine(line.id!).then(async () => {
            useEventBus().emit('EmbyLineChanged', {})
            ElMessage.success({
                message: "删除成功"
            })
        }).catch(e => ElMessage.error("删除失败" + e))
    })
}
async function savedialogEmbyServerAddLine() {
    let savePromise
    if (dialogEmbyServerAddLine.value.id) {
        savePromise = updateEmbyLineDb(dialogEmbyServerAddLine.value)
    } else {
        dialogEmbyServerAddLine.value.id = generateGuid();
        savePromise = addEmbyLineDb(dialogEmbyServerAddLine.value)
    }
    savePromise.then(async () => {
        if (dialogEmbyServerAddLine.value.id === dialogEmbyServer.value.line_id) {
            updateEmbyServerDb({
                id: dialogEmbyServerAddLine.value.emby_server_id,
                base_url: dialogEmbyServerAddLine.value.base_url,
                reverse_proxy_id: dialogEmbyServerAddLine.value.reverse_proxy_id,
                browse_proxy_id: dialogEmbyServerAddLine.value.browse_proxy_id,
                play_proxy_id: dialogEmbyServerAddLine.value.play_proxy_id
            });
        }
        if (dialogEmbyServerAddLine.value.id === showServerLine.value.id) {
            showServerLine.value = dialogEmbyServerAddLine.value
        }
        ElMessage.success({
            message: "保存成功"
        })
    }).catch(e => ElMessage.error("保存失败" + e)).finally(() => dialogAddLineVisible.value = false)
}
async function configLineChange(value: string) {
    useEmbyLine().getEmbyLine(value).then(async line => {
        if (!line) {
            ElMessage.error('获取线路失败')
            return
        }
        if (line.emby_server_id === showEmbyServer.value.id) {
            showEmbyServer.value.line_id = line.id
            showServerLine.value = line
        }
        let tmpEmbyServer = {
            id: line.emby_server_id,
            base_url: line.base_url,
            reverse_proxy_id: line.reverse_proxy_id,
            browse_proxy_id: line.browse_proxy_id,
            play_proxy_id: line.play_proxy_id,
            line_id: line.id,
        }
        updateEmbyServerDb(tmpEmbyServer)
    }).catch(e => ElMessage.error('获取线路失败' + e))
}
function proxyChange(line: EmbyLine) {
    useEmbyLine().updateEmbyLine(line).then(() => {
        useEventBus().emit('EmbyLineChanged', {})
        if (line.id === dialogEmbyServer.value.line_id || line.id === showEmbyServer.value.line_id) {
            useEmbyServer().updateEmbyServer({
                id: line.emby_server_id,
                reverse_proxy_id: line.reverse_proxy_id,
                browse_proxy_id: line.browse_proxy_id,
                play_proxy_id: line.play_proxy_id
            }).then(() => {
                useEventBus().emit('EmbyServerChanged', {event: 'update', id: line.emby_server_id})
            }).catch(e => ElMessage.error('修改失败' + e));
        }
        ElMessage.success('修改成功');
    }).catch(e => ElMessage.error('修改失败' + e));
}

// 选择反代服务器后，两个代理服务器默认切换为不使用代理
function reverseProxyChange(target: {reverse_proxy_id?: string, browse_proxy_id?: string, play_proxy_id?: string}) {
    if (target.reverse_proxy_id && target.reverse_proxy_id !== 'no') {
        target.browse_proxy_id = 'no'
        target.play_proxy_id = 'no'
    }
}

const dialogEditEmbyIconVisible = ref(false)
const selectedEmbyIconLibrary = ref('')
const embyIconLibrary = ref<EmbyIconLibrary[]>([]);
const embyIconList = ref<{name: string, url: string, local_url?: string}[]>([])
const embyIconListLoading = ref(false)
const searchEmbyIconName = ref('')
async function listAllEmbyIconLibrary() {
    return useEmbyIconLibrary().listAllEmbyIconLibrary().then(list => {
        embyIconLibrary.value = list;
        return list
    })
}
async function editEmbyIcon(embyServer: EmbyServer) {
    dialogEditEmbyIconVisible.value = true
    searchEmbyIconName.value = ''
    listAllEmbyIconLibrary().then(list => {
        if (list && list.length > 0) {
            selectedEmbyIconLibrary.value = list[0].id!
            embyIconLibraryChange()
        }
    })
    dialogEmbyServer.value = _.clone(embyServer)
}
function embyIconLibraryChange() {
    embyIconListLoading.value = true
    embyIconList.value = []
    const lib = embyIconLibrary.value.find(item => item.id === selectedEmbyIconLibrary.value)
    appApi.getEmbyIconLibrary(lib!.url!).then(response => {
        let json: {name: string, icons:{name: string, url: string}[]} = JSON.parse(response);
        embyIconList.value = json.icons
        for (const icon of embyIconList.value) {
            useImage().loadIcon(icon.url!).then(local_url => icon.local_url = local_url)
        }
    }).catch(e => ElMessage.error(e)).finally(() => embyIconListLoading.value = false)
}
function updateEmbyIcon(url: string) {
    let tmp = {
        id: dialogEmbyServer.value.id!,
        icon_url: url
    }
    updateEmbyServerDb(tmp)
    dialogEditEmbyIconVisible.value = false
}

const showEmbyServer = ref<EmbyServer>({})
const showServerLine = ref<EmbyLine>({})
watch(
  () => route.path + embyServers.value?.length + Object.keys(embyLines.value).length,
  () => {
        if (route.path.startsWith('/nav/emby/') && embyServers.value && embyServers.value.length > 0) {
            showEmbyServer.value = embyServers.value.filter(emby => emby.id == route.params.embyId)[0]
            if (embyLines.value && embyLines.value[showEmbyServer.value.id!] && embyLines.value[showEmbyServer.value.id!].length > 0) {
                showServerLine.value = embyLines.value[showEmbyServer.value.id!].filter(line => line.id === showEmbyServer.value.line_id)[0]!
            }
        }
    }
)

const global_browse_proxy_name = ref<string>('不使用代理');
function getGlobalBrowseProxy() {
    useGlobalConfig().getGlobalConfigValue("global_browse_proxy_id").then(async value => {
        global_browse_proxy_name.value = await useProxyServer().getProxyServerName(value);
    }).catch(e => ElMessage.error('获取全局浏览代理失败' + e))
}
const global_play_proxy_name = ref<string>('不使用代理');
function getGlobalPlayProxy() {
    useGlobalConfig().getGlobalConfigValue("global_play_proxy_id").then(async value => {
        global_play_proxy_name.value = await useProxyServer().getProxyServerName(value);
    }).catch(e => ElMessage.error('获取全局播放代理失败' + e))
}
function getGlobalProxy() {
    getGlobalBrowseProxy();
    getGlobalPlayProxy();
}
getGlobalProxy()
onMounted(() => useEventBus().on('GlobalProxyChanged', getGlobalProxy))
onUnmounted(() => useEventBus().remove('GlobalProxyChanged', getGlobalProxy))

const dialogNotifyCenterVisible = ref(false)
const notifyScrollbarRef = ref<ScrollbarInstance>()
const notifyMessages = computed(() => useNotifyCenter().notifyMessages);
onMounted(() => useEventBus().on('notifyMessageChange', notifyMessageChange))
onUnmounted(() => useEventBus().remove('notifyMessageChange', notifyMessageChange))
interface NotifyMessageChangeParam {
    force_open: boolean
}
function notifyMessageChange(param: NotifyMessageChangeParam) {
    if (param.force_open) {
        dialogNotifyCenterVisible.value = true;
    }
    notifyScrollBottom()
}
function notifyScrollBottom() {
    nextTick(() => {
        if (dialogNotifyCenterVisible.value && notifyScrollbarRef.value && notifyScrollbarRef.value.wrapRef && notifyScrollbarRef.value.wrapRef.scrollHeight) {
            notifyScrollbarRef.value.setScrollTop(notifyScrollbarRef.value.wrapRef.scrollHeight)
            notifyScrollbarRef.value.update()
        }
    })
}
</script>

<style scoped>
/* ——— 外壳 ——— */
.shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--ink);
}

.shell__body {
    display: flex;
    flex: auto;
    min-height: 0;
}

/* ——— 左侧导航 ——— */
.rail {
    flex: none;
    width: var(--nav-menu-width);
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 14px 10px 8px;
    background: var(--ink-lift);
    border-right: 1px solid var(--hairline);
}

.rail__brand {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 2px 8px 16px;
    cursor: pointer;
    user-select: none;
}

.rail__mark {
    align-self: center;
    display: flex;
    color: var(--lamp);
}

.rail__name {
    font-size: 17px;
    font-weight: 600;
    letter-spacing: 0.02em;
}

.rail__version {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
}

.rail__group {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--hairline);
    margin-bottom: 12px;
}

.rail__item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-base);
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
}

.rail__item:hover {
    background: #1A2028;
    color: var(--text-1);
}

.rail__item.is-active {
    background: var(--lamp-soft);
    color: var(--text-1);
}

.rail__item.is-active::before {
    content: '';
    position: absolute;
}

.rail__group .rail__item.is-active {
    box-shadow: inset 2px 0 0 var(--lamp);
}

.rail__label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px 6px;
    font-size: var(--text-xs);
    color: var(--text-3);
}

.rail__add {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: 1px solid var(--hairline);
    border-radius: 4px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition: border-color 0.15s ease, color 0.15s ease;
}

.rail__add:hover {
    border-color: var(--lamp-line);
    color: var(--lamp);
}

.rail__servers {
    flex: auto;
    min-height: 0;
    overflow: hidden;
}

.rail__item--server {
    position: relative;
    gap: 8px;
    margin-bottom: 2px;
}

.rail__item--server.is-active {
    box-shadow: inset 2px 0 0 var(--lamp);
}

.rail__item--server.is-off {
    opacity: 0.5;
}

.rail__dot {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-3);
}

.rail__dot[data-state='on'] {
    background: #4FA35E;
}

.rail__dot[data-state='unsigned'] {
    background: var(--lamp);
}

.rail__dot[data-state='off'] {
    background: #4C5560;
}

.rail__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    color: currentColor;
}

.rail__icon img {
    max-width: 16px;
    max-height: 16px;
    border-radius: 3px;
}

.rail__server-name {
    flex: auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.rail__keepalive {
    flex: none;
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px 5px;
    border-radius: 4px;
    border: 1px solid var(--hairline);
    color: var(--text-3);
}

.rail__keepalive[data-level='ok'] {
    color: #6FBF7C;
    border-color: rgba(79, 163, 94, 0.4);
}

.rail__keepalive[data-level='soon'] {
    color: var(--lamp);
    border-color: var(--lamp-line);
}

.rail__keepalive[data-level='over'] {
    color: #E07972;
    border-color: rgba(207, 91, 84, 0.45);
}

.rail__hint {
    padding: 10px;
    font-size: var(--text-sm);
    color: var(--text-3);
    line-height: 1.6;
}

/* ——— 右侧内容区 ——— */
.board {
    flex: auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
}

.topbar {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    height: var(--topbar-height);
    padding: 0 var(--page-gutter);
    border-bottom: 1px solid var(--hairline);
    background: var(--ink);
}

.topbar__left {
    display: flex;
    align-items: baseline;
    gap: 12px;
    min-width: 0;
}

.topbar__title {
    font-size: var(--text-lg);
    font-weight: 600;
    letter-spacing: 0.01em;
}

.topbar__sub {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-sm);
    color: var(--text-3);
    min-width: 0;
}

.topbar__server {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.topbar__rule {
    width: 1px;
    height: 10px;
    background: var(--hairline);
}

.topbar__status[data-state='on'] {
    color: #6FBF7C;
}

.topbar__status[data-state='unsigned'] {
    color: var(--lamp);
}

.topbar__tools {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
}

.topbar__select {
    width: 150px;
}

.topbar__select--line {
    width: 170px;
}

.topbar__key {
    color: var(--text-3);
    font-size: var(--text-xs);
    margin-right: 4px;
}

.board__scroll {
    flex: auto;
    min-height: 0;
}

/* ——— 状态栏：只放消息，不挤压内容 ——— */
.status {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--status-bar-height);
    padding: 0 12px 0 10px;
    background: var(--ink-lift);
    border-top: 1px solid var(--hairline);
    font-size: var(--text-xs);
    color: var(--text-3);
}

.status__left,
.status__right {
    display: flex;
    align-items: center;
    gap: 14px;
}

.status__notify {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-xs);
    cursor: pointer;
}

.status__notify:hover {
    background: #1A2028;
    color: var(--text-1);
}

.status__badge {
    font-family: var(--font-mono);
    color: var(--lamp);
}

.status__meta {
    font-family: var(--font-mono);
    color: var(--text-3);
}

.status__anchor {
    display: block;
    width: 1px;
    height: 1px;
}

/* ——— 消息中心 ——— */
.notify__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 10px;
    margin-bottom: 6px;
    border-bottom: 1px solid var(--hairline);
    color: var(--text-1);
    font-weight: 600;
}

.notify__close {
    display: flex;
    border: none;
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
}

.notify__empty {
    padding: 28px 0;
    text-align: center;
    color: var(--text-3);
    font-size: var(--text-sm);
}

.notify__item {
    display: flex;
    gap: 10px;
    padding: 10px 0;
    border-bottom: 1px solid var(--hairline);
}

.notify__source {
    flex: none;
    display: flex;
    color: var(--text-2);
}

.notify__body {
    flex: auto;
    min-width: 0;
}

.notify__meta {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
    font-size: var(--text-sm);
}

.notify__from {
    color: var(--text-1);
}

.notify__time {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-3);
}

.notify__content {
    margin-top: 6px;
    padding: 7px 9px;
    border-left: 2px solid var(--hairline);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    background: #1B2128;
    font-size: var(--text-sm);
    color: var(--text-2);
    word-break: break-all;
}

.notify__content[data-level='success'] {
    border-left-color: #4FA35E;
}

.notify__content[data-level='warning'] {
    border-left-color: var(--lamp);
}

.notify__content[data-level='danger'] {
    border-left-color: #CF5B54;
}

.notify__content[data-level='primary'] {
    border-left-color: var(--cyan);
}

/* ——— 对话框内统一排版 ——— */
.dlg-form {
    width: 80%;
    margin: 22px auto 4px;
}

.dlg-actions {
    display: flex;
    justify-content: flex-end;
    width: 100%;
    margin-top: 4px;
}

.dlg-actions--split {
    justify-content: space-between;
}

.dlg-done {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 34px 0 40px;
    text-align: center;
}

.dlg-done__title {
    font-size: var(--text-lg);
    color: var(--text-1);
}

.dlg-done__text {
    color: var(--text-3);
    font-size: var(--text-sm);
    margin-bottom: 10px;
}

/* 线路列表 */
.ghost-add {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 14px;
    padding: 7px 12px;
    border: 1px dashed var(--hairline);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-family: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
}

.ghost-add:hover {
    border-color: var(--lamp-line);
    color: var(--lamp);
}

.line-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
    width: 100%;
}

.line-item {
    display: flex;
    align-items: stretch;
    width: 100%;
    height: auto;
    margin: 0;
    padding: 10px 12px;
}

.line-item :deep(.el-radio__label) {
    flex: auto;
    min-width: 0;
    width: 100%;
    color: var(--text-1);
}

.line-item__body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    width: 100%;
}

.line-item__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
}

.line-item__name {
    font-weight: 500;
}

.line-item__ops {
    flex: none;
}

.line-item__url {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 图标选择 */
.iconpick__tools {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
}

.iconpick__grid {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    min-height: 200px;
}

.iconpick__cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 72px;
    padding: 8px 4px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-3);
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
}

.iconpick__cell:hover {
    border-color: var(--lamp-line);
    background: var(--lamp-soft);
    color: var(--text-1);
}

.iconpick__cell img {
    max-width: 40px;
    max-height: 40px;
}

.iconpick__cell span {
    width: 100%;
    text-align: center;
    word-break: break-all;
}
</style>
