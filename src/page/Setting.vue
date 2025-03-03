<template>
    <div style="padding: 20px;">
        <div>
            <h1>代理服务器</h1>
            <el-table :data="proxyServerTableData" style="width: 100%">
                <el-table-column prop="name" label="Name" width="140" show-overflow-tooltip />
                <el-table-column prop="proxy_type" label="Type" width="80" />
                <el-table-column prop="addr" label="Address" width="160" show-overflow-tooltip />
                <el-table-column prop="username" label="Username" width="140" />
                <el-table-column prop="location" label="Location" show-overflow-tooltip />
                <el-table-column fixed="right" label="Operations" width="160">
                    <template #header>
                        <el-button type="primary" size="small" @click.prevent="addProxy()">添加代理服务器</el-button>
                    </template>
                    <template #default="scope">
                        <el-button plain :loading="checkProxyLoading[scope.row.id]" type="success" size="small" @click.prevent="checkProxy(scope.row.id)">检测</el-button>
                        <el-button plain type="primary" size="small" @click.prevent="editProxy(scope.$index)">编辑</el-button>
                    </template>
                </el-table-column>
            </el-table>
        </div>
        <div>
            <h1>Emby服务代理配置</h1>
            <el-table :data="embyServerTableData" style="width: 100%">
                <el-table-column prop="server_name" label="Emby" show-overflow-tooltip />
                <el-table-column label="媒体库浏览">
                    <template #default="scope">
                        <el-select v-model="scope.row.browse_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" label="跟随全局代理" value="follow"/>
                            <el-option v-for="proxyServer in proxyServerTableData" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </template>
                </el-table-column>
                <el-table-column label="媒体流播放">
                    <template #default="scope">
                        <el-select v-model="scope.row.play_proxy_id">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="follow" label="跟随全局代理" value="follow"/>
                            <el-option v-for="proxyServer in proxyServerTableData" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </template>
                </el-table-column>
            </el-table>
            <div>
                <el-button type="primary" @click="saveEmbyProxy()">保存修改</el-button>
            </div>
        </div>
    </div>

    <el-dialog
        v-model="dialogProxyServerVisible"
        title="添加代理服务器"
        width="800"
    >
        <el-scrollbar>
            <el-form label-position="top">
                <el-form-item label="代理名称">
                    <el-input v-model="dialogProxyServer.name" placeholder="代理名称" />
                </el-form-item>
                <el-form-item label="代理类型">
                    <el-select v-model="dialogProxyServer.proxy_type">
                        <el-option key="socks5" label="socks5" value="socks5"/>
                        <el-option key="socks" label="socks4" value="socks"/>
                        <el-option key="https" label="https" value="https"/>
                        <el-option key="http" label="http" value="http"/>
                    </el-select>
                </el-form-item>
                <el-form-item label="服务器地址">
                    <el-input v-model="dialogProxyServer.addr" placeholder="服务器地址（ ip:port ）" />
                </el-form-item>
                <el-form-item label="用户名">
                    <el-input v-model="dialogProxyServer.username" placeholder="用户名（可选）" />
                </el-form-item>
                <el-form-item label="密码">
                    <el-input v-model="dialogProxyServer.password" placeholder="密码（可选）" />
                </el-form-item>
                <el-form-item>
                    <div style="width: 100%; display: flex; justify-content: end;">
                        <el-button @click="saveProxyServer" type="primary">保存</el-button>
                        <el-button @click="dialogProxyServerVisible = false">取消</el-button>
                    </div>
                </el-form-item>
            </el-form>
        </el-scrollbar>
    </el-dialog>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { useConfig } from '../store/config';
import { ElMessage } from 'element-plus';
import { ProxyServerConfig } from '../store/config';
import _ from 'lodash';
import { generateGuid } from '../util/uuid_util';
import appApi from '../api/appApi';

const config = useConfig().get_config();
const proxyServerTableData = config.proxy_server ? config.proxy_server : [];

const dialogProxyServerVisible = ref(false);
const dialogProxyServer = ref<ProxyServerConfig>({})

function addProxy() {
    dialogProxyServerVisible.value = true;
    dialogProxyServer.value = {};
}
function editProxy(index: number) {
    dialogProxyServerVisible.value = true;
    dialogProxyServer.value = _.clone(proxyServerTableData[index]);
}
function saveProxyServer() {
    if (dialogProxyServer.value.id) {
        let index = proxyServerTableData.findIndex((item) => item.id == dialogProxyServer.value.id);
        proxyServerTableData[index] = dialogProxyServer.value;
    } else {
        dialogProxyServer.value.id = generateGuid();
        proxyServerTableData.push(dialogProxyServer.value);
    }
    config.proxy_server = proxyServerTableData;
    useConfig().save_config(config);
    dialogProxyServerVisible.value = false;
    ElMessage.success('保存成功');
}

const checkProxyLoading = ref<{[key: string]: boolean}>({});
function checkProxy(id: string) {
    checkProxyLoading.value[id] = true;
    appApi.getProxyLocation(id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json = JSON.parse(response.body);
        for (let index = 0; index < proxyServerTableData.length; index++) {
            if (proxyServerTableData[index].id === id) {
                proxyServerTableData[index].location = json["ip"] + " " + json["country"]["code"];
            }
        }
    }).catch(e => {
        ElMessage.error({
            message: e + '    可能是代理配置错误，请检查代理配置'
        })
    }).finally(() => checkProxyLoading.value[id] = false);
}

const embyServerTableData = config.emby_server ? config.emby_server : [];
const global_browse_proxy_id = config.global_proxy?.browse_proxy_id ? config.global_proxy?.browse_proxy_id : 'no';
const global_play_proxy_id = config.global_proxy?.play_proxy_id ? config.global_proxy?.play_proxy_id : 'no';
embyServerTableData.splice(0, 0, {server_name: '全局配置', browse_proxy_id: global_browse_proxy_id, play_proxy_id: global_play_proxy_id});
function saveEmbyProxy() {
    const global_proxy = embyServerTableData.shift();
    config.global_proxy = {
        browse_proxy_id: global_proxy?.browse_proxy_id,
        play_proxy_id: global_proxy?.play_proxy_id
    };
    config.emby_server = embyServerTableData;
    useConfig().save_config(config);
}
</script>

<style scoped>
</style>