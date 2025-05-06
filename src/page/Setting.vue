<template>
    <el-tabs v-model="activePane" @tab-change="handlePaneChange" style="height: calc(100vh - 10px); padding: 10px 20px 0 20px;">
        <el-tab-pane label="常规" name="Common">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <el-form label-position="top">
                    <el-form-item label="应用更新">
                        <span style="margin-right: 10px;">当前版本: {{ version }}</span>
                        <el-button type="primary" size="small" :loading="checkUpdateLoading" @click="checkUpdate()">检查更新</el-button>
                    </el-form-item>
                </el-form>
            </el-scrollbar>
        </el-tab-pane>
        
        <el-tab-pane label="MPV" name="MPV">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <el-form label-position="top">
                    <el-form-item label="MPV文件路径">
                        <el-input v-model="mpv_path" @change="mpvPathChange" placeholder="示例: C:\App\mpv_config-2024.12.04\mpv.exe 或 /usr/bin/mpv" />
                    </el-form-item>
                    <el-form-item label="MPV启动目录">
                        <el-input v-model="mpv_startup_dir" @change="mpvStartupDirChange" placeholder="示例: C:\App\mpv_config-2024.12.04 留空默认为 mpv 所在目录" />
                    </el-form-item>
                    <el-form-item label="MPV缓存（按秒计算缓存大小，平均码率除以8再乘以秒即为实际缓存大小，如果大于最大缓存大小，则按最大缓存大小）">
                        <el-input-number v-model="mpv_cache_seconds" @change="mpvCacheSecondsChange" :min="1" :precision="0" :controls="false">
                            <template #prefix>
                                <span>前向缓存</span>
                            </template>
                            <template #suffix>
                                <span>秒</span>
                            </template>
                        </el-input-number>
                        <el-input-number v-model="mpv_cache_max_bytes" @change="mpvCacheMaxBytesChange" :min="1" :precision="0" :controls="false">
                            <template #prefix>
                                <span>前向最大缓存</span>
                            </template>
                            <template #suffix>
                                <span>MiB</span>
                            </template>
                        </el-input-number>
                        <el-input-number v-model="mpv_cache_back_seconds" @change="mpvCacheBackSecondsChange" :min="1" :precision="0" :controls="false">
                            <template #prefix>
                                <span>后向缓存</span>
                            </template>
                            <template #suffix>
                                <span>秒</span>
                            </template>
                        </el-input-number>
                        <el-input-number v-model="mpv_cache_back_max_bytes" @change="mpvCacheBackMaxBytesChange" :min="1" :precision="0" :controls="false">
                            <template #prefix>
                                <span>后向最大缓存</span>
                            </template>
                            <template #suffix>
                                <span>MiB</span>
                            </template>
                        </el-input-number>
                    </el-form-item>
                    <el-form-item>
                        <template #label>
                            <div style="display: flex; align-items: center;">
                                <span>MPV参数</span>
                                <el-link target="_blank" href="https://mpv.io/manual/stable/" style="margin-left: 10px;">官方文档</el-link>
                                <el-link target="_blank" href="https://hooke007.github.io/official_man/mpv.html" style="margin-left: 10px;">中文文档</el-link>
                            </div>
                        </template>
                        <el-input v-model="mpv_args" @change="mpvArgsChange" :rows="4" type="textarea" placeholder="每行一个，示例: 
ontop=no
demuxer-max-bytes=512MiB
demuxer-max-back-bytes=512MiB
demuxer-readahead-secs=180" />
                    </el-form-item>
                </el-form>
            </el-scrollbar>
        </el-tab-pane>
        
        <el-tab-pane label="Trakt" name="Trakt">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <el-form label-position="top">
                    <el-form-item label="Trakt （剧集或电影播放完成时可以在网页端看到记录，未播放完成的可以通过接口查询记录）">
                        <div v-if="trakt_info.username">
                            <el-text>{{ trakt_info.username }}</el-text>
                            <el-switch v-model="trakt_sync_switch" @change="traktSyncSwitchChange" active-value="on" inactive-value="off" inline-prompt style="margin-left: 10px; --el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="同步已开启" inactive-text="同步已关闭" />
                            <el-button type="danger" @click="delAuthTrakt()" size="small" style="margin: 0 10px;">删除授权</el-button>
                        </div>
                        <el-button type="primary" :loading="traktAuthLoading" @click="goAuthTrakt()" size="small">{{ traktAuthStatus }}</el-button>
                    </el-form-item>
                    <el-form-item label="Trakt代理">
                        <el-select v-model="trakt_proxy_id" @change="traktProxyChange" style="width: 220px;">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="followBrowse" label="跟随全局媒体库浏览代理" value="followBrowse"/>
                            <el-option key="followPlay" label="跟随全局媒体流播放代理" value="followPlay"/>
                            <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
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
                <el-form :inline="true">
                    <el-form-item label="全局媒体库浏览">
                        <el-select v-model="global_browse_proxy_id" @change="globalBrowseProxyChange" style="width: 220px;">
                            <template #label="{ label }">
                                <span>全局配置: </span>
                                <span style="font-weight: bold">{{ label }}</span>
                            </template>
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                    <el-form-item label="全局媒体流播放">
                        <el-select v-model="global_play_proxy_id" @change="globalPlayProxyChange" style="width: 220px;">
                            <template #label="{ label }">
                                <span>全局配置: </span>
                                <span style="font-weight: bold">{{ label }}</span>
                            </template>
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                </el-form>
                <el-table :data="embyLines" style="width: 100%" :span-method="lineSpanMethod">
                    <el-table-column prop="emby_server_name" label="Emby" show-overflow-tooltip />
                    <el-table-column prop="name" label="线路" show-overflow-tooltip />
                    <el-table-column label="媒体库浏览">
                        <template #default="scope">
                            <el-select v-model="scope.row.browse_proxy_id" @change="proxyChange(scope.row)">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="follow" label="跟随全局代理" value="follow"/>
                                <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </template>
                    </el-table-column>
                    <el-table-column label="媒体流播放">
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
        <el-tab-pane label="Emby图标库" name="EmbyIconLibrary">
            <el-scrollbar style="height: calc(100vh - 100px);">
                <h1>Emby图标库</h1>
                <el-form :inline="true">
                    <el-form-item label="应用数据代理（图标、自动更新等）">
                        <el-select v-model="app_proxy_id" @change="appProxyChange" style="width: 220px;">
                            <el-option key="no" label="不使用代理" value="no"/>
                            <el-option key="followBrowse" label="跟随全局媒体库浏览代理" value="followBrowse"/>
                            <el-option key="followPlay" label="跟随全局媒体流播放代理" value="followPlay"/>
                            <el-option v-for="proxyServer in proxyServer" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                        </el-select>
                    </el-form-item>
                </el-form>
                <el-table :data="embyIconLibrary" style="width: 100%">
                    <el-table-column prop="name" label="Name" width="140" show-overflow-tooltip />
                    <el-table-column prop="url" label="Url" show-overflow-tooltip />
                    <el-table-column fixed="right" label="Operations" width="210">
                        <template #header>
                            <el-button type="primary" size="small" @click.prevent="addEmbyIconLibrary()">添加图标库</el-button>
                        </template>
                        <template #default="scope">
                            <el-button plain type="primary" size="small" @click.prevent="editEmbyIconLibrary(scope.$index)">编辑</el-button>
                            <el-button plain type="danger" size="small" @click.prevent="delEmbyIconLibrary(scope.$index)">删除</el-button>
                        </template>
                    </el-table-column>
                </el-table>
            </el-scrollbar>
        </el-tab-pane>
    </el-tabs>

    <el-dialog
        v-model="dialogProxyServerVisible"
        title="代理服务器"
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
    <el-dialog
        v-model="dialogEmbyIconLibraryVisible"
        title="Emby图标库"
        width="800"
    >
        <el-scrollbar>
            <el-form label-position="top">
                <el-form-item label="名称">
                    <el-input v-model="dialogEmbyIconLibrary.name" placeholder="图标库名称" />
                </el-form-item>
                <el-form-item label="地址">
                    <el-input v-model="dialogEmbyIconLibrary.url" placeholder="图标库 http 地址" />
                </el-form-item>
                <el-form-item>
                    <div style="width: 100%; display: flex; justify-content: end;">
                        <el-button @click="saveEmbyIconLibrary" type="primary">保存</el-button>
                        <el-button @click="dialogEmbyIconLibraryVisible = false">取消</el-button>
                    </div>
                </el-form-item>
            </el-form>
        </el-scrollbar>
    </el-dialog>
</template>

<script lang="ts" setup>
import { computed, h, onMounted, onUnmounted, ref } from 'vue';
import { ElButton, ElMessage, ElMessageBox, ElNotification, TableColumnCtx } from 'element-plus';
import { ProxyServer, useProxyServer } from '../store/db/proxyServer';
import _ from 'lodash';
import { generateGuid } from '../util/uuid_util';
import appApi from '../api/appApi';
import { EmbyServer, useEmbyServer } from '../store/db/embyServer';
import { EmbyLine, useEmbyLine } from '../store/db/embyLine';
import { useGlobalConfig } from '../store/db/globalConfig';
import { useEventBus } from '../store/eventBus';
import invokeApi from '../api/invokeApi';
import { listen } from '@tauri-apps/api/event';
import traktApi from '../api/traktApi';
import {useRuntimeConfig} from "../store/runtimeConfig.ts";
import { EmbyIconLibrary, useEmbyIconLibrary } from '../store/db/embyIconLibrary.ts';

const version = useRuntimeConfig().runtimeConfig?.version

const proxyServer = ref<ProxyServer[]>([]);
function listAllProxyServer() {
    useProxyServer().listAllProxyServer().then(list => {
        proxyServer.value = list;
    })
}
listAllProxyServer()
onMounted(() => useEventBus().on('ProxyServerChanged', listAllProxyServer))
onUnmounted(() => useEventBus().remove('ProxyServerChanged', listAllProxyServer))

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
        useEventBus().emit('ProxyServerChanged', {})
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
            useEventBus().emit('ProxyServerChanged', {})
            ElMessage.success('删除成功');
        }).catch(e => ElMessage.error('删除失败' + e))
  })
}

const dialogEmbyIconLibraryVisible = ref(false);
const dialogEmbyIconLibrary = ref<EmbyIconLibrary>({})
const embyIconLibrary = ref<EmbyIconLibrary[]>([]);
function listAllEmbyIconLibrary() {
    useEmbyIconLibrary().listAllEmbyIconLibrary().then(list => {
        embyIconLibrary.value = list;
    })
}
function addEmbyIconLibrary() {
    dialogEmbyIconLibraryVisible.value = true;
    dialogEmbyIconLibrary.value = {};
}
function editEmbyIconLibrary(index: number) {
    dialogEmbyIconLibraryVisible.value = true;
    dialogEmbyIconLibrary.value = _.clone(embyIconLibrary.value[index]);
}
function saveEmbyIconLibrary() {
    let savePromise;
    if (dialogEmbyIconLibrary.value.id) {
        savePromise = useEmbyIconLibrary().updateEmbyIconLibrary(dialogEmbyIconLibrary.value)
    } else {
        dialogEmbyIconLibrary.value.id = generateGuid();
        savePromise = useEmbyIconLibrary().addEmbyIconLibrary(dialogEmbyIconLibrary.value)
    }
    savePromise.then(() => {
        ElMessage.success('保存成功');
        listAllEmbyIconLibrary()
    }).catch(e => {
        ElMessage.error('保存失败' + e);
    }).finally(() => dialogEmbyIconLibraryVisible.value = false)
}
function delEmbyIconLibrary(index: number) {
    ElMessageBox.confirm(
    `确认删除图标库「${embyIconLibrary.value[index].name}」吗`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useEmbyIconLibrary().delEmbyIconLibrary(embyIconLibrary.value[index].id!).then(() => {
            ElMessage.success('删除成功');
            listAllEmbyIconLibrary()
        }).catch(e => ElMessage.error('删除失败' + e))
  })
}

const checkProxyLoading = ref<{[key: string]: boolean}>({});
function checkProxy(id: string) {
    checkProxyLoading.value[id] = true;
    appApi.getProxyLocation(id).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error(response.status_code + ' ' + response.status_text)
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

const checkUpdateLoading = ref<boolean>(false);
function checkUpdate() {
    checkUpdateLoading.value = true;
    invokeApi.updater().then(res => {
        if (res) {
            ElNotification.success({
                title: '新版本准备就绪',
                message: h('p', {'style': "display: flex; justify-content: space-between; align-items: center;"}, [
                  h('span', null, '重启应用生效'),
                  h(ElButton, {
                    'size': 'small',
                    'type': 'success',
                    onClick: () => {
                      invokeApi.restartApp()
                    },
                  }, "现在重启"),
                ]),
                position: 'bottom-right',
            })
        } else {
            ElMessage.success('已经是最新版本')
        }
    }).finally(() => checkUpdateLoading.value = false);
}

const embyServers = ref<EmbyServer[]>([])
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
listAllEmbyServer()
onMounted(() => useEventBus().on('EmbyServerChanged', listAllEmbyServer))
onUnmounted(() => useEventBus().remove('EmbyServerChanged', listAllEmbyServer))

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
onMounted(() => useEventBus().on('EmbyLineChanged', listAllEmbyLine))
onUnmounted(() => useEventBus().remove('EmbyLineChanged', listAllEmbyLine))

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

const trakt_sync_switch = ref("on")
function getTraktSyncSwitch() {
    return useGlobalConfig().getGlobalConfigValue("trakt_sync_switch").then(value => {
        trakt_sync_switch.value = value ? value : "on";
    }).catch(e => ElMessage.error('获取Trakt同步开关失败' + e))
}
function traktSyncSwitchChange() {
    useGlobalConfig().getGlobalConfig("trakt_sync_switch").then(config => {
        let savePromise;
        if (config) {
            config.config_value = trakt_sync_switch.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "trakt_sync_switch",
                config_value: trakt_sync_switch.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getTraktSyncSwitch()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改Trakt同步开关失败' + e))
}

const traktAuthLoading = ref(false)
const trakt_info = ref<{access_token?: string, refresh_token?: string, expires_in?: number, username?: string, redirect_uri?: string}>({});
const traktAuthStatus = ref('去授权')
function getTraktInfo() {
    return useGlobalConfig().getGlobalConfigValue("trakt_info").then(value => {
        trakt_info.value = value ? JSON.parse(value) : {};
        if (!traktAuthLoading.value) {
            traktAuthStatus.value = trakt_info.value.username ? '换个账户？授权失效？' : '去授权'
        }
    }).catch(e => ElMessage.error('获取Trakt信息失败' + e))
}
async function saveTraktInfo() {
    return useGlobalConfig().getGlobalConfig("trakt_info").then(config => {
        let savePromise;
        if (config) {
            config.config_value = JSON.stringify(trakt_info.value);
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "trakt_info",
                config_value: JSON.stringify(trakt_info.value)
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        return savePromise.then(() => {
            getTraktInfo()
        }).catch(e => {
            ElMessage.error('保存失败' + e);
        }).catch(e => ElMessage.error('保存Trakt信息失败' + e))
    }).catch(e => ElMessage.error('获取Trakt信息失败' + e))
}
function delAuthTrakt() {
  ElMessageBox.confirm(
    `确认删除 Trakt 授权吗？同时建议前往 Trakt 官网吊销应用授权，这将删除该应用获取的所有授权，官网地址: https://trakt.tv/oauth/authorized_applications`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useGlobalConfig().delGlobalConfig("trakt_info").then(() => {
            getTraktInfo()
            ElMessageBox.alert('删除成功，同时建议前往 Trakt 官网吊销应用授权，这将删除该应用获取的所有授权，官网地址: https://trakt.tv/oauth/authorized_applications')
        })
    })
}
function goAuthTrakt() {
    traktAuthLoading.value = true
    traktAuthStatus.value = '等待授权回调'
    invokeApi.go_trakt_auth().then(() => {
        ElMessage.success('打开浏览器成功，您也可以手动复制地址，去其他浏览器授权');
    }).catch(e => ElMessage.error('授权Trakt失败' + e))
}
listen<string>('trakt_auth', (event) => {
    console.log(`trakt_auth: code: ${event.payload}`);
    traktAuthStatus.value = '授权成功，正在获取授权信息'
    const redirect_uri = `http://127.0.0.1:${useRuntimeConfig().runtimeConfig?.axum_port}/trakt_auth`
    traktApi.token({code: event.payload, redirect_uri}).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error(response.status_code + ' ' + response.status_text)
            return
        }
        let json: {access_token: string, refresh_token: string, expires_in: number, created_at: number} = JSON.parse(response.body);
        trakt_info.value = {
            access_token: json.access_token,
            refresh_token: json.refresh_token,
            expires_in: json.expires_in + json.created_at,
            redirect_uri: redirect_uri,
        };
        await saveTraktInfo()
        traktAuthStatus.value = '正在获取用户信息'
        traktApi.getUserInfo().then(response => {
            if (response.status_code != 200) {
                ElMessage.error(response.status_code + ' ' + response.status_text)
                return
            }
            let json: {user: {username: string}} = JSON.parse(response.body);
            trakt_info.value.username = json.user.username;
            saveTraktInfo()
            traktAuthLoading.value = false
            traktAuthStatus.value = '换个账户？授权失效？'
        }).catch(e => {
            ElMessage.error('获取Trakt用户信息失败' + e)
            traktAuthStatus.value = '获取Trakt用户信息失败'
        })
    }).catch(e => ElMessage.error('授权Trakt失败' + e))
});

const trakt_proxy_id = ref<string>('followBrowse');
function getTraktProxy() {
    useGlobalConfig().getGlobalConfigValue("trakt_proxy_id").then(value => {
        trakt_proxy_id.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取Trakt代理失败' + e))
}
function traktProxyChange() {
    useGlobalConfig().getGlobalConfig("trakt_proxy_id").then(config => {
        let savePromise;
        if (config) {
            config.config_value = trakt_proxy_id.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "trakt_proxy_id",
                config_value: trakt_proxy_id.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getTraktProxy()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改全局浏览代理失败' + e))
}
const app_proxy_id = ref<string>('followBrowse');
function getAppProxy() {
    useGlobalConfig().getGlobalConfigValue("app_proxy_id").then(value => {
        app_proxy_id.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取Trakt代理失败' + e))
}
function appProxyChange() {
    useGlobalConfig().getGlobalConfig("app_proxy_id").then(config => {
        let savePromise;
        if (config) {
            config.config_value = app_proxy_id.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "app_proxy_id",
                config_value: app_proxy_id.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getAppProxy()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改全局浏览代理失败' + e))
}
const global_browse_proxy_id = ref<string>('no');
function getGlobalBrowseProxy() {
    useGlobalConfig().getGlobalConfigValue("global_browse_proxy_id").then(value => {
        global_browse_proxy_id.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取全局浏览代理失败' + e))
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
}
const global_play_proxy_id = ref<string>('no');
function getGlobalPlayProxy() {
    useGlobalConfig().getGlobalConfigValue("global_play_proxy_id").then(value => {
        global_play_proxy_id.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取全局播放代理失败' + e))
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
}
function proxyChange(line: EmbyLine) {
    useEmbyLine().updateEmbyLine(line).then(() => {
        useEventBus().emit('EmbyLineChanged', {})
        if (line.in_use) {
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

const mpv_startup_dir = ref<string>('');
function getMpvStartupDir() {
    useGlobalConfig().getGlobalConfigValue("mpv_startup_dir").then(value => {
        mpv_startup_dir.value = value ? value : "";
    }).catch(e => ElMessage.error('获取MPV启动目录失败' + e))
}
function mpvStartupDirChange() {
    useGlobalConfig().getGlobalConfig("mpv_startup_dir").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_startup_dir.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_startup_dir",
                config_value: mpv_startup_dir.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvStartupDir()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV启动目录失败' + e))
}

const mpv_args = ref<string>('');
function getMpvArgs() {
    useGlobalConfig().getGlobalConfigValue("mpv_args").then(value => {
        mpv_args.value = value ? value : "";
    }).catch(e => ElMessage.error('获取MPV启动参数失败' + e))
}
function mpvArgsChange() {
    useGlobalConfig().getGlobalConfig("mpv_args").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_args.value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_args",
                config_value: mpv_args.value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvArgs()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV启动参数失败' + e))
}

const mpv_cache_seconds = ref<number>(0);
function getMpvCacheSeconds() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_seconds").then(value => {
        mpv_cache_seconds.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取MPV路径失败' + e))
}
function mpvCacheSecondsChange() {
    useGlobalConfig().getGlobalConfig("mpv_cache_seconds").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_cache_seconds.value + '';
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_cache_seconds",
                config_value: mpv_cache_seconds.value + ''
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvCacheSeconds()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV路径失败' + e))
}

const mpv_cache_max_bytes = ref<number>(0);
function getMpvCacheMaxBytes() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_max_bytes").then(value => {
        mpv_cache_max_bytes.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取MPV路径失败' + e))
}
function mpvCacheMaxBytesChange() {
    useGlobalConfig().getGlobalConfig("mpv_cache_max_bytes").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_cache_max_bytes.value + '';
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_cache_max_bytes",
                config_value: mpv_cache_max_bytes.value + ''
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvCacheMaxBytes()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV路径失败' + e))
}

const mpv_cache_back_seconds = ref<number>(0);
function getMpvCacheBackSeconds() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_back_seconds").then(value => {
        mpv_cache_back_seconds.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取MPV路径失败' + e))
}
function mpvCacheBackSecondsChange() {
    useGlobalConfig().getGlobalConfig("mpv_cache_back_seconds").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_cache_back_seconds.value + '';
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_cache_back_seconds",
                config_value: mpv_cache_back_seconds.value + ''
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvCacheBackSeconds()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV路径失败' + e))
}

const mpv_cache_back_max_bytes = ref<number>(0);
function getMpvCacheBackMaxBytes() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_back_max_bytes").then(value => {
        mpv_cache_back_max_bytes.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取MPV路径失败' + e))
}
function mpvCacheBackMaxBytesChange() {
    useGlobalConfig().getGlobalConfig("mpv_cache_back_max_bytes").then(config => {
        let savePromise;
        if (config) {
            config.config_value = mpv_cache_back_max_bytes.value + '';
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                id: generateGuid(),
                config_key: "mpv_cache_back_max_bytes",
                config_value: mpv_cache_back_max_bytes.value + ''
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        savePromise.then(() => {
            getMpvCacheBackMaxBytes()
            ElMessage.success('修改成功');
        }).catch(e => {
            ElMessage.error('修改失败' + e);
        })
    }).catch(e => ElMessage.error('修改MPV路径失败' + e))
}

const activePane = ref('Common')
function handlePaneChange() {
    if (activePane.value == 'Common') {
    } else if (activePane.value == 'MPV') {
        getMpvPath()
        getMpvStartupDir()
        getMpvArgs()
        getMpvCacheSeconds()
        getMpvCacheMaxBytes()
        getMpvCacheBackSeconds()
        getMpvCacheBackMaxBytes()
    } else if (activePane.value == 'Trakt') {
        getTraktInfo()
        getTraktSyncSwitch()
        getTraktProxy()
    } else if (activePane.value == 'ProxyServer') {
    } else if (activePane.value == 'EmbyLineProxy') {
        listAllEmbyLine()
        getGlobalBrowseProxy()
        getGlobalPlayProxy()
    } else if (activePane.value == 'EmbyIconLibrary') {
        getAppProxy()
        listAllEmbyIconLibrary()
    }
}
handlePaneChange()
</script>

<style scoped>
</style>