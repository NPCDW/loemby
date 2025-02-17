<template>
    <div style="height: 100vh; display: flex; flex-direction: row;">
        <div style="flex: none; width: 200px;">
            <el-menu class="el-menu" :collapse="true" :router="true" :default-active="active">
                <el-menu-item index="/nav/search">
                    <el-icon><i-ep-Search /></el-icon>聚合搜索
                </el-menu-item>
                <el-menu-item @click="addEmbyServer">
                    <el-icon><i-ep-Plus /></el-icon>添加服务器
                </el-menu-item>
                <el-menu-item v-if="embyServers" v-for="embyServer in embyServers">
                    <el-icon v-if="embyServer.disabled" style="color: #909399;"><i-ep-CircleCloseFilled /></el-icon>
                    <el-icon v-else-if="embyServer.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                    <el-icon v-else-if="embyServer.request_fail" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon>
                    <el-icon v-else style="color: #67C23A;"><i-ep-SuccessFilled /></el-icon>
                    {{ embyServer.server_name }}
                    <el-dropdown trigger="click">
                        <span class="el-dropdown-link">
                            <el-icon style="color: #F56C6C;"><i-ep-MoreFilled /></el-icon>
                        </span>
                        <template #dropdown>
                            <el-dropdown-menu>
                                <el-dropdown-item @click="reLogin(embyServer)">重新登录</el-dropdown-item>
                                <el-dropdown-item @click="editEmbyServer(embyServer)">编辑</el-dropdown-item>
                                <el-dropdown-item @click="enabledEmbyServer(embyServer)" v-text="embyServer.disabled ? '启用' : '禁用'"></el-dropdown-item>
                                <el-dropdown-item @click="logoutEmbyServer(embyServer)">退出登录</el-dropdown-item>
                                <el-dropdown-item @click="delEmbyServer(embyServer)">删除</el-dropdown-item>
                            </el-dropdown-menu>
                        </template>
                    </el-dropdown>
                </el-menu-item>
            </el-menu>
        </div>
        <div style="flex: auto;">
            <router-view></router-view>
        </div>
    </div>
  <el-dialog v-model="dialogAddEmbyServerVisible" title="Emby Server" width="800">
    <el-steps style="max-width: 600px" :active="stepActive" align-center>
        <el-step title="服务器地址" />
        <el-step title="用户名密码">
        </el-step>
        <el-step title="完成">
        </el-step>
    </el-steps>
    <div v-if="stepActive == 1">
        <el-input v-model="tmpEmbyServerConfig.base_url" style="width: 240px" placeholder="Please input" />
        <el-button :loading="addEmbyServerAddrLoading" @click="addEmbyServerAddr">下一步</el-button>
    </div>
    <div v-if="stepActive == 2">
        <el-input v-model="tmpEmbyServerConfig.server_name" style="width: 240px" placeholder="Please input" />
        <el-input v-model="tmpEmbyServerConfig.username" style="width: 240px" placeholder="Please input" />
        <el-input v-model="tmpEmbyServerConfig.password" style="width: 240px" placeholder="Please input" />
        <el-button :loading="addEmbyServerAuthLoading" @click="addEmbyServerPrevStep">上一步</el-button>
        <el-button :loading="addEmbyServerAuthLoading" @click="addEmbyServerAuth">下一步</el-button>
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
    <el-input v-model="tmpEmbyServerConfig.base_url" style="width: 240px" placeholder="Please input" />
    <el-input v-model="tmpEmbyServerConfig.server_name" style="width: 240px" placeholder="Please input" />
    <el-input v-model="tmpEmbyServerConfig.username" style="width: 240px" placeholder="Please input" />
    <el-input v-model="tmpEmbyServerConfig.password" style="width: 240px" placeholder="Please input" />
    <el-button @click="saveEditEmbyServer">保存</el-button>
    <el-button @click="dialogEditEmbyServerVisible = false">取消</el-button>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useRoute } from 'vue-router'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage, ElMessageBox } from "element-plus";
import { generateGuid } from "../util/uuid";
import { getOsInfo } from '../util/os'
import _ from "lodash";

const active = ref("");
const route = useRoute();
watchEffect(() => {
    active.value = route.path;
    console.log(active.value)
})

const embyServers = ref<EmbyServerConfig[]>([])
useConfig().get_config().then(config => {
    embyServers.value = config?.emby_server ? config.emby_server : []
})
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
const tmpEmbyServerConfig = ref<EmbyServerConfig>({})
function addEmbyServer() {
    stepActive.value = 1;
    dialogAddEmbyServerVisible.value = true
    const client = "loemby";
    const client_version = "0.1.0";
    const user_agent = client + "/" + client_version;
    tmpEmbyServerConfig.value = {
        id: generateGuid(),
        server_name: '未完成',
        disabled: true,
        user_agent: user_agent,
        client: client,
        client_version: client_version,
        device: getOsInfo().name,
        device_id: generateGuid(),
    }
}
const dialogEditEmbyServerVisible = ref(false)
function editEmbyServer(embyServer: EmbyServerConfig) {
    dialogEditEmbyServerVisible.value = true
    tmpEmbyServerConfig.value = _.clone(embyServer)
}
async function enabledEmbyServer(embyServer: EmbyServerConfig) {
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
    if (!tmpEmbyServerConfig || !tmpEmbyServerConfig.value?.base_url) {
        return
    }
    await saveEmbyServer(tmpEmbyServerConfig.value);
    embyApi.getServerInfo(tmpEmbyServerConfig.value).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: {ServerName: string, Id: string} = await response.json();
        tmpEmbyServerConfig.value!.server_name = json['ServerName']
        tmpEmbyServerConfig.value!.server_id = json['Id']
        await saveEmbyServer(tmpEmbyServerConfig.value);
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
    if (!tmpEmbyServerConfig || !tmpEmbyServerConfig.value?.username) {
        return
    }
    login(tmpEmbyServerConfig.value).then(() => {
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
        ElMessage.info({
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
        if (response.status != 200) {
            return Promise.reject('response status' + response.status + ' ' + response.statusText)
        }
        let json: {User: {Id: string}, AccessToken: string} = await response.json();
        embyServerConfig.auth_token = json['AccessToken']
        embyServerConfig.user_id = json["User"]['Id']
        embyServerConfig.disabled = false
        await saveEmbyServer(embyServerConfig);
    })
}
async function saveEditEmbyServer() {
    await saveEmbyServer(tmpEmbyServerConfig.value!);
    dialogEditEmbyServerVisible.value = false
}
</script>

<style scoped>
.el-menu {
    width: 100%;
    height: 100vh;
    background-color: var(--dark-background-color);
}
</style>