<template>
    <div style="display: flex; flex-direction: row;">
        <el-menu style="height: 100%; width: 200px; min-height: 100vh;" class="el-menu" :collapse="true" :router="true" :default-active="active">
            <el-menu-item index="/nav/search">
                <el-icon><i-ep-Search /></el-icon>聚合搜索
            </el-menu-item>
            <el-menu-item index="/nav/setting">
                <el-icon><i-ep-Setting /></el-icon>设置
            </el-menu-item>
            <el-menu-item index="" @click="addEmbyServer()">
                <el-icon><i-ep-Plus /></el-icon>添加服务器
            </el-menu-item>
            <el-scrollbar style="height: calc(100vh - 168px); flex: none;">
                <Container @drop="onDrop" style="height: 100%; width: 100%;">  
                    <Draggable v-for="embyServer in embyServers" :key="embyServer.id" style="height: 100%; width: 100%;">
                        <el-dropdown trigger="contextmenu" style="height: 100%; width: 100%;">
                            <el-menu-item style="height: 100%; width: 100%;" :index="'/nav/emby/' + embyServer.id" :disabled="embyServer.disabled ? true : false">
                                <div style="height: 100%; width: 100%; display: flex; align-items: center;">
                                    <el-icon v-if="embyServer.disabled" style="color: #909399;"><i-ep-CircleCloseFilled /></el-icon>
                                    <!-- <el-icon v-else-if="embyServer.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                                    <el-icon v-else-if="embyServer.request_fail" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon> -->
                                    <el-icon v-else size="24"><svg-icon name="emby" /></el-icon>
                                    {{ embyServer.server_name }}
                                    <el-tag v-if="embyServer.keep_alive_days" disable-transitions size="small" :type="keep_alive_days[embyServer.id!] > 7 ? 'success' : keep_alive_days[embyServer.id!] > 3 ? 'warning' : 'danger'">
                                        {{ '+' + keep_alive_days[embyServer.id!] }}
                                    </el-tag>
                                </div>
                            </el-menu-item>
                            <template #dropdown>
                                <el-dropdown-menu>
                                    <el-dropdown-item>
                                        <el-dropdown placement="right-start" trigger="hover" style="width: 100%; height: 100%;" >
                                            <span style="width: 100%; height: 100%;">
                                                <i-ep-Promotion style="position: absolute; left: 10; margin-left: -15px;" />
                                                <span style="margin-left: 15px;">线路</span>
                                            </span>
                                            <template #dropdown>
                                                <el-dropdown-menu>
                                                    <el-dropdown-item v-for='line in embyLines[embyServer.id!]' @click="configLineChange(line.id!, embyServer)">
                                                        <i-ep-Select v-if="line.in_use" style="position: absolute; left: 10;" />
                                                        <span style="margin-left: 15px;">{{line.name}}</span>
                                                    </el-dropdown-item>
                                                    <el-dropdown-item divided @click="configLine(embyServer)">
                                                        <i-ep-SetUp style="position: absolute; left: 10;" />
                                                        <span style="margin-left: 15px;">配置</span>
                                                    </el-dropdown-item>
                                                </el-dropdown-menu>
                                            </template>
                                        </el-dropdown>
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
        <el-scrollbar style="flex: auto; height: 100vh; width: calc(100% - 200px);">
            <router-view></router-view>
        </el-scrollbar>
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
                <el-input v-model="currentEmbyServer.base_url" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="媒体库浏览代理">
                <el-select v-model="currentEmbyServer.browse_proxy_id">
                    <el-option key="no" label="不使用代理" value="no"/>
                    <el-option key="follow" label="跟随全局代理" value="follow"/>
                    <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                </el-select>
            </el-form-item>
            <el-form-item label="媒体流播放代理">
                <el-select v-model="currentEmbyServer.play_proxy_id">
                    <el-option key="no" label="不使用代理" value="no"/>
                    <el-option key="follow" label="跟随全局代理" value="follow"/>
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
                <el-input v-model="currentEmbyServer.server_name" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="用户名">
                <el-input v-model="currentEmbyServer.username" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="密码">
                <el-input v-model="currentEmbyServer.password" placeholder="Please input" show-password />
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
            <el-input v-model="currentEmbyServer.base_url" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="服务器名称">
            <el-input v-model="currentEmbyServer.server_name" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="用户名">
            <el-input v-model="currentEmbyServer.username" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="密码">
            <el-input v-model="currentEmbyServer.password" placeholder="Please input" show-password />
        </el-form-item>
        <el-form-item label="媒体库代理">
            <el-select v-model="currentEmbyServer.browse_proxy_id">
                <el-option key="no" label="不使用代理" value="no"/>
                <el-option key="follow" label="跟随全局代理" value="follow"/>
                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
            </el-select>
        </el-form-item>
        <el-form-item label="媒体流代理">
            <el-select v-model="currentEmbyServer.play_proxy_id">
                <el-option key="no" label="不使用代理" value="no"/>
                <el-option key="follow" label="跟随全局代理" value="follow"/>
                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
            </el-select>
        </el-form-item>
        <el-form-item label="播放保活天数（仅显示剩余天数，无自动保活功能）">
            <el-input-number v-model="currentEmbyServer.keep_alive_days" />
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
            <el-radio-group v-model="currentEmbyServerLineId" @change="configLineChange(currentEmbyServerLineId, currentEmbyServer)">
                <el-radio v-for="line in currentEmbyServerLines" :value="line.id" size="large" border style="height: 100%; margin-bottom: 20px;">
                    <div style="padding: 10px;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            {{ line.name }}
                            <span>
                                <el-button type="primary" text size="small" @click="editLine(line)"><i-ep-Edit /></el-button>
                                <el-button type="danger" :disabled="line.in_use ? true : false" text size="small" @click="delLine(line)"><i-ep-Delete /></el-button>
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
            <el-input v-model="currentEmbyServerAddLine.name" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="线路地址">
            <el-input v-model="currentEmbyServerAddLine.base_url" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="媒体库代理">
            <el-select v-model="currentEmbyServerAddLine.browse_proxy_id">
                <el-option key="no" label="不使用代理" value="no"/>
                <el-option key="follow" label="跟随全局代理" value="follow"/>
                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
            </el-select>
        </el-form-item>
        <el-form-item label="媒体流代理">
            <el-select v-model="currentEmbyServerAddLine.play_proxy_id">
                <el-option key="no" label="不使用代理" value="no"/>
                <el-option key="follow" label="跟随全局代理" value="follow"/>
                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
            </el-select>
        </el-form-item>
        <el-form-item>
            <div style="width: 100%; display: flex; justify-content: center;">
                <el-button type="primary" @click="saveCurrentEmbyServerAddLine">保存</el-button>
                <el-button @click="dialogAddLineVisible = false">取消</el-button>
            </div>
        </el-form-item>
    </el-form>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watchEffect } from "vue";
import { useRoute } from 'vue-router'
import embyApi from '../api/embyApi'
import { ElMessage, ElMessageBox } from "element-plus";
import { generateGuid } from "../util/uuid_util";
import _ from "lodash";
import { Container, Draggable } from "vue3-smooth-dnd";
import invoke from "../api/invokeApi";
import { ProxyServer, useProxyServer } from "../store/db/proxyServer";
import { EmbyServer, useEmbyServer } from "../store/db/embyServer";
import { EmbyLine, useEmbyLine } from "../store/db/embyLine";
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'
import { useEventBus } from "../store/eventBus";

const active = ref("");
const route = useRoute();
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
useEventBus().on('ProxyServerChanged', listAllProxyServer)
onUnmounted(() => useEventBus().remove('ProxyServerChanged', listAllProxyServer))

const embyServers = ref<EmbyServer[]>([])
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
listAllEmbyServer()
useEventBus().on('EmbyServerChanged', listAllEmbyServer)
onUnmounted(() => useEventBus().remove('EmbyServerChanged', listAllEmbyServer))

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
        currentEmbyServerLines.value = embyLines.value[currentEmbyServer.value.id!]
    })
}
listAllEmbyLine()
useEventBus().on('EmbyLineChanged', listAllEmbyLine)
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
const currentEmbyServer = ref<EmbyServer>({})
function addEmbyServer() {
    stepActive.value = 1;
    dialogAddEmbyServerVisible.value = true
    invoke.getSysInfo().then(hostname => {
        const client = "loemby";
        const client_version = import.meta.env.VITE_APP_VERSION;
        const user_agent = client + "/" + client_version;
        useEmbyServer().getMaxOrderBy().then(max_order_by => {
            currentEmbyServer.value = {
                id: generateGuid(),
                server_name: '未完成',
                disabled: 1,
                keep_alive_days: 0,
                user_agent: user_agent,
                client: client,
                client_version: client_version,
                device: hostname,
                device_id: hostname,
                order_by: max_order_by! + 1,
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
    currentEmbyServer.value = _.clone(embyServer)
}
async function enabledEmbyServer(embyServer: EmbyServer) {
    if (!embyServer.auth_token && embyServer.disabled) {
        ElMessage.error({
            message: '请先登录'
        })
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
        await embyApi.logout(embyServer)
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
    if (!currentEmbyServer || !currentEmbyServer.value?.base_url) {
        return
    }
    let embyServer = await useEmbyServer().getEmbyServer(currentEmbyServer.value.id!)
    if (!embyServer) {
        let line = {
            id: generateGuid(),
            name: '线路一',
            base_url: currentEmbyServer.value!.base_url,
            emby_server_id: currentEmbyServer.value.id!,
            emby_server_name: currentEmbyServer.value.server_name,
            in_use: 1,
            browse_proxy_id: currentEmbyServer.value!.browse_proxy_id,
            play_proxy_id: currentEmbyServer.value!.play_proxy_id
        }
        await addEmbyLineDb(line)
        await addEmbyServerDb(currentEmbyServer.value);
    }
    embyApi.getServerInfo(currentEmbyServer.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json: {ServerName: string, Id: string} = JSON.parse(response.body);
        currentEmbyServer.value!.server_name = json['ServerName']
        currentEmbyServer.value!.server_id = json['Id']
        await updateEmbyServerDb(currentEmbyServer.value);
        updateEmbyLineServerName(currentEmbyServer.value!.id!, currentEmbyServer.value!.server_name!);
        stepActive.value = stepActive.value + 1;
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => addEmbyServerAddrLoading.value = false)
}
const addEmbyServerAuthLoading = ref(false)
function addEmbyServerPrevStep() {
    addEmbyServerAuthLoading.value = true
    stepActive.value = stepActive.value - 1;
    addEmbyServerAuthLoading.value = false
}
async function addEmbyServerAuth() {
    if (!currentEmbyServer || !currentEmbyServer.value?.username) {
        ElMessage.error('请至少填写用户名')
        return
    }
    updateEmbyLineServerName(currentEmbyServer.value.id!, currentEmbyServer.value.server_name!);
    addEmbyServerAuthLoading.value = true
    login(currentEmbyServer.value).then(() => {
        stepActive.value = stepActive.value + 1;
    }).catch(e => {
        ElMessage.error(e)
    }).finally(() => addEmbyServerAuthLoading.value = false)
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
    login(embyServerConfig).then(() => {
        ElMessage.success({
            message: "登录成功"
        })
    }).catch(e => {
        ElMessage.error(e)
    })
    })
}
async function login(embyServerConfig: EmbyServer) {
    await updateEmbyServerDb(embyServerConfig);
    return embyApi.authenticateByName(embyServerConfig).then(async response => {
        if (response.status_code != 200) {
            return Promise.reject('response status' + response.status_code + ' ' + response.status_text)
        }
        let json: {User: {Id: string}, AccessToken: string} = JSON.parse(response.body);
        embyServerConfig.auth_token = json['AccessToken']
        embyServerConfig.user_id = json["User"]['Id']
        embyServerConfig.disabled = 0
        await updateEmbyServerDb(embyServerConfig);
    })
}
async function saveEditEmbyServer() {
    await useEmbyLine().getUsingEmbyLine(currentEmbyServer.value!.id!).then(async line => {
        if (!line) {
            ElMessage.error('获取正在使用的线路失败')
            return
        }
        line.base_url = currentEmbyServer.value!.base_url
        line.browse_proxy_id = currentEmbyServer.value!.browse_proxy_id
        line.play_proxy_id = currentEmbyServer.value!.play_proxy_id
        await updateEmbyLineDb(line)
        if (line.emby_server_name != currentEmbyServer.value!.server_name) {
            await updateEmbyLineServerName(currentEmbyServer.value!.id!, currentEmbyServer.value!.server_name!);
        }
    }).catch(e => ElMessage.error('获取正在使用的线路失败' + e))
    await updateEmbyServerDb(currentEmbyServer.value!);
    ElMessage.success({
        message: "保存成功"
    })
    dialogEditEmbyServerVisible.value = false
}

async function onDrop({removedIndex, addedIndex}: {removedIndex: number, addedIndex: number}) {
    useEmbyServer().updateOrder(embyServers.value[removedIndex].id!, embyServers.value[removedIndex].order_by!, embyServers.value[addedIndex].order_by!).then(() => {
        useEventBus().emit('EmbyServerChanged', {})
    }).catch(e => ElMessage.error("排序失败" + e))
    // 页面操作，防止刷新Emby列表闪烁
    let element = embyServers.value.splice(removedIndex, 1);
    embyServers.value.splice(addedIndex, 0, element[0]);
}

const dialogConfigLineVisible = ref(false)
const currentEmbyServerLineId = ref('')
const currentEmbyServerLines = ref<EmbyLine[]>([])
function configLine(embyServer: EmbyServer) {
    currentEmbyServer.value = _.clone(embyServer)
    useEmbyLine().getUsingEmbyLine(currentEmbyServer.value!.id!).then(async line => {
        if (!line) {
            ElMessage.error('获取正在使用的线路失败')
            return
        }
        currentEmbyServerLineId.value = line.id!
    }).catch(e => ElMessage.error('获取正在使用的线路失败' + e))
    currentEmbyServerLines.value = embyLines.value[embyServer.id!]
    dialogConfigLineVisible.value = true
}
const dialogAddLineVisible = ref(false)
const currentEmbyServerAddLine = ref<EmbyLine>({})
function addLine() {
    currentEmbyServerAddLine.value = {
        in_use: 0,
        emby_server_id: currentEmbyServer.value.id!,
        emby_server_name: currentEmbyServer.value.server_name,
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow'
    }
    dialogAddLineVisible.value = true
}
function editLine(line: EmbyLine) {
    currentEmbyServerAddLine.value = _.clone(line)
    dialogAddLineVisible.value = true
}
function delLine(line: EmbyLine) {
    if (line.in_use) {
        ElMessage.error({
            message: '不能删除正在使用的服务器线路'
        })
        return
    }
    ElMessageBox.confirm(
        `确认删除服务器「${currentEmbyServer.value.server_name}」的线路「${line.name}」吗`,
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
async function saveCurrentEmbyServerAddLine() {
    let savePromise
    if (currentEmbyServerAddLine.value.id) {
        savePromise = updateEmbyLineDb(currentEmbyServerAddLine.value)
    } else {
        currentEmbyServerAddLine.value.id = generateGuid();
        savePromise = addEmbyLineDb(currentEmbyServerAddLine.value)
    }
    savePromise.then(async () => {
        if (currentEmbyServerAddLine.value.in_use) {
            updateEmbyServerDb({
                id: currentEmbyServerAddLine.value.emby_server_id,
                browse_proxy_id: currentEmbyServerAddLine.value.browse_proxy_id,
                play_proxy_id: currentEmbyServerAddLine.value.play_proxy_id
            });
        }
        ElMessage.success({
            message: "保存成功"
        })
    }).catch(e => ElMessage.error("保存失败" + e)).finally(() => dialogAddLineVisible.value = false)
}
async function configLineChange(value: string, embyServer: EmbyServer) {
    useEmbyLine().getEmbyLine(value).then(async line => {
        if (!line) {
            ElMessage.error('获取线路失败')
            return
        }
        useEmbyLine().updateEmbyUsing(embyServer.id!).then(() => {
            updateEmbyLineDb({id: value, in_use: 1}).catch(e => ElMessage.error("切换失败" + e))
            let tmpEmbyServer = {
                id: embyServer.id!,
                base_url: line.base_url,
                browse_proxy_id: line.browse_proxy_id,
                play_proxy_id: line.play_proxy_id,
            }
            updateEmbyServerDb(tmpEmbyServer).then(() => {
                ElMessage.success({
                    message: "线路切换成功"
                })
            }).catch(e => ElMessage.error("切换失败" + e))
        }).catch(e => ElMessage.error("切换失败" + e))
    }).catch(e => ElMessage.error('获取线路失败' + e))
}
</script>

<style scoped>
.el-menu {
    width: 100%;
    height: 100vh;
    background-color: var(--dark-background-color);
}
</style>