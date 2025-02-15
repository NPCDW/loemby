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
                </el-menu-item>
            </el-menu>
        </div>
        <div style="flex: auto;">
            <router-view></router-view>
        </div>
    </div>
  <el-dialog v-model="dialogAddEmbyServerVisible" title="Emby Server" width="800">
    <el-steps style="max-width: 600px" :active="2" align-center>
        <el-step title="服务器地址">
            <el-inputt v-model="tmpEmbyServerConfig?.base_url" style="width: 240px" placeholder="Please input" />
            <el-button :loading="addEmbyServerAddrLoading" @click="addEmbyServerAddr">下一步</el-button>
        </el-step>
        <el-step title="用户名密码">
            <el-inputt v-model="tmpEmbyServerConfig?.server_name" style="width: 240px" placeholder="Please input" />
            <el-inputt v-model="tmpEmbyServerConfig?.username" style="width: 240px" placeholder="Please input" />
            <el-inputt v-model="tmpEmbyServerConfig?.password" style="width: 240px" placeholder="Please input" />
            <el-button>下一步</el-button>
        </el-step>
        <el-step title="完成">
            <el-result
                icon="success"
                title="Success Tip"
                sub-title="Please follow the instructions"
            >
                <template #extra>
                <el-button type="primary">Back</el-button>
                </template>
            </el-result>
        </el-step>
    </el-steps>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useRoute } from 'vue-router'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage } from "element-plus";

const active = ref("");
const route = useRoute();
watchEffect(() => {
    active.value = route.path;
    console.log(active.value)
})

const embyServers = ref<EmbyServerConfig[]>()
useConfig().get_config().then(config => {
    embyServers.value = config?.emby_server
})

const dialogAddEmbyServerVisible = ref(false)
const tmpEmbyServerConfig = ref<EmbyServerConfig>()
function addEmbyServer() {
    dialogAddEmbyServerVisible.value = true
    tmpEmbyServerConfig.value = {
        id: '',
        base_url: '',
        disabled: true
    }
}

const addEmbyServerAddrLoading = ref(false)
function addEmbyServerAddr() {
    addEmbyServerAddrLoading.value = true
    if (!tmpEmbyServerConfig || !tmpEmbyServerConfig.value?.base_url) {
        return
    }
    embyApi.getServerInfo(tmpEmbyServerConfig.value.base_url).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: {ServerName: string, Id: string} = await response.json();
        tmpEmbyServerConfig.value!.server_name = json['ServerName']
    }).catch(e => {
        ElMessage.error({
            message: e
        })
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