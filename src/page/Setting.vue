<template>
    <el-tabs v-model="activePane" @tab-change="handlePaneChange" style="height: calc(100vh - 10px); padding: 10px 20px 0 20px;">
        <el-tab-pane label="常规" name="Common">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <el-form label-position="top">
                    <el-form-item label="MPV路径">
                        <el-input v-model="mpv_path" @change="mpvPathChange" placeholder="示例: C:\App\mpv_config-2024.12.04\mpv.exe" />
                    </el-form-item>
                </el-form>
            </el-scrollbar>
        </el-tab-pane>
        <el-tab-pane label="代理服务器" name="ProxyServer">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <h1>代理服务器</h1>
                <el-table :data="proxyServer" style="width: 100%">
                    <el-table-column prop="name" label="Name" width="140" show-overflow-tooltip />
                    <el-table-column prop="proxy_type" label="Type" width="80" />
                    <el-table-column prop="addr" label="Address" width="160" show-overflow-tooltip />
                    <el-table-column prop="username" label="Username" width="140" />
                    <el-table-column prop="location" label="Location" show-overflow-tooltip />
                    <el-table-column fixed="right" label="Operations" width="210">
                        <template #header>
                            <el-button type="primary" size="small" @click.prevent="addProxy()">添加代理服务器</el-button>
                        </template>
                        <template #default="scope">
                            <el-button plain :loading="checkProxyLoading[scope.row.id]" type="success" size="small" @click.prevent="checkProxy(scope.row.id)">检测</el-button>
                            <el-button plain type="primary" size="small" @click.prevent="editProxy(scope.$index)">编辑</el-button>
                            <el-button plain type="danger" size="small" @click.prevent="delProxy(scope.$index)">删除</el-button>
                        </template>
                    </el-table-column>
                </el-table>
            </el-scrollbar>
        </el-tab-pane>
        <el-tab-pane label="Emby线路代理" name="EmbyLineProxy">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <h1>Emby线路代理配置</h1>
                <el-table :data="embyLines" style="width: 100%" :span-method="lineSpanMethod">
                    <el-table-column prop="emby_server_name" label="Emby" show-overflow-tooltip />
                    <el-table-column prop="name" label="线路" show-overflow-tooltip />
                    <el-table-column label="媒体库浏览">
                        <template #header="">
                            <span>媒体库浏览</span><br/>
                            <el-select v-model="global_browse_proxy_id" @change="globalBrowseProxyChange">
                                <template #label="{ label }">
                                    <span>全局配置: </span>
                                    <span style="font-weight: bold">{{ label }}</span>
                                </template>
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </template>
                        <template #default="scope">
                            <el-select v-model="scope.row.browse_proxy_id" @change="proxyChange(scope.row)">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="follow" label="跟随全局代理" value="follow"/>
                                <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </template>
                    </el-table-column>
                    <el-table-column label="媒体流播放">
                        <template #header="">
                            <span>媒体流播放</span><br/>
                            <el-select v-model="global_play_proxy_id" @change="globalPlayProxyChange">
                                <template #label="{ label }">
                                    <span>全局配置: </span>
                                    <span style="font-weight: bold">{{ label }}</span>
                                </template>
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </template>
                        <template #default="scope">
                            <el-select v-model="scope.row.play_proxy_id" @change="proxyChange(scope.row)">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="follow" label="跟随全局代理" value="follow"/>
                                <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </template>
                    </el-table-column>
                </el-table>
            </el-scrollbar>
        </el-tab-pane>
    </el-tabs>

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
import { computed, onUnmounted, ref } from 'vue';
import { ElMessage, ElMessageBox, TableColumnCtx } from 'element-plus';
import { ProxyServer, useProxyServer } from '../store/db/proxyServer';
import _ from 'lodash';
import { generateGuid } from '../util/uuid_util';
import appApi from '../api/appApi';
import { EmbyServer, useEmbyServer } from '../store/db/embyServer';
import { EmbyLine, useEmbyLine } from '../store/db/embyLine';
import { useGlobalConfig } from '../store/db/globalConfig';
import { useEventBus } from '../store/eventBus';

const proxyServer = ref<ProxyServer[]>([]);
function listAllProxyServer() {
    useProxyServer().listAllProxyServer().then(list => {
        proxyServer.value = list;
    })
}
useEventBus().on('ProxyServerListChanged', listAllProxyServer)
onUnmounted(() => {
    useEventBus().remove('ProxyServerListChanged', listAllProxyServer)
})

const dialogProxyServerVisible = ref(false);
const dialogProxyServer = ref<ProxyServer>({})

function addProxy() {
    dialogProxyServerVisible.value = true;
    dialogProxyServer.value = {};
}
function editProxy(index: number) {
    dialogProxyServerVisible.value = true;
    dialogProxyServer.value = _.clone(proxyServer.value[index]);
}
function saveProxyServer() {
    let savePromise;
    if (dialogProxyServer.value.id) {
        savePromise = useProxyServer().updateProxyServer(dialogProxyServer.value)
    } else {
        dialogProxyServer.value.id = generateGuid();
        savePromise = useProxyServer().addProxyServer(dialogProxyServer.value)
    }
    savePromise.then(() => {
        useEventBus().emit('ProxyServerListChanged', {})
        ElMessage.success('保存成功');
    }).catch(e => {
        ElMessage.error('保存失败' + e);
    }).finally(() => dialogProxyServerVisible.value = false)
}
function delProxy(index: number) {
    ElMessageBox.confirm(
    `确认删除代理服务器「${proxyServer.value[index].name}」吗`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useProxyServer().delProxyServer(proxyServer.value[index].id!).then(() => {
            useEventBus().emit('ProxyServerListChanged', {})
            ElMessage.success('删除成功');
        }).catch(e => ElMessage.error('删除失败' + e))
  })
}

const checkProxyLoading = ref<{[key: string]: boolean}>({});
function checkProxy(id: string) {
    checkProxyLoading.value[id] = true;
    appApi.getProxyLocation(id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error('response status' + response.status_code + ' ' + response.status_text)
            return
        }
        let json = JSON.parse(response.body);
        for (let index = 0; index < proxyServer.value.length; index++) {
            if (proxyServer.value[index].id === id) {
                proxyServer.value[index].location = json["ip"] + " " + json["country"]["code"];
            }
        }
    }).catch(e => {
        ElMessage.error('检测代理失败，可能是代理配置错误，请检查代理配置' + e)
    }).finally(() => checkProxyLoading.value[id] = false);
}

const embyServers = ref<EmbyServer[]>([])
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
listAllEmbyServer()
useEventBus().on('EmbyServerListChanged', listAllEmbyServer)
onUnmounted(() => {
    useEventBus().remove('EmbyServerListChanged', listAllEmbyServer)
})

const embyLinesOrigin = ref<EmbyLine[]>([]);
const embyLines = computed(() => {
    const embyServersSort = embyServers.value.map(item=> item.id)
    embyLinesOrigin.value.sort((a,b) => embyServersSort.indexOf(a.emby_server_id) - embyServersSort.indexOf(b.emby_server_id))
    return embyLinesOrigin.value
})
function listAllEmbyLine() {
    useEmbyLine().listAllEmbyLine().then(list => {
        embyLinesOrigin.value = list
    })
}
useEventBus().on('EmbyLineListChanged', listAllEmbyLine)
onUnmounted(() => {
    useEventBus().remove('EmbyLineListChanged', listAllEmbyLine)
})

interface SpanMethodProps {
  row: typeof embyLines.value[0]
  column: TableColumnCtx<typeof embyLines.value[0]>
  rowIndex: number
  columnIndex: number
}
const lineSpanMethod = ({row, rowIndex, columnIndex}: SpanMethodProps) => {
  if (columnIndex === 0) {
    if (rowIndex !== 0 && row.emby_server_id === embyLines.value[rowIndex - 1].emby_server_id) {
        return {
            rowspan: 0,
            colspan: 0,
        }
    } else {
        let rowspan = 1;
        for (let i = rowIndex + 1; i < embyLines.value.length; i++) {
            if (embyLines.value[i].emby_server_id !== row.emby_server_id) {
                break;
            }
            rowspan++;
        }
        return {
            rowspan: rowspan,
            colspan: 1,
        }
    }
  }
}

const global_browse_proxy_id = ref<string>('no');
function getGlobalBrowseProxy() {
    useGlobalConfig().getGlobalConfigValue("global_browse_proxy_id").then(value => {
        global_browse_proxy_id.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取全局浏览代理失败' + e))
}
const global_play_proxy_id = ref<string>('no');
function getGlobalPlayProxy() {
    useGlobalConfig().getGlobalConfigValue("global_play_proxy_id").then(value => {
        global_play_proxy_id.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取全局播放代理失败' + e))
}
function globalBrowseProxyChange() {
    useGlobalConfig().getGlobalConfig("global_browse_proxy_id").then(config => {
        let savePromise;
        if (config) {
            config.config_value = global_browse_proxy_id.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "global_browse_proxy_id",
                config_value: global_browse_proxy_id.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getGlobalBrowseProxy()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改全局浏览代理失败' + e))
    ElMessage.success('修改成功');
}
function globalPlayProxyChange() {
    useGlobalConfig().getGlobalConfig("global_play_proxy_id").then(config => {
        let savePromise;
        if (config) {
            config.config_value = global_play_proxy_id.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "global_play_proxy_id",
                config_value: global_play_proxy_id.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getGlobalPlayProxy()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改全局浏览代理失败' + e))
    ElMessage.success('修改成功');
}
function proxyChange(line: EmbyLine) {
    useEmbyLine().updateEmbyLine(line).then(() => {
        useEventBus().emit('EmbyLineListChanged', {})
        if (line.in_use) {
            useEmbyServer().updateEmbyServer({
                id: line.emby_server_id,
                browse_proxy_id: line.browse_proxy_id,
                play_proxy_id: line.play_proxy_id
            }).then(() => {
                useEventBus().emit('EmbyServerListChanged', {})
            }).catch(e => ElMessage.error('修改失败' + e));
        }
        ElMessage.success('修改成功');
    }).catch(e => ElMessage.error('修改失败' + e));
}

const mpv_path = ref<string>('');
function getMpvPath() {
    useGlobalConfig().getGlobalConfigValue("mpv_path").then(value => {
        mpv_path.value = value ? value : "";
    }).catch(e => ElMessage.error('获取MPV路径失败' + e))
}
function mpvPathChange() {
    useGlobalConfig().getGlobalConfig("mpv_path").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_path.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_path",
                config_value: mpv_path.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvPath()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV路径失败' + e))
}

const activePane = ref('Common')
function handlePaneChange() {
    if (activePane.value == 'Common') {
        getMpvPath()
    } else if (activePane.value == 'ProxyServer') {
        listAllProxyServer()
    } else if (activePane.value == 'EmbyLineProxy') {
        if (proxyServer.value.length == 0) {
            listAllProxyServer()
        }
        listAllEmbyLine()
        getGlobalBrowseProxy()
        getGlobalPlayProxy()
    }
}
getMpvPath()
</script>

<style scoped>
</style>