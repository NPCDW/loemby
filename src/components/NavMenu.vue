<template>
    <div style="display: flex; flex-direction: row;">
        <el-scrollbar style="height: 100vh; flex: none; width: 200px;">
            <el-menu style="height: 100%; min-height: 100vh;" class="el-menu" :collapse="true" :router="true" :default-active="active">
                <el-menu-item index="/nav/search">
                    <el-icon><i-ep-Search /></el-icon>聚合搜索
                </el-menu-item>
                <el-menu-item index="/nav/setting">
                    <el-icon><i-ep-Setting /></el-icon>设置
                </el-menu-item>
                <el-menu-item @click="addEmbyServer">
                    <el-icon><i-ep-Plus /></el-icon>添加服务器
                </el-menu-item>
                <Container @drop="onDrop" style="height: 100%; width: 100%;">  
                    <Draggable v-for="embyServer in embyServers" :key="embyServer.id" style="height: 100%; width: 100%;">
                        <el-dropdown trigger="contextmenu" style="height: 100%; width: 100%;">
                            <el-menu-item style="height: 100%; width: 100%;" :index="'/nav/emby/' + embyServer.id" :disabled="embyServer.disabled">
                                <div style="height: 100%; width: 100%; display: flex; align-items: center;">
                                    <el-icon v-if="embyServer.disabled" style="color: #909399;"><i-ep-CircleCloseFilled /></el-icon>
                                    <!-- <el-icon v-else-if="embyServer.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                                    <el-icon v-else-if="embyServer.request_fail" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon> -->
                                    <el-icon v-else size="24"><svg-icon name="emby" /></el-icon>
                                    {{ embyServer.server_name }}
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
                                                    <el-dropdown-item v-for='line in embyServer.line' @click="configLineChange(line.id!, embyServer)">
                                                        <i-ep-Select v-if="line.using" style="position: absolute; left: 10;" />
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
            </el-menu>
        </el-scrollbar>
        <div style="flex: auto; height: 100vh; width: calc(100% - 200px);">
            <router-view></router-view>
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
                <el-input v-model="currentEmbyServerConfig.base_url" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="媒体库代理">
                <el-select v-model="currentEmbyServerConfig.browse_proxy_id">
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
                <el-input v-model="currentEmbyServerConfig.server_name" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="用户名">
                <el-input v-model="currentEmbyServerConfig.username" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="密码">
                <el-input v-model="currentEmbyServerConfig.password" placeholder="Please input" show-password />
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
            <el-input v-model="currentEmbyServerConfig.base_url" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="服务器名称">
            <el-input v-model="currentEmbyServerConfig.server_name" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="用户名">
            <el-input v-model="currentEmbyServerConfig.username" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="密码">
            <el-input v-model="currentEmbyServerConfig.password" placeholder="Please input" show-password />
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
            <el-radio-group v-model="currentEmbyServerConfigLine" @change="configLineChange(currentEmbyServerConfigLine, currentEmbyServerConfig)">
                <el-radio v-for="line in currentEmbyServerConfig.line" :value="line.id" size="large" border style="height: 100%; margin-bottom: 20px;">
                    <div style="padding: 10px;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            {{ line.name }}
                            <span>
                                <el-button type="primary" text @click="editLine(line)"><i-ep-Edit /></el-button>
                                <el-button type="danger" text @click="delLine(line)"><i-ep-Delete /></el-button>
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
import { ref, watchEffect } from "vue";
import { useRoute } from 'vue-router'
import { useConfig, EmbyServerConfig, ServerLine } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage, ElMessageBox } from "element-plus";
import { generateGuid } from "../util/uuid_util";
import _ from "lodash";
import { Container, Draggable } from "vue3-smooth-dnd";
import invoke from "../api/invoke";

const active = ref("");
const route = useRoute();
watchEffect(() => {
    active.value = route.path;
    console.log(active.value)
})

let config = useConfig().get_config()
const proxyServers = config.proxy_server ? config.proxy_server : [];
const embyServers = ref(config?.emby_server ? config.emby_server : [])
for (let index = 0; index < embyServers.value.length; index++) {
    if (!embyServers.value[index].line || embyServers.value[index].line!.length == 0) {
        let line = {
            id: generateGuid(),
            name: embyServers.value[index].server_name,
            base_url: embyServers.value[index].base_url,
            using: true,
            browse_proxy_id: embyServers.value[index].browse_proxy_id,
            play_proxy_id: embyServers.value[index].play_proxy_id
        }
        embyServers.value[index].line = [line]
    }
}
async function saveEmbyServer(tmp: EmbyServerConfig) {
    let value = _.cloneDeep(tmp);
    for (let index = 0; index < embyServers.value.length; index++) {
        if (embyServers.value[index].id === value.id) {
            embyServers.value[index] = value
            await useConfig().saveEmbyServer(embyServers.value)
            return
        }
    }
    embyServers.value.push(value)
    await useConfig().saveEmbyServer(embyServers.value)
}


const stepActive = ref(1)
const dialogAddEmbyServerVisible = ref(false)
const currentEmbyServerConfig = ref<EmbyServerConfig>({})
function addEmbyServer() {
    stepActive.value = 1;
    dialogAddEmbyServerVisible.value = true
    invoke.getSysInfo().then(hostname => {
        const client = "loemby";
        const client_version = "0.4.6";
        const user_agent = client + "/" + client_version;
        currentEmbyServerConfig.value = {
            id: generateGuid(),
            server_name: '未完成',
            disabled: true,
            user_agent: user_agent,
            client: client,
            client_version: client_version,
            device: hostname,
            device_id: hostname,
            browse_proxy_id: 'follow',
            play_proxy_id: 'follow',
            line: [],
        }
    }).catch(e => {
        ElMessage.error({
            message: '获取主机名失败' + e
        })
    })
}
const dialogEditEmbyServerVisible = ref(false)
function editEmbyServer(embyServer: EmbyServerConfig) {
    dialogEditEmbyServerVisible.value = true
    currentEmbyServerConfig.value = _.clone(embyServer)
}
async function enabledEmbyServer(embyServer: EmbyServerConfig) {
    if (!embyServer.auth_token && embyServer.disabled) {
        ElMessage.error({
            message: '请先登录'
        })
        return
    }
    embyServer.disabled = !embyServer.disabled
    await saveEmbyServer(embyServer)
}
function logoutEmbyServer(embyServer: EmbyServerConfig) {
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
    embyServer.disabled = true
    embyServer.auth_token = ''
    await saveEmbyServer(embyServer)
    })
}
function delEmbyServer(tmp: EmbyServerConfig) {
  ElMessageBox.confirm(
    `确认删除服务器「${tmp.server_name}」吗`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        for (let index = 0; index < embyServers.value.length; index++) {
            if (embyServers.value[index].id === tmp.id) {
                embyServers.value.splice(index, 1)
                await useConfig().saveEmbyServer(embyServers.value)
                return
            }
        }
    })
}

const addEmbyServerAddrLoading = ref(false)
async function addEmbyServerAddr() {
    addEmbyServerAddrLoading.value = true
    if (!currentEmbyServerConfig || !currentEmbyServerConfig.value?.base_url) {
        return
    }
    syncModifyLine(currentEmbyServerConfig.value!)
    await saveEmbyServer(currentEmbyServerConfig.value);
    embyApi.getServerInfo(currentEmbyServerConfig.value).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: {ServerName: string, Id: string} = JSON.parse(response.body);
        currentEmbyServerConfig.value!.server_name = json['ServerName']
        currentEmbyServerConfig.value!.server_id = json['Id']
        await saveEmbyServer(currentEmbyServerConfig.value);
        stepActive.value = stepActive.value + 1;
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => addEmbyServerAddrLoading.value = false)
}
const addEmbyServerAuthLoading = ref(false)
function addEmbyServerPrevStep() {
    addEmbyServerAuthLoading.value = true
    stepActive.value = stepActive.value - 1;
    addEmbyServerAuthLoading.value = false
}
async function addEmbyServerAuth() {
    addEmbyServerAuthLoading.value = true
    if (!currentEmbyServerConfig || !currentEmbyServerConfig.value?.username) {
        return
    }
    login(currentEmbyServerConfig.value).then(() => {
        stepActive.value = stepActive.value + 1;
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => addEmbyServerAuthLoading.value = false)
}
async function reLogin(embyServerConfig: EmbyServerConfig) {
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
        ElMessage.error({
            message: e
        })
    })
    })
}
async function login(embyServerConfig: EmbyServerConfig) {
    await saveEmbyServer(embyServerConfig);
    return embyApi.authenticateByName(embyServerConfig).then(async response => {
        if (response.status_code != 200) {
            return Promise.reject('response status' + response.status_code + ' ' + response.status_text)
        }
        let json: {User: {Id: string}, AccessToken: string} = JSON.parse(response.body);
        embyServerConfig.auth_token = json['AccessToken']
        embyServerConfig.user_id = json["User"]['Id']
        embyServerConfig.disabled = false
        await saveEmbyServer(embyServerConfig);
    })
}
async function saveEditEmbyServer() {
    syncModifyLine(currentEmbyServerConfig.value!)
    await saveEmbyServer(currentEmbyServerConfig.value!);
    dialogEditEmbyServerVisible.value = false
}
function syncModifyLine(embyServer: EmbyServerConfig) {
    if (!embyServer.line || embyServer.line.length == 0) {
        let line = {
            id: generateGuid(),
            name: embyServer.server_name,
            base_url: embyServer.base_url,
            using: true,
            browse_proxy_id: embyServer.browse_proxy_id,
            play_proxy_id: embyServer.play_proxy_id
        }
        embyServer.line = [line]
    }
    for (let index = 0; index < embyServer.line!.length; index++) {
        if (embyServer.line![index].using) {
            embyServer.line![index].base_url = embyServer.base_url
            embyServer.line![index].browse_proxy_id = embyServer.browse_proxy_id
            embyServer.line![index].play_proxy_id = embyServer.play_proxy_id
        }
    }
}

async function onDrop(dropResult: {removedIndex: number, addedIndex: number}) {
    let element = embyServers.value.splice(dropResult.removedIndex, 1);
    embyServers.value.splice(dropResult.addedIndex, 0, element[0]);
    await useConfig().saveEmbyServer(embyServers.value)
}

const dialogConfigLineVisible = ref(false)
const currentEmbyServerConfigLine = ref('')
function configLine(embyServer: EmbyServerConfig) {
    currentEmbyServerConfig.value = _.clone(embyServer)
    if (!embyServer.line || embyServer.line.length == 0) {
        let line = {
            id: generateGuid(),
            name: embyServer.server_name,
            base_url: embyServer.base_url,
            using: true,
            browse_proxy_id: embyServer.browse_proxy_id,
            play_proxy_id: embyServer.play_proxy_id
        }
        currentEmbyServerConfig.value.line = [line]
    }
    for (let index = 0; index < currentEmbyServerConfig.value.line!.length; index++) {
        if (currentEmbyServerConfig.value.line![index].using) {
            currentEmbyServerConfigLine.value = currentEmbyServerConfig.value.line![index].id!
        }
    }
    dialogConfigLineVisible.value = true
}
const dialogAddLineVisible = ref(false)
const currentEmbyServerAddLine = ref<ServerLine>({})
function addLine() {
    currentEmbyServerAddLine.value = {
        id: generateGuid(),
        using: false,
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow'
    }
    console.log(currentEmbyServerAddLine.value)
    dialogAddLineVisible.value = true
}
function editLine(line: ServerLine) {
    currentEmbyServerAddLine.value = _.clone(line)
    dialogAddLineVisible.value = true
}
function delLine(line: ServerLine) {
    if (!currentEmbyServerConfig.value.line || currentEmbyServerConfig.value.line.length <= 1) {
        ElMessage.error({
            message: '至少保留一个服务器线路'
        })
        return
    }
    ElMessageBox.confirm(
        `确认删除服务器「${currentEmbyServerConfig.value.server_name}」的线路「${line.name}」吗`,
        'Warning',
        {
            confirmButtonText: 'OK',
            cancelButtonText: 'Cancel',
            type: 'warning',
        }
    ).then(async () => {
        currentEmbyServerConfig.value.line = currentEmbyServerConfig.value.line!.filter(item => item.id != line.id)
        await saveEmbyServer(currentEmbyServerConfig.value!);
    })
}
async function saveCurrentEmbyServerAddLine() {
    let value = _.cloneDeep(currentEmbyServerAddLine.value);
    for (let index = 0; index < currentEmbyServerConfig.value.line!.length; index++) {
        if (currentEmbyServerConfig.value.line![index].id === value.id) {
            currentEmbyServerConfig.value.line![index] = value
            await saveEmbyServer(currentEmbyServerConfig.value!);
            dialogAddLineVisible.value = false
            return
        }
    }
    currentEmbyServerConfig.value.line!.push(value)
    await saveEmbyServer(currentEmbyServerConfig.value!);
    dialogAddLineVisible.value = false
}
async function configLineChange(value: string, embyServer: EmbyServerConfig) {
    for (let index = 0; index < embyServer.line!.length; index++) {
        embyServer.line![index].using = false
        if (embyServer.line![index].id === value) {
            embyServer.line![index].using = true
            embyServer.base_url = embyServer.line![index].base_url
            embyServer.browse_proxy_id = embyServer.line![index].browse_proxy_id
            embyServer.play_proxy_id = embyServer.line![index].play_proxy_id
        }
    }
    await saveEmbyServer(embyServer!);
    ElMessage.success({
        message: "线路切换成功"
    })
}
</script>

<style scoped>
.el-menu {
    width: 100%;
    height: 100vh;
    background-color: var(--dark-background-color);
}
</style>