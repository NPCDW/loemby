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
                <el-menu-item :index="'/nav/emby/' + embyServer.id" v-if="embyServers" v-for="embyServer in embyServers" :disabled="embyServer.disabled">
                    <el-dropdown trigger="contextmenu" style="height: 100%; width: 100%;">
                        <div style="height: 100%; width: 100%; display: flex; align-items: center;">
                            <el-icon v-if="embyServer.disabled" style="color: #909399;"><i-ep-CircleCloseFilled /></el-icon>
                            <!-- <el-icon v-else-if="embyServer.request_status" class="is-loading" style="color: #409EFF;"><i-ep-Loading /></el-icon>
                            <el-icon v-else-if="embyServer.request_fail" style="color: #E6A23C;"><i-ep-WarningFilled /></el-icon> -->
                            <el-icon v-else size="24"><svg-icon name="emby" /></el-icon>
                            {{ embyServer.server_name }}
                        </div>
                        <template #dropdown>
                            <el-dropdown-menu split-button>
                                <el-dropdown-item @click="reLogin(embyServer)"><i-ep-Promotion /> 重新登录</el-dropdown-item>
                                <el-dropdown-item @click="editEmbyServer(embyServer)"><i-ep-Edit />编辑</el-dropdown-item>
                                <el-dropdown-item @click="enabledEmbyServer(embyServer)">
                                    <template v-if="embyServer.disabled">
                                        <i-ep-CircleCheckFilled /> 启用
                                    </template>
                                    <template v-else>
                                        <i-ep-CircleCloseFilled /> 禁用
                                    </template>
                                </el-dropdown-item>
                                <el-dropdown-item style="color: #E6A23C" @click="logoutEmbyServer(embyServer)"><i-ep-WarnTriangleFilled /> 退出登录</el-dropdown-item>
                                <el-dropdown-item style="color: #F56C6C" @click="delEmbyServer(embyServer)"><i-ep-Delete /> 删除</el-dropdown-item>
                            </el-dropdown-menu>
                        </template>
                    </el-dropdown>
                </el-menu-item>
            </el-menu>
        </el-scrollbar>
        <div style="flex: auto; height: 100vh; width: 100%;">
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
                <el-input v-model="tmpEmbyServerConfig.base_url" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="服务器地址">
                <el-select v-model="tmpEmbyServerConfig.browse_proxy_id">
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
                <el-input v-model="tmpEmbyServerConfig.server_name" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="用户名">
                <el-input v-model="tmpEmbyServerConfig.username" placeholder="Please input" />
            </el-form-item>
            <el-form-item label="密码">
                <el-input v-model="tmpEmbyServerConfig.password" placeholder="Please input" show-password />
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
            <el-input v-model="tmpEmbyServerConfig.base_url" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="服务器名称">
            <el-input v-model="tmpEmbyServerConfig.server_name" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="用户名">
            <el-input v-model="tmpEmbyServerConfig.username" placeholder="Please input" />
        </el-form-item>
        <el-form-item label="密码">
            <el-input v-model="tmpEmbyServerConfig.password" placeholder="Please input" show-password />
        </el-form-item>
        <el-form-item>
            <div style="width: 100%; display: flex; justify-content: center;">
                <el-button type="primary" @click="saveEditEmbyServer">保存</el-button>
                <el-button @click="dialogEditEmbyServerVisible = false">取消</el-button>
            </div>
        </el-form-item>
    </el-form>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useRoute } from 'vue-router'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage, ElMessageBox } from "element-plus";
import { generateGuid } from "../util/uuid_util";
import { getOsInfo } from '../util/os_util'
import _ from "lodash";

const active = ref("");
const route = useRoute();
watchEffect(() => {
    active.value = route.path;
    console.log(active.value)
})

let config = useConfig().get_config()
const proxyServers = config.proxy_server ? config.proxy_server : [];
const embyServers = config?.emby_server ? config.emby_server : []
async function saveEmbyServer(tmp: EmbyServerConfig) {
    let value = _.cloneDeep(tmp);
    for (let index = 0; index < embyServers.length; index++) {
        if (embyServers[index].id === value.id) {
            embyServers[index] = value
            await useConfig().saveEmbyServer(embyServers)
            return
        }
    }
    embyServers.push(value)
    await useConfig().saveEmbyServer(embyServers)
}


const stepActive = ref(1)
const dialogAddEmbyServerVisible = ref(false)
const tmpEmbyServerConfig = ref<EmbyServerConfig>({})
function addEmbyServer() {
    stepActive.value = 1;
    dialogAddEmbyServerVisible.value = true
    const client = "loemby";
    const client_version = "0.4.1";
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
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow',
    }
}
const dialogEditEmbyServerVisible = ref(false)
function editEmbyServer(embyServer: EmbyServerConfig) {
    dialogEditEmbyServerVisible.value = true
    tmpEmbyServerConfig.value = _.clone(embyServer)
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
        for (let index = 0; index < embyServers.length; index++) {
            if (embyServers[index].id === tmp.id) {
                embyServers.splice(index, 1)
                await useConfig().saveEmbyServer(embyServers)
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
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: {ServerName: string, Id: string} = JSON.parse(response.body);
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