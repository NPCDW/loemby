<template>
    <div>
        <div style="display: flex; flex-direction: row; height: calc(100vh - 30px);">
            <el-menu style="height: 100%; width: 200px; min-height: calc(100vh - 30px); background-color: var(--dark-background-color)" :collapse="false" :default-active="active">
                <el-menu-item index="/nav/history" @click="jumpRoute('/nav/history')">
                    <el-icon><i-ep-Clock /></el-icon>播放历史
                </el-menu-item>
                <el-menu-item index="/nav/search" @click="jumpRoute('/nav/search')">
                    <el-icon><i-ep-Search /></el-icon>聚合搜索
                </el-menu-item>
                <el-menu-item index="/nav/setting" @click="jumpRoute('/nav/setting')">
                    <el-icon><i-ep-Setting /></el-icon>设置
                </el-menu-item>
                <el-menu-item index="addEmbyServer" @click="addEmbyServer()">
                    <el-icon><i-ep-Plus /></el-icon>添加服务器
                </el-menu-item>
                <el-scrollbar style="height: calc(100vh - 254px); flex: none;">
                    <Container @drop="onDrop" style="height: 100%; width: 100%;">  
                        <Draggable v-for="embyServer in embyServers" :key="embyServer.id" style="height: 100%; width: 100%;">
                            <el-dropdown trigger="contextmenu" style="height: 100%; width: 100%;">
                                <el-menu-item style="height: 100%; width: 100%;" :index="'/nav/emby/' + embyServer.id" @click="jumpRoute('/nav/emby/' + embyServer.id)" :disabled="embyServer.disabled ? true : false">
                                    <div style="height: 100%; width: 100%; display: flex; align-items: center;">
                                        <el-icon size="24" style="width: 24px; height: 24px;">
                                            <img v-if="embyServer.icon_url" v-lazy="embyIconLocalUrl[embyServer.id!]" style="max-width: 24px; max-height: 24px;">
                                            <svg-icon v-else name="emby" />
                                        </el-icon>
                                        {{ embyServer.server_name }}
                                        <el-tag v-if="embyServer.keep_alive_days" disable-transitions size="small" :type="keep_alive_days[embyServer.id!] > 7 ? 'success' : keep_alive_days[embyServer.id!] > 3 ? 'warning' : 'danger'">
                                            {{ keep_alive_days[embyServer.id!] }}
                                        </el-tag>
                                    </div>
                                </el-menu-item>
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
                                        <el-dropdown-item style="color: #E6A23C" @click="logoutEmbyServer(embyServer)">
                                            <i-ep-WarnTriangleFilled style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">退出登录</span>
                                        </el-dropdown-item>
                                        <el-dropdown-item style="color: #F56C6C" @click="delEmbyServer(embyServer)">
                                            <i-ep-Delete style="position: absolute; left: 10;" />
                                            <span style="margin-left: 15px;">删除</span>
                                        </el-dropdown-item>
                                    </el-dropdown-menu>
                                </template>
                            </el-dropdown>
                        </Draggable>
                    </Container>
                </el-scrollbar>
            </el-menu>
            <el-scrollbar style="flex: auto; height: calc(100vh - 30px); width: calc(100% - 200px); position: relative;">
                <router-view v-slot="{ Component }">
                    <keep-alive>
                        <component :is="Component" :key="$route.fullPath" v-if="$route.meta.keepAlive" />
                    </keep-alive>
                    <component :is="Component" :key="$route.fullPath" v-if="!$route.meta.keepAlive" />
                </router-view>
            </el-scrollbar>
        </div>
        <div style="height: 29px; border-top: 1px solid #4c4d4f; display: flex; justify-content: space-between; align-items: center;">
            <div style="display: flex; align-items: center; margin-left: 3px;">
                <el-popover
                    :visible="dialogNotifyCenterVisible"
                    :width="400"
                    transition="el-zoom-in-bottom"
                    placement="top-start">
                    <template #reference>
                        <el-icon @click="() => {dialogNotifyCenterVisible = !dialogNotifyCenterVisible;notifyScrollBottom()}" style="margin: 0 7px;"><i-ep-BellFilled /></el-icon>
                    </template>
                    <div>
                        <div @click="dialogNotifyCenterVisible = false" style="display: flex; justify-content: space-between;">
                            <el-text>消息中心</el-text>
                            <el-icon><i-ep-ArrowDownBold /></el-icon>
                        </div>
                        <el-scrollbar max-height="500px" ref="notifyScrollbarRef">
                            <div v-if="!notifyMessages || notifyMessages.length <= 0" style="text-align: center;">
                                无新通知
                            </div>
                            <div v-else v-for="message in notifyMessages" style="display: flex; margin: 10px 7px 0 3px;">
                                <el-icon size="32" style="width: 32px; height: 32px; margin: 0 5px 0 0;">
                                    <svg-icon v-if="message.username == 'trakt'" name="trakt" />
                                    <svg-icon v-else-if="message.username == 'loemby'" name="app-icon" />
                                    <template v-else>
                                        <img v-if="message.icon && embyServerMap[message.icon] && embyServerMap[message.icon].icon_url" v-lazy="embyIconLocalUrl[embyServerMap[message.icon].id!]" style="max-width: 32px; max-height: 32px;">
                                        <svg-icon v-else name="emby" />
                                    </template>
                                </el-icon>
                                <div>
                                    <div style="display: flex; justify-content: space-between;">
                                        <el-text>{{ message.username == "embyServer" ? embyServerMap[message.icon!].server_name : message.username }}</el-text>
                                        <el-text>{{ message.datetime }}</el-text>
                                    </div>
                                    <div :style="{'background-color': messageContentBg(message.level)}" class="message-content">
                                        <component v-if="isVNode(message.content)" :is="message.content"></component>
                                        <template v-else>{{ message.content }}</template>
                                    </div>
                                </div>
                            </div>
                        </el-scrollbar>
                    </div>
                </el-popover>
                <el-text>服务器总数：{{ embyServers.length }}</el-text>
            </div>
            <div v-if="$route.path.startsWith('/nav/emby/')" style="display: flex; align-items: center; margin-right: 3px;">
                <el-text>{{ showEmbyServer.server_name }}</el-text>
                <el-select v-model="showEmbyServer.line_id" @change="configLineChange" placement="top" size="small" style="width: 180px; margin-left: 5px;">
                    <template #label="{ label }">
                        <span style="font-weight: bold">线路: </span>
                        <span>{{ label }}</span>
                    </template>
                    <el-option v-for='line in embyLines[showEmbyServer.id!]' :key="line.id" :label="line.name" :value="line.id"/>
                    <template #footer>
                        <el-button size="small" @click="configLine(showEmbyServer)">配置线路</el-button>
                    </template>
                </el-select>
                <el-select v-model="showServerLine.browse_proxy_id" @change="proxyChange(showServerLine)" placement="top" size="small" style="width: 180px; margin-left: 5px;">
                    <template #label="{ label }">
                        <span style="font-weight: bold">浏览: </span>
                        <span>{{ label }}</span>
                    </template>
                    <el-option key="no" label="不使用代理" value="no"/>
                    <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                    <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                </el-select>
                <el-select v-model="showServerLine.play_proxy_id" @change="proxyChange(showServerLine)" placement="top" size="small" style="width: 180px; margin-left: 5px;">
                    <template #label="{ label }">
                        <span style="font-weight: bold">播放: </span>
                        <span>{{ label }}</span>
                    </template>
                    <el-option key="no" label="不使用代理" value="no"/>
                    <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                    <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                </el-select>
            </div>
        </div>
    </div>

  <el-dialog v-model="dialogAddEmbyServerVisible" title="Emby Server" width="800">
    <el-steps :active="stepActive" align-center>
        <el-step title="服务器地址" />
        <el-step title="用户名密码">
        </el-step>
        <el-step title="完成">
        </el-step>
    </el-steps>
    <div v-if="stepActive == 1" style="width: 60%; margin: 25px auto;">
        <el-form label-position="top">
            <el-form-item label="服务器地址">
                <el-input v-model="dialogEmbyServer.base_url" placeholder="Please input" />
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
            <el-form-item>
                <div style="width: 100%; display: flex; justify-content: end;">
                    <el-button :loading="addEmbyServerAddrLoading" @click="addEmbyServerAddr" type="primary">下一步</el-button>
                </div>
            </el-form-item>
        </el-form>
    </div>
    <div v-if="stepActive == 2" style="width: 60%; margin: 25px auto;">
        <el-form label-position="top">
            <el-form-item label="服务器名称">
                <el-input v-model="dialogEmbyServer.server_name" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="用户名">
                <el-input v-model="dialogEmbyServer.username" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="密码">
                <el-input v-model="dialogEmbyServer.password" placeholder="Please input" show-password />
            </el-form-item>
            <el-form-item>
                <div style="width: 100%; display: flex; justify-content: space-between;">
                    <el-button :loading="addEmbyServerAuthLoading" @click="addEmbyServerPrevStep">上一步</el-button>
                    <el-button :loading="addEmbyServerAuthLoading" @click="addEmbyServerAuth" type="primary">下一步</el-button>
                </div>
            </el-form-item>
        </el-form>
    </div>
    <div v-if="stepActive == 3">
        <el-result
            icon="success"
            title="Success"
        >
            <template #extra>
                <el-button type="primary" @click="dialogAddEmbyServerVisible = false">完成</el-button>
            </template>
        </el-result>
    </div>
  </el-dialog>
  <el-dialog v-model="dialogEditEmbyServerVisible" title="Emby Server" width="800">
    <el-form label-position="top" style="width: 60%; margin: 25px auto;">
        <el-form-item label="服务器地址">
            <el-input v-model="dialogEmbyServer.base_url" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="服务器名称">
            <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                <el-input v-model="dialogEmbyServer.server_name" placeholder="Please input" />
                <el-button :loading="serverInfoLoading" @click="getServerInfo(dialogEmbyServer)" style="margin-left: 5px;">获取</el-button>
            </div>
        </el-form-item>
        <el-form-item label="用户名">
            <el-input v-model="dialogEmbyServer.username" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="密码">
            <el-input v-model="dialogEmbyServer.password" placeholder="Please input" show-password />
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
        <el-form-item label="提醒我观看（大于7天显示绿色，小于等于7天显示黄色，小于等于3天显示红色）">
            <el-input-number v-model="dialogEmbyServer.keep_alive_days" />
        </el-form-item>
        <el-form-item>
            <div style="width: 100%; display: flex; justify-content: center;">
                <el-button type="primary" @click="saveEditEmbyServer">保存</el-button>
                <el-button @click="dialogEditEmbyServerVisible = false">取消</el-button>
            </div>
        </el-form-item>
    </el-form>
  </el-dialog>
  <el-dialog v-model="dialogConfigLineVisible" title="线路配置" width="800">
    <el-scrollbar style="width: 100%;height: 400px; padding: 20px;">
        <el-button @click="addLine" type="primary">添加</el-button>
        <div style="padding-top: 20px;">
            <el-radio-group v-model="dialogEmbyServer.line_id" @change="configLineChange">
                <el-radio v-for="line in dialogEmbyServerLines" :value="line.id" size="large" border style="height: 100%; margin-bottom: 20px;">
                    <div style="padding: 10px;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            {{ line.name }}
                            <span>
                                <el-button type="primary" text size="small" @click="editLine(line)"><i-ep-Edit /></el-button>
                                <el-button type="danger" :disabled="line.id === dialogEmbyServer.line_id ? true : false" text size="small" @click="delLine(line)"><i-ep-Delete /></el-button>
                            </span>
                        </div>
                        <el-text truncated style="width: 280px;">{{ line.base_url }}</el-text>
                    </div>
                </el-radio>
            </el-radio-group>
        </div>
    </el-scrollbar>
  </el-dialog>
  <el-dialog v-model="dialogAddLineVisible" title="配置线路" width="800">
    <el-form label-position="top" style="width: 60%; margin: 25px auto;">
        <el-form-item label="线路名称">
            <el-input v-model="dialogEmbyServerAddLine.name" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="线路地址">
            <el-input v-model="dialogEmbyServerAddLine.base_url" placeholder="Please input" />
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
        <el-form-item>
            <div style="width: 100%; display: flex; justify-content: center;">
                <el-button type="primary" @click="savedialogEmbyServerAddLine">保存</el-button>
                <el-button @click="dialogAddLineVisible = false">取消</el-button>
            </div>
        </el-form-item>
    </el-form>
  </el-dialog>
  <el-dialog v-model="dialogEditEmbyIconVisible" title="Emby Icon" width="400" style="height: 400px;">
    <el-select v-model="selectedEmbyIconLibrary" @change="embyIconLibraryChange">
      <el-option
        v-for="item in embyIconLibrary"
        :key="item.id"
        :label="item.name"
        :value="item.id"
      />
    </el-select>
    <el-input v-model="searchEmbyIconName" placeholder="搜索图标" style="margin-top: 5px;" />
    <el-scrollbar style="height: 240px; padding: 10px;">
        <div style="display: flex; flex-wrap: wrap; flex-direction: row; height: 240px;" v-loading="embyIconListLoading">
            <template v-for="embyIcon in embyIconList">
                <div style="display: flex; flex-direction: column; align-items: center; margin: 5px; width: 75px;" v-if="embyIcon.name.toLowerCase().includes(searchEmbyIconName.toLowerCase())">
                        <el-icon :size="48" @click="updateEmbyIcon(embyIcon.url)" style="max-height: 48px; max-width: 48px;">
                            <img v-lazy="embyIcon.local_url" style="max-height: 48px; max-width: 48px; cursor: pointer;" />
                        </el-icon>
                        <span style="word-break: break-all; font-size: small; max-width: 60px; text-align: center;">{{ embyIcon.name }}</span>
                </div>
            </template>
        </div>
    </el-scrollbar>
  </el-dialog>
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

const active = ref("/nav/search");
const route = useRoute();
const router = useRouter()
watchEffect(() => {
    active.value = route.path;
    console.log(active.value)
})

const proxyServers = ref<ProxyServer[]>([]);
function listAllProxyServer() {
    useProxyServer().listAllProxyServer().then(list => {
        proxyServers.value = list;
    })
}
listAllProxyServer()
onMounted(() => useEventBus().on('ProxyServerChanged', listAllProxyServer))
onUnmounted(() => useEventBus().remove('ProxyServerChanged', listAllProxyServer))

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
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
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
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
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
        }).catch(e => {
            ElMessage.error('删除Emby服务器失败' + e)
        })
    })
}

const addEmbyServerAddrLoading = ref(false)
async function addEmbyServerAddr() {
    addEmbyServerAddrLoading.value = true
    if (!dialogEmbyServer || !dialogEmbyServer.value?.base_url) {
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
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
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
            confirmButtonText: 'OK',
            cancelButtonText: 'Cancel',
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
                browse_proxy_id: line.browse_proxy_id,
                play_proxy_id: line.play_proxy_id
            }).then(() => {
                useEventBus().emit('EmbyServerChanged', {event: 'update', id: line.emby_server_id})
            }).catch(e => ElMessage.error('修改失败' + e));
        }
        ElMessage.success('修改成功');
    }).catch(e => ElMessage.error('修改失败' + e));
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
function messageContentBg(level?: string): string {
    switch (level) {
        case "primary": return "rgb(33, 61, 91)";
        case "success": return "rgb(45, 72, 31)";
        case "warning": return "rgb(83, 63, 32)";
        case "danger": return "rgb(88, 46, 46)";
        case "info": return "rgb(57, 58, 60)";
        default: return "rgb(57, 58, 60)";
    }
}
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
.message-content {
    margin: 0;
    padding: 5px;
    background-color: #303030;
    word-break: break-all;
    border-radius: 5px;
}
</style>