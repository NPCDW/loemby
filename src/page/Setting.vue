<template>
    <el-tabs v-model="activePane" @tab-change="handlePaneChange" class="setting-tabs">
        <el-tab-pane label="常规" name="Common">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header"><span class="setting-card-title">应用信息</span></div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="应用更新">
                            <div class="setting-inline">
                                <span class="setting-plain-text">当前版本: {{ runtimeConfig?.version }}</span>
                                <el-button plain type="primary" size="small" :loading="checkUpdateLoading" @click="checkUpdate()">检查更新</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="日志等级">
                            <div class="setting-inline">
                                <span class="setting-plain-text">{{ runtimeConfig?.app_config.log_level }}</span>
                                <el-button plain type="primary" size="small" @click="invokeApi.open_folder('config')">打开配置目录</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="数据库类型">
                            <span class="setting-plain-text">{{ runtimeConfig?.app_config.database_type }}</span>
                        </el-form-item>
                        <el-form-item label="Web端口">
                            <span class="setting-plain-text">{{ runtimeConfig?.axum_port }}</span>
                        </el-form-item>
                    </el-form>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="播放" name="MPV">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header"><span class="setting-card-title">播放策略</span></div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="播放版本自动选择策略">
                            <el-select
                                v-model="play_version_auto_select_policy"
                                @change="configValueChange('play_version_auto_select_policy', play_version_auto_select_policy + '', getPlayVersionAutoSelectPolicy, '播放版本自动选择策略')"
                                class="setting-select">
                                <el-option key="high-resolution" label="高分辨率优先，后高码率优先" value="high-resolution"/>
                                <el-option key="high-bitrate" label="高码率优先" value="high-bitrate"/>
                            </el-select>
                        </el-form-item>
                        <el-form-item label="预加载下一集（当设定的缓存范围到达本集末尾时，提前获取下一集内容）">
                            <el-switch
                                v-model="prefetch_playlist"
                                @change="configValueChange('prefetch_playlist', prefetch_playlist + '', getPrefetchPlaylist, '预加载下一集')"
                                active-value="yes" inactive-value="no" />
                        </el-form-item>
                        <el-form-item label="播放参数IsPlayback">
                            <el-switch
                                v-model="play_param_IsPlayback"
                                @change="configValueChange('play_param_IsPlayback', play_param_IsPlayback + '', getPlayParamIsPlayback, '播放参数IsPlayback开关')"
                                active-value="true" inactive-value="false" />
                        </el-form-item>
                    </el-form>
                </el-card>

                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header"><span class="setting-card-title">网速显示</span></div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="网速显示">
                            <el-switch
                                v-model="cache_speed_enabled"
                                @change="configValueChange('cache_speed_enabled', cache_speed_enabled + '', getCacheSpeedEnabled, '网速显示启用')"
                                active-value="yes" inactive-value="no" />
                        </el-form-item>
                        <el-form-item label="网速显示ASS样式">
                            <el-input
                                v-model="cache_speed_ass_style"
                                @change="configValueChange('cache_speed_ass_style', cache_speed_ass_style + '', getCacheSpeedAssStyle, '网速显示ASS样式')"
                                class="setting-input-lg" placeholder="ASS样式：示例：{\an9\3c&HA066FD&}" />
                        </el-form-item>
                    </el-form>
                </el-card>

                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">MPV缓存</span>
                            <span class="setting-card-desc">按秒计算缓存大小，平均码率除以8再乘以秒即为实际缓存大小，如果大于最大缓存大小，则按最大缓存大小</span>
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="前向缓存">
                            <div class="setting-inline">
                                <el-input-number
                                    v-model="mpv_cache_seconds"
                                    @change="configValueChange('mpv_cache_seconds', mpv_cache_seconds + '', getMpvCacheSeconds, '前向缓存')"
                                    :min="0" :precision="0" :controls="false" class="setting-number-200">
                                    <template #suffix>
                                        <span>秒</span>
                                    </template>
                                </el-input-number>
                                <el-input-number
                                    v-model="mpv_cache_min_bytes"
                                    @change="configValueChange('mpv_cache_min_bytes', mpv_cache_min_bytes + '', getMpvCacheMinBytes, '前向最小缓存')"
                                    :min="0" :precision="0" :controls="false" class="setting-number-200">
                                    <template #prefix>
                                        <span>最小</span>
                                    </template>
                                    <template #suffix>
                                        <span>MiB</span>
                                    </template>
                                </el-input-number>
                                <el-input-number
                                    v-model="mpv_cache_max_bytes"
                                    @change="configValueChange('mpv_cache_max_bytes', mpv_cache_max_bytes + '', getMpvCacheMaxBytes, '前向最大缓存')"
                                    :min="0" :precision="0" :controls="false" class="setting-number-200">
                                    <template #prefix>
                                        <span>最大</span>
                                    </template>
                                    <template #suffix>
                                        <span>MiB</span>
                                    </template>
                                </el-input-number>
                            </div>
                        </el-form-item>
                        <el-form-item label="后向缓存">
                            <div class="setting-inline">
                                <el-input-number
                                    v-model="mpv_cache_back_seconds"
                                    @change="configValueChange('mpv_cache_back_seconds', mpv_cache_back_seconds + '', getMpvCacheBackSeconds, '后向缓存')"
                                    :min="0" :precision="0" :controls="false" class="setting-number-200">
                                    <template #suffix>
                                        <span>秒</span>
                                    </template>
                                </el-input-number>
                                <el-input-number
                                    v-model="mpv_cache_back_min_bytes"
                                    @change="configValueChange('mpv_cache_back_min_bytes', mpv_cache_back_min_bytes + '', getMpvCacheBackMinBytes, '后向最小缓存')"
                                    :min="0" :precision="0" :controls="false" class="setting-number-200">
                                    <template #prefix>
                                        <span>最小</span>
                                    </template>
                                    <template #suffix>
                                        <span>MiB</span>
                                    </template>
                                </el-input-number>
                                <el-input-number
                                    v-model="mpv_cache_back_max_bytes"
                                    @change="configValueChange('mpv_cache_back_max_bytes', mpv_cache_back_max_bytes + '', getMpvCacheBackMaxBytes, '后向最大缓存')"
                                    :min="0" :precision="0" :controls="false" class="setting-number-200">
                                    <template #prefix>
                                        <span>最大</span>
                                    </template>
                                    <template #suffix>
                                        <span>MiB</span>
                                    </template>
                                </el-input-number>
                            </div>
                        </el-form-item>
                    </el-form>
                </el-card>

                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">MPV参数</span>
                            <div class="setting-inline">
                                <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_url('https://mpv.io/manual/stable/')">官方文档</el-button>
                                <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_url('https://hooke007.github.io/official_man/mpv.html')">中文文档</el-button>
                                <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_folder('inner_mpv')">打开内置MPV目录</el-button>
                                <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_file('keymap')">快捷键示意图</el-button>
                            </div>
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="MPV参数">
                            <el-input
                                v-model="mpv_args"
                                @change="configValueChange('mpv_args', mpv_args, getMpvArgs, 'MPV参数')"
                                :rows="4" type="textarea" placeholder="每行一个，示例: 
ontop=no
volume=130
demuxer-max-bytes=512MiB
demuxer-max-back-bytes=512MiB" />
                        </el-form-item>
                    </el-form>
                </el-card>

                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">外部MPV播放器</span>
                            <el-switch
                                v-model="external_mpv_switch"
                                @change="configValueChange('external_mpv_switch', external_mpv_switch + '', getExternalMpvSwitch, '使用外部MPV播放器开关')"
                                active-value="on" inactive-value="off" />
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="MPV文件路径和配置目录">
                            <el-input
                                v-model="mpv_path"
                                @change="configValueChange('mpv_path', mpv_path, getMpvPath, 'MPV文件路径和启动目录')"
                                :disabled="external_mpv_switch != 'on'"
                                :rows="4" type="textarea" placeholder="每行一个mpv路径和配置目录，以英文分号;隔开，不写配置目录默认为mpv同级的portable_config目录或~/.config/mpv目录，示例: 
C:\App\mpv_config-2024.12.04\mpv.exe
/usr/bin/mpv;/usr/local/mpv/portable_config" />
                        </el-form-item>
                    </el-form>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="追踪" name="Track">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">Trakt</span>
                            <el-switch
                                v-model="trakt_sync_switch"
                                @change="configValueChange('trakt_sync_switch', trakt_sync_switch + '', getTraktSyncSwitch, 'Trakt同步开关')"
                                active-value="on" inactive-value="off" inline-prompt active-text="同步已开启" inactive-text="同步已关闭" />
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="Trakt 授权（播放进度 >80% 才能在网页端看到记录）">
                            <div class="setting-inline">
                                <span v-if="trakt_username" class="setting-plain-text">{{ trakt_username }}</span>
                                <el-button v-if="trakt_username" plain type="danger" @click="delAuthTrakt()" size="small">删除授权</el-button>
                                <el-button plain type="primary" :loading="traktAuthLoading" @click="goAuthTrakt()" size="small">{{ traktAuthStatus }}</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="Trakt代理">
                            <el-select
                                v-model="trakt_proxy_id"
                                @change="configValueChange('trakt_proxy_id', trakt_proxy_id + '', getTraktProxy, 'Trakt代理')"
                                class="setting-select-xl">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </el-form-item>
                    </el-form>
                </el-card>
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">Simkl</span>
                            <el-switch
                                v-model="simkl_sync_switch"
                                @change="configValueChange('simkl_sync_switch', simkl_sync_switch + '', getSimklSyncSwitch, 'Simkl同步开关')"
                                active-value="on" inactive-value="off" inline-prompt active-text="同步已开启" inactive-text="同步已关闭" />
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="Simkl 授权">
                            <div class="setting-inline">
                                <span v-if="simkl_username" class="setting-plain-text">{{ simkl_username }}</span>
                                <el-button v-if="simkl_username" plain type="danger" @click="delAuthSimkl()" size="small">删除授权</el-button>
                                <el-button plain type="primary" :loading="simklAuthLoading" @click="goAuthSimkl()" size="small">{{ simklAuthStatus }}</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="Simkl代理">
                            <el-select
                                v-model="simkl_proxy_id"
                                @change="configValueChange('simkl_proxy_id', simkl_proxy_id + '', getSimklProxy, 'Simkl代理')"
                                class="setting-select-xl">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </el-form-item>
                    </el-form>
                </el-card>
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">YamTrack</span>
                            <el-switch
                                v-model="yamtrack_sync_switch"
                                @change="configValueChange('yamtrack_sync_switch', yamtrack_sync_switch + '', getYamTrackSyncSwitch, 'YamTrack同步开关')"
                                active-value="on" inactive-value="off" inline-prompt active-text="同步已开启" inactive-text="同步已关闭" />
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="YamTrack 同步地址 (Emby Integrations)">
                            <el-input
                                v-model="yamtrack_sync_url"
                                @change="configValueChange('yamtrack_sync_url', yamtrack_sync_url, getYamTrackSyncUrl, 'YamTrack同步地址')"
                                placeholder="请输入YamTrack同步地址，示例： https://yamtrack.example.com/webhook/emby/NzDIG-XSNeLD7rYdLRz24dRyJK2v70jd" />
                        </el-form-item>
                        <el-form-item label="YamTrack代理">
                            <el-select
                                v-model="yamtrack_proxy_id"
                                @change="configValueChange('yamtrack_proxy_id', yamtrack_proxy_id + '', getYamTrackProxy, 'YamTrack代理')"
                                class="setting-select-xl">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </el-form-item>
                    </el-form>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="代理服务器" name="ProxyServer">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">代理服务器</span>
                            <el-button plain type="primary" size="small" @click.prevent="addProxy()">添加代理服务器</el-button>
                        </div>
                    </template>
                    <p class="setting-tip">推荐使用 http 代理，reqwest 库的 socks5 代理在某些服可能有问题</p>
                    <el-table :data="proxyServers" class="setting-table">
                        <el-table-column prop="name" label="Name" width="140" show-overflow-tooltip />
                        <el-table-column prop="proxy_type" label="Type" width="80" />
                        <el-table-column prop="addr" label="Address" width="160" show-overflow-tooltip />
                        <el-table-column prop="username" label="Username" width="140" />
                        <el-table-column prop="location" label="Location" show-overflow-tooltip />
                        <el-table-column fixed="right" label="Operations" width="210" align="center">
                            <template #default="scope">
                                <el-button plain :loading="checkProxyLoading[scope.row.id]" type="success" size="small" @click.prevent="checkProxy(scope.row.id)">检测</el-button>
                                <el-button plain type="primary" size="small" @click.prevent="editProxy(scope.$index)">编辑</el-button>
                                <el-button plain type="danger" size="small" @click.prevent="delProxy(scope.$index)">删除</el-button>
                            </template>
                        </el-table-column>
                    </el-table>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="反代服务器" name="ReverseProxyServer">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">反代服务器</span>
                            <el-button plain type="primary" size="small" @click.prevent="addReverseProxy()">添加反代服务器</el-button>
                        </div>
                    </template>
                    <p class="setting-tip">反代服务器地址末尾会自动补充 /，请求地址将拼接为：反代服务器地址 + 原始地址</p>
                    <el-table :data="reverseProxyServers" class="setting-table">
                        <el-table-column prop="name" label="Name" width="140" show-overflow-tooltip />
                        <el-table-column prop="url" label="Url" show-overflow-tooltip />
                        <el-table-column prop="location" label="Location" show-overflow-tooltip />
                        <el-table-column fixed="right" label="Operations" width="210" align="center">
                            <template #default="scope">
                                <el-button plain :loading="checkReverseProxyLoading[scope.row.id]" type="success" size="small" @click.prevent="checkReverseProxy(scope.row.id)">检测</el-button>
                                <el-button plain type="primary" size="small" @click.prevent="editReverseProxy(scope.$index)">编辑</el-button>
                                <el-button plain type="danger" size="small" @click.prevent="delReverseProxy(scope.$index)">删除</el-button>
                            </template>
                        </el-table-column>
                    </el-table>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="Emby线路代理" name="EmbyLineProxy">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">全局代理配置</span>
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="全局媒体库浏览">
                            <el-select
                                v-model="global_browse_proxy_id"
                                @change="configValueChange('global_browse_proxy_id', global_browse_proxy_id + '', () => {getGlobalBrowseProxy(); useEventBus().emit('GlobalProxyChanged', {})}, '全局媒体库浏览代理')"
                                class="setting-select-xl">
                                <template #label="{ label }">
                                    <span style="font-weight: bold">全局配置: </span>
                                    <span>{{ label }}</span>
                                </template>
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </el-form-item>
                        <el-form-item label="全局媒体流播放">
                            <el-select
                                v-model="global_play_proxy_id"
                                @change="configValueChange('global_play_proxy_id', global_play_proxy_id + '', () => {getGlobalPlayProxy(); useEventBus().emit('GlobalProxyChanged', {})}, '全局媒体流播放代理')"
                                class="setting-select-xl">
                                <template #label="{ label }">
                                    <span style="font-weight: bold">全局配置: </span>
                                    <span>{{ label }}</span>
                                </template>
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </el-form-item>
                    </el-form>
                </el-card>
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">线路代理</span>
                            <span class="setting-card-desc">每一行代表一台 Emby 服务器的一条线路，可分别指定反代与代理</span>
                        </div>
                    </template>
                    <el-table :data="embyLines" class="setting-table" :span-method="lineSpanMethod">
                        <el-table-column prop="emby_server_name" label="Emby" show-overflow-tooltip />
                        <el-table-column prop="name" label="线路" show-overflow-tooltip />
                        <el-table-column label="反代服务器">
                            <template #default="scope">
                                <el-select v-model="scope.row.reverse_proxy_id" @change="proxyChange(scope.row)">
                                    <el-option key="no" label="不使用反代" value="no"/>
                                    <el-option v-for="reverseProxyServer in reverseProxyServers" :key="reverseProxyServer.id" :label="reverseProxyServer.name" :value="reverseProxyServer.id"/>
                                </el-select>
                            </template>
                        </el-table-column>
                        <el-table-column label="媒体库浏览">
                            <template #default="scope">
                                <el-select v-model="scope.row.browse_proxy_id" @change="proxyChange(scope.row)">
                                    <el-option key="no" label="不使用代理" value="no"/>
                                    <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                                    <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                </el-select>
                            </template>
                        </el-table-column>
                        <el-table-column label="媒体流播放">
                            <template #default="scope">
                                <el-select v-model="scope.row.play_proxy_id" @change="proxyChange(scope.row)">
                                    <el-option key="no" label="不使用代理" value="no"/>
                                    <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                                    <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                </el-select>
                            </template>
                        </el-table-column>
                    </el-table>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="Emby图标库" name="EmbyIconLibrary">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">应用数据代理</span>
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="应用数据代理（图标、自动更新等）">
                            <el-select
                                v-model="app_proxy_id"
                                @change="configValueChange('app_proxy_id', app_proxy_id + '', getAppProxy, '应用数据代理')"
                                class="setting-select-xl">
                                <el-option key="no" label="不使用代理" value="no"/>
                                <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                            </el-select>
                        </el-form-item>
                    </el-form>
                </el-card>
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">Emby图标库</span>
                            <el-button plain type="primary" size="small" @click.prevent="addEmbyIconLibrary()">添加图标库</el-button>
                        </div>
                    </template>
                    <el-table :data="embyIconLibrary" class="setting-table">
                        <el-table-column prop="name" label="Name" width="140" show-overflow-tooltip />
                        <el-table-column prop="url" label="Url" show-overflow-tooltip />
                        <el-table-column fixed="right" label="Operations" width="210" align="center">
                            <template #default="scope">
                                <el-button plain type="primary" size="small" @click.prevent="editEmbyIconLibrary(scope.$index)">编辑</el-button>
                                <el-button plain type="danger" size="small" @click.prevent="delEmbyIconLibrary(scope.$index)">删除</el-button>
                            </template>
                        </el-table-column>
                    </el-table>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="缓存与日志" name="CacheAndLog">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">缓存与日志</span>
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form setting-form-masonry">
                        <el-form-item label="日志保存天数">
                            <div class="setting-inline">
                                <el-input-number
                                    v-model="logStoredDays"
                                    @change="configValueChange('logStoredDays', logStoredDays + '', getLogStoredDays, '日志保存天数')"
                                    :min="1" :precision="0">
                                    <template #suffix>
                                        <span>天</span>
                                    </template>
                                </el-input-number>
                                <el-button plain type="primary" @click="invokeApi.open_folder('log')">打开日志目录</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="封面图保存天数">
                            <div class="setting-inline">
                                <el-input-number v-model="coverImageStoredDays"
                                    @change="configValueChange('coverImageStoredDays', coverImageStoredDays + '', getCoverImageStoredDays, '封面图保存天数')"
                                    :min="1" :precision="0">
                                    <template #suffix>
                                        <span>天</span>
                                    </template>
                                </el-input-number>
                                <el-button plain type="primary" @click="invokeApi.open_folder('cache')">打开缓存目录</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="图标保存天数">
                            <div class="setting-inline">
                                <el-input-number v-model="iconStoredDays"
                                    @change="configValueChange('iconStoredDays', iconStoredDays + '', getIconStoredDays, '图标保存天数')"
                                    :min="1" :precision="0">
                                    <template #suffix>
                                        <span>天</span>
                                    </template>
                                </el-input-number>
                                <el-button plain type="primary" :loading="cleanIconCacheLoading" @click="cleanIconCache()">清除所有图标缓存</el-button>
                            </div>
                        </el-form-item>
                        <el-form-item label="图片缓存">
                            <el-switch 
                                v-model="disabled_image_cache"
                                @change="configValueChange('disabled_image_cache', disabled_image_cache + '', getDisabledImageCache, '禁用图片缓存')"
                                active-value="off" inactive-value="on"
                                active-text="使用图片缓存" inactive-text="禁用图片缓存" />
                        </el-form-item>
                        <el-form-item label="图片加载">
                            <el-switch 
                                v-model="disabledImage"
                                @change="configValueChange('disabledImage', disabledImage + '', getDisabledImage, '禁用图片加载')"
                                active-value="off" inactive-value="on"
                                active-text="正常显示图片" inactive-text="不请求任何图片" />
                        </el-form-item>
                    </el-form>
                </el-card>
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">Emby服务器缓存</span>
                            <el-button plain type="primary" :loading="cleanAllEmbyCacheLoading" size="small" @click.prevent="cleanAllEmbyCache()">清除所有缓存</el-button>
                        </div>
                    </template>
                    <el-table :data="embyServers" class="setting-table">
                        <el-table-column prop="server_name" label="服务名" />
                        <el-table-column prop="username" label="用户名" />
                        <el-table-column fixed="right" label="Operations" width="180" align="center">
                            <template #default="scope">
                                <el-button plain type="primary" :loading="cleanEmbyCacheLoading" size="small" @click.prevent="cleanEmbyCache(scope.row)">清除缓存</el-button>
                            </template>
                        </el-table-column>
                    </el-table>
                </el-card>
            </div>
        </el-tab-pane>

        <el-tab-pane label="其他" name="Other">
            <div class="setting-pane">
                <el-card class="setting-card">
                    <template #header>
                        <div class="setting-card-header">
                            <span class="setting-card-title">其他</span>
                        </div>
                    </template>
                    <el-form label-position="top" class="setting-form">
                        <el-form-item label="忽略SSL证书错误（重启应用生效）">
                            <el-switch
                                v-model="danger_accept_invalid_certs"
                                @change="configValueChange('danger_accept_invalid_certs', danger_accept_invalid_certs + '', getDangerAcceptInvalidCerts, '忽略SSL证书错误')"
                                active-value="true" inactive-value="false"
                                style="--el-switch-on-color: #F56C6C; --el-switch-off-color: #67C23A"
                                active-text="忽略（危险的）" inactive-text="不忽略（安全）" />
                        </el-form-item>
                    </el-form>
                </el-card>
            </div>
        </el-tab-pane>
    </el-tabs>

    <el-dialog
        v-model="dialogProxyServerVisible"
        title="代理服务器"
        width="800"
    >
        <el-scrollbar>
            <el-form label-position="top" class="setting-form">
                <el-form-item label="代理名称">
                    <el-input v-model="dialogProxyServer.name" placeholder="代理名称" />
                </el-form-item>
                <el-form-item label="代理类型">
                    <el-select v-model="dialogProxyServer.proxy_type" class="setting-select">
                        <el-option key="socks5" label="socks5" value="socks5"/>
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
                    <div class="setting-dialog-actions">
                        <el-button @click="dialogProxyServerVisible = false">取消</el-button>
                        <el-button @click="saveProxyServer" type="primary">保存</el-button>
                    </div>
                </el-form-item>
            </el-form>
        </el-scrollbar>
    </el-dialog>
    <el-dialog
        v-model="dialogReverseProxyServerVisible"
        title="反代服务器"
        width="800"
    >
        <el-scrollbar>
            <el-form label-position="top" class="setting-form">
                <el-form-item label="反代名称">
                    <el-input v-model="dialogReverseProxyServer.name" placeholder="反代名称" />
                </el-form-item>
                <el-form-item label="服务器地址">
                    <el-input v-model="dialogReverseProxyServer.url" placeholder="反代服务器地址，例如 https://proxy.example.org/" />
                </el-form-item>
                <el-form-item>
                    <div class="setting-dialog-actions">
                        <el-button @click="dialogReverseProxyServerVisible = false">取消</el-button>
                        <el-button @click="saveReverseProxyServer" type="primary">保存</el-button>
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
            <el-form label-position="top" class="setting-form">
                <el-form-item label="名称">
                    <el-input v-model="dialogEmbyIconLibrary.name" placeholder="图标库名称" />
                </el-form-item>
                <el-form-item label="地址">
                    <el-input v-model="dialogEmbyIconLibrary.url" placeholder="图标库 http 地址" />
                </el-form-item>
                <el-form-item>
                    <div class="setting-dialog-actions">
                        <el-button @click="dialogEmbyIconLibraryVisible = false">取消</el-button>
                        <el-button @click="saveEmbyIconLibrary" type="primary">保存</el-button>
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
import { ReverseProxyServer, useReverseProxyServer } from '../store/db/reverseProxyServer';
import _ from 'lodash';
import { generateGuid } from '../util/uuid_util';
import appApi from '../api/appApi';
import { EmbyServer, useEmbyServer } from '../store/db/embyServer';
import { EmbyLine, useEmbyLine } from '../store/db/embyLine';
import { useGlobalConfig } from '../store/db/globalConfig';
import { useEventBus } from '../store/eventBus';
import invokeApi from '../api/invokeApi';
import { listen } from '@tauri-apps/api/event';
import {useRuntimeConfig} from "../store/runtimeConfig.ts";
import { EmbyIconLibrary, useEmbyIconLibrary } from '../store/db/embyIconLibrary.ts';

const runtimeConfig = useRuntimeConfig().runtimeConfig;

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

const dialogReverseProxyServerVisible = ref(false);
const dialogReverseProxyServer = ref<ReverseProxyServer>({})

function addReverseProxy() {
    dialogReverseProxyServerVisible.value = true;
    dialogReverseProxyServer.value = {};
}
function editReverseProxy(index: number) {
    dialogReverseProxyServerVisible.value = true;
    dialogReverseProxyServer.value = _.clone(reverseProxyServers.value[index]);
}
function saveReverseProxyServer() {
    // 反代地址末尾必须为 /
    if (dialogReverseProxyServer.value.url && !dialogReverseProxyServer.value.url.trim().endsWith('/')) {
        dialogReverseProxyServer.value.url = dialogReverseProxyServer.value.url.trim() + '/';
    }
    let savePromise;
    if (dialogReverseProxyServer.value.id) {
        savePromise = useReverseProxyServer().updateReverseProxyServer(dialogReverseProxyServer.value)
    } else {
        dialogReverseProxyServer.value.id = generateGuid();
        savePromise = useReverseProxyServer().addReverseProxyServer(dialogReverseProxyServer.value)
    }
    savePromise.then(() => {
        useEventBus().emit('ReverseProxyServerChanged', {})
        ElMessage.success('保存成功');
    }).catch(e => {
        ElMessage.error('保存失败' + e);
    }).finally(() => dialogReverseProxyServerVisible.value = false)
}
function delReverseProxy(index: number) {
    ElMessageBox.confirm(
    `确认删除反代服务器「${reverseProxyServers.value[index].name}」吗`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useReverseProxyServer().delReverseProxyServer(reverseProxyServers.value[index].id!).then(() => {
            useEventBus().emit('ReverseProxyServerChanged', {})
            ElMessage.success('删除成功');
        }).catch(e => ElMessage.error('删除失败' + e))
  })
}

const dialogProxyServerVisible = ref(false);
const dialogProxyServer = ref<ProxyServer>({})

function addProxy() {
    dialogProxyServerVisible.value = true;
    dialogProxyServer.value = {};
}
function editProxy(index: number) {
    dialogProxyServerVisible.value = true;
    dialogProxyServer.value = _.clone(proxyServers.value[index]);
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
    `确认删除代理服务器「${proxyServers.value[index].name}」吗`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useProxyServer().delProxyServer(proxyServers.value[index].id!).then(() => {
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
        let json = JSON.parse(response);
        for (let index = 0; index < proxyServers.value.length; index++) {
            if (proxyServers.value[index].id === id) {
                proxyServers.value[index].location = json["ip"] + " " + json["country_code"];
            }
        }
    }).catch(e => ElMessage.error('检测代理失败，可能是代理配置错误，请检查代理配置' + e)).finally(() => checkProxyLoading.value[id] = false);
}

const checkReverseProxyLoading = ref<{[key: string]: boolean}>({});
function checkReverseProxy(id: string) {
    checkReverseProxyLoading.value[id] = true;
    appApi.getReverseProxyLocation(id).then(async response => {
        let json = JSON.parse(response);
        for (let index = 0; index < reverseProxyServers.value.length; index++) {
            if (reverseProxyServers.value[index].id === id) {
                reverseProxyServers.value[index].location = json["ip"] + " " + json["country_code"];
            }
        }
    }).catch(e => ElMessage.error('检测反代失败，可能是反代配置错误，请检查反代配置' + e)).finally(() => checkReverseProxyLoading.value[id] = false);
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
const embyServerMap = ref<{[key: string]: EmbyServer}>({})
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
        list.forEach(item => {
            embyServerMap.value[item.id!] = item
        })
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

const traktAuthLoading = ref(false)
const trakt_username = ref<string>('');
const traktAuthStatus = ref('去授权')
function getTraktInfo() {
    return useGlobalConfig().getGlobalConfigValue("trakt_username").then(value => {
        trakt_username.value = value;
        if (!traktAuthLoading.value) {
            traktAuthStatus.value = trakt_username.value ? '换个账户？授权失效？' : '去授权'
        }
    }).catch(e => ElMessage.error('获取Trakt信息失败' + e))
}
function delAuthTrakt() {
  ElMessageBox.confirm(
    `确认删除 Trakt 授权吗？`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useGlobalConfig().delGlobalConfig("trakt_username").then(() => {
            useGlobalConfig().delGlobalConfig("trakt_refresh_token")
            useGlobalConfig().delGlobalConfig("trakt_expires_in")
            useGlobalConfig().delGlobalConfig("trakt_access_token")
            useGlobalConfig().delGlobalConfig("trakt_redirect_uri")
            getTraktInfo()
            ElMessageBox.alert('删除成功，同时建议前往 Trakt 官网吊销应用授权，这将删除该应用获取的所有授权，官网地址: https://trakt.tv/oauth/authorized_applications')
        })
    })
}
function goAuthTrakt() {
    traktAuthLoading.value = true
    traktAuthStatus.value = '等待授权回调'
    listenTraktAuth()
    invokeApi.go_trakt_auth().then(() => {
        ElMessage.success('打开浏览器成功，您也可以手动复制地址，去其他浏览器授权');
    }).catch(e => ElMessage.error('授权Trakt失败' + e))
}
const unlistenTraktAuth = ref<() => void>()
async function listenTraktAuth() {
    unlistenTraktAuth.value = await listen<string>('trakt_auth', () => {
        console.log(`trakt_auth event`);
        traktAuthStatus.value = '授权成功'
        useGlobalConfig().refreshCache("trakt_username").then(() => {
            getTraktInfo().then(() => traktAuthLoading.value = false)
        })
    });
}
onUnmounted(() => unlistenTraktAuth.value?.())


const simkl_sync_switch = ref("on")
function getSimklSyncSwitch() {
    return useGlobalConfig().getGlobalConfigValue("simkl_sync_switch").then(value => {
        simkl_sync_switch.value = value ? value : "on";
    }).catch(e => ElMessage.error('获取Simkl同步开关失败' + e))
}

const simklAuthLoading = ref(false)
const simkl_username = ref<string>('');
const simklAuthStatus = ref('去授权')
function getSimklInfo() {
    return useGlobalConfig().getGlobalConfigValue("simkl_username").then(value => {
        simkl_username.value = value;
        if (!simklAuthLoading.value) {
            simklAuthStatus.value = simkl_username.value ? '换个账户？授权失效？' : '去授权'
        }
    }).catch(e => ElMessage.error('获取Simkl信息失败' + e))
}
function delAuthSimkl() {
  ElMessageBox.confirm(
    `确认删除 Simkl 授权吗？`,
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  ).then(async () => {
        useGlobalConfig().delGlobalConfig("simkl_username").then(() => {
            useGlobalConfig().delGlobalConfig("simkl_access_token")
            useGlobalConfig().delGlobalConfig("simkl_redirect_uri")
            getSimklInfo()
            ElMessageBox.alert('删除成功，同时建议前往 Simkl 官网吊销应用授权，这将删除该应用获取的所有授权，官网地址: https://simkl.com/settings/connected-apps/')
        })
    })
}
function goAuthSimkl() {
    simklAuthLoading.value = true
    simklAuthStatus.value = '等待授权回调'
    listenSimklAuth()
    invokeApi.go_simkl_auth().then(() => {
        ElMessage.success('打开浏览器成功，您也可以手动复制地址，去其他浏览器授权');
    }).catch(e => ElMessage.error('授权Simkl失败' + e))
}
const unlistenSimklAuth = ref<() => void>()
async function listenSimklAuth() {
    unlistenSimklAuth.value = await listen<string>('simkl_auth', () => {
        console.log(`simkl_auth event`);
        simklAuthStatus.value = '授权成功'
        useGlobalConfig().refreshCache("simkl_username").then(() => {
            getSimklInfo().then(() => simklAuthLoading.value = false)
        })
    });
}
onUnmounted(() => unlistenSimklAuth.value?.())

const yamtrack_sync_switch = ref("on")
function getYamTrackSyncSwitch() {
    return useGlobalConfig().getGlobalConfigValue("yamtrack_sync_switch").then(value => {
        yamtrack_sync_switch.value = value ? value : "on";
    }).catch(e => ElMessage.error('获取YamTrack同步开关失败' + e))
}
const yamtrack_sync_url = ref("")
function getYamTrackSyncUrl() {
    return useGlobalConfig().getGlobalConfigValue("yamtrack_sync_url").then(value => {
        yamtrack_sync_url.value = value ? value : "";
    }).catch(e => ElMessage.error('获取YamTrack同步地址失败' + e))
}

const yamtrack_proxy_id = ref<string>('followBrowse');
function getYamTrackProxy() {
    useGlobalConfig().getGlobalConfigValue("yamtrack_proxy_id").then(value => {
        yamtrack_proxy_id.value = value ? value : "followBrowse";
    }).catch(e => ElMessage.error('获取YamTrack代理失败' + e))
}
const trakt_proxy_id = ref<string>('followBrowse');
function getTraktProxy() {
    useGlobalConfig().getGlobalConfigValue("trakt_proxy_id").then(value => {
        trakt_proxy_id.value = value ? value : "followBrowse";
    }).catch(e => ElMessage.error('获取Trakt代理失败' + e))
}
const simkl_proxy_id = ref<string>('followBrowse');
function getSimklProxy() {
    useGlobalConfig().getGlobalConfigValue("simkl_proxy_id").then(value => {
        simkl_proxy_id.value = value ? value : "followBrowse";
    }).catch(e => ElMessage.error('获取Simkl代理失败' + e))
}
const app_proxy_id = ref<string>('followBrowse');
function getAppProxy() {
    useGlobalConfig().getGlobalConfigValue("app_proxy_id").then(value => {
        app_proxy_id.value = value ? value : "followBrowse";
    }).catch(e => ElMessage.error('获取App代理失败' + e))
}
const global_browse_proxy_id = ref<string>('no');
const global_browse_proxy_name = ref<string>('不使用代理');
function getGlobalBrowseProxy() {
    useGlobalConfig().getGlobalConfigValue("global_browse_proxy_id").then(async value => {
        global_browse_proxy_id.value = value ? value : "no";
        global_browse_proxy_name.value = await useProxyServer().getProxyServerName(value);
    }).catch(e => ElMessage.error('获取全局浏览代理失败' + e))
}
getGlobalBrowseProxy()
const global_play_proxy_id = ref<string>('no');
const global_play_proxy_name = ref<string>('不使用代理');
function getGlobalPlayProxy() {
    useGlobalConfig().getGlobalConfigValue("global_play_proxy_id").then(async value => {
        global_play_proxy_id.value = value ? value : "no";
        global_play_proxy_name.value = await useProxyServer().getProxyServerName(value);
    }).catch(e => ElMessage.error('获取全局播放代理失败' + e))
}
getGlobalPlayProxy()
function proxyChange(line: EmbyLine) {
    // 选择反代服务器后，两个代理服务器默认切换为不使用代理
    if (line.reverse_proxy_id && line.reverse_proxy_id !== 'no') {
        line.browse_proxy_id = 'no'
        line.play_proxy_id = 'no'
    }
    useEmbyLine().updateEmbyLine(line).then(() => {
        useEventBus().emit('EmbyLineChanged', {})
        if (line.id === embyServerMap.value[line.emby_server_id!].line_id) {
            useEmbyServer().updateEmbyServer({
                id: line.emby_server_id,
                browse_proxy_id: line.browse_proxy_id,
                play_proxy_id: line.play_proxy_id,
                reverse_proxy_id: line.reverse_proxy_id
            }).then(() => {
                useEventBus().emit('EmbyServerChanged', {event: 'update', id: line.emby_server_id})
            }).catch(e => ElMessage.error('修改失败' + e));
        }
        ElMessage.success('修改成功');
    }).catch(e => ElMessage.error('修改失败' + e));
}

const play_version_auto_select_policy = ref<string>('');
function getPlayVersionAutoSelectPolicy() {
    useGlobalConfig().getGlobalConfigValue("play_version_auto_select_policy").then(value => {
        play_version_auto_select_policy.value = value ? value : "high-resolution";
    }).catch(e => ElMessage.error('获取播放版本自动选择策略失败' + e))
}

const prefetch_playlist = ref<string>('no');
function getPrefetchPlaylist() {
    useGlobalConfig().getGlobalConfigValue("prefetch_playlist").then(value => {
        prefetch_playlist.value = value ? value : "no";
    }).catch(e => ElMessage.error('获取播放参数IsPlayback开关失败' + e))
}

const play_param_IsPlayback = ref<string>('true');
function getPlayParamIsPlayback() {
    useGlobalConfig().getGlobalConfigValue("play_param_IsPlayback").then(value => {
        play_param_IsPlayback.value = value ? value : "true";
    }).catch(e => ElMessage.error('获取播放参数IsPlayback开关失败' + e))
}

const cache_speed_enabled = ref<string>('yes');
function getCacheSpeedEnabled() {
    useGlobalConfig().getGlobalConfigValue("cache_speed_enabled").then(value => {
        cache_speed_enabled.value = value ? value : "yes";
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const cache_speed_ass_style = ref<string>('');
function getCacheSpeedAssStyle() {
    useGlobalConfig().getGlobalConfigValue("cache_speed_ass_style").then(value => {
        cache_speed_ass_style.value = value ? value : "";
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const external_mpv_switch = ref<string>('off');
function getExternalMpvSwitch() {
    useGlobalConfig().getGlobalConfigValue("external_mpv_switch").then(value => {
        external_mpv_switch.value = value ? value : "off";
    }).catch(e => ElMessage.error('获取是否使用外部播放器失败' + e))
}

const mpv_path = ref<string>('');
function getMpvPath() {
    useGlobalConfig().getGlobalConfigValue("mpv_path").then(value => {
        mpv_path.value = value ? value : "";
    }).catch(e => ElMessage.error('获取MPV路径失败' + e))
}

const mpv_args = ref<string>('');
function getMpvArgs() {
    useGlobalConfig().getGlobalConfigValue("mpv_args").then(value => {
        mpv_args.value = value ? value : "";
    }).catch(e => ElMessage.error('获取MPV启动参数失败' + e))
}

const mpv_cache_seconds = ref<number>(0);
function getMpvCacheSeconds() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_seconds").then(value => {
        mpv_cache_seconds.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const mpv_cache_min_bytes = ref<number>(0);
function getMpvCacheMinBytes() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_min_bytes").then(value => {
        mpv_cache_min_bytes.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const mpv_cache_max_bytes = ref<number>(0);
function getMpvCacheMaxBytes() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_max_bytes").then(value => {
        mpv_cache_max_bytes.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const mpv_cache_back_seconds = ref<number>(0);
function getMpvCacheBackSeconds() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_back_seconds").then(value => {
        mpv_cache_back_seconds.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const mpv_cache_back_min_bytes = ref<number>(0);
function getMpvCacheBackMinBytes() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_back_min_bytes").then(value => {
        mpv_cache_back_min_bytes.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const mpv_cache_back_max_bytes = ref<number>(0);
function getMpvCacheBackMaxBytes() {
    useGlobalConfig().getGlobalConfigValue("mpv_cache_back_max_bytes").then(value => {
        mpv_cache_back_max_bytes.value = value ? Number(value) : 0;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const logStoredDays = ref<number>(30);
function getLogStoredDays() {
    useGlobalConfig().getGlobalConfigValue("logStoredDays").then(value => {
        logStoredDays.value = value ? Number(value) : 30;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const disabled_image_cache = ref<string>('off');
function getDisabledImageCache() {
    useGlobalConfig().getGlobalConfigValue("disabled_image_cache").then(value => {
        disabled_image_cache.value = value ? value : 'off';
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const disabledImage = ref<string>('off');
function getDisabledImage() {
    useGlobalConfig().getGlobalConfigValue("disabledImage").then(value => {
        disabledImage.value = value ? value : 'off';
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const coverImageStoredDays = ref<number>(30);
function getCoverImageStoredDays() {
    useGlobalConfig().getGlobalConfigValue("coverImageStoredDays").then(value => {
        coverImageStoredDays.value = value ? Number(value) : 30;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const iconStoredDays = ref<number>(365);
function getIconStoredDays() {
    useGlobalConfig().getGlobalConfigValue("iconStoredDays").then(value => {
        iconStoredDays.value = value ? Number(value) : 365;
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

const danger_accept_invalid_certs = ref<string>('false');
function getDangerAcceptInvalidCerts() {
    useGlobalConfig().getGlobalConfigValue("danger_accept_invalid_certs").then(value => {
        danger_accept_invalid_certs.value = value ? value : "false";
    }).catch(e => ElMessage.error('获取配置失败' + e))
}

function configValueChange(key: string, value: string, callback: () => void, keyName: string = key) {
    return useGlobalConfig().getGlobalConfig(key).then(config => {
        let savePromise;
        if (config) {
            config.config_value = value;
            savePromise = useGlobalConfig().updateGlobalConfig(config);
        } else {
            config = {
                config_key: key,
                config_value: value
            }
            savePromise = useGlobalConfig().addGlobalConfig(config);
        }
        return savePromise.then(() => {
            callback()
        }).catch(e => {
            ElMessage.error('修改' + keyName + '失败' + e);
        })
    }).catch(e => ElMessage.error('修改配置失败' + e))
}

const cleanIconCacheLoading = ref(false)
function cleanIconCache() {
    cleanIconCacheLoading.value = true
    invokeApi.clean_icon_cache().finally(() => {
        cleanIconCacheLoading.value = false
    })
}
const cleanEmbyCacheLoading = ref(false)
function cleanEmbyCache(server: EmbyServer) {
    cleanEmbyCacheLoading.value = true
    invokeApi.clean_emby_image_cache(server.id).finally(() => {
        cleanEmbyCacheLoading.value = false
    })
}
const cleanAllEmbyCacheLoading = ref(false)
function cleanAllEmbyCache() {
    cleanAllEmbyCacheLoading.value = true
    invokeApi.clean_emby_image_cache().finally(() => {
        cleanAllEmbyCacheLoading.value = false
    })
}

const activePane = ref('Common')
function handlePaneChange() {
    if (activePane.value == 'Common') {
    } else if (activePane.value == 'MPV') {
        getPlayVersionAutoSelectPolicy()
        getExternalMpvSwitch()
        getPrefetchPlaylist()
        getPlayParamIsPlayback()
        getCacheSpeedEnabled()
        getCacheSpeedAssStyle()
        getMpvPath()
        getMpvArgs()
        getMpvCacheSeconds()
        getMpvCacheMinBytes()
        getMpvCacheMaxBytes()
        getMpvCacheBackSeconds()
        getMpvCacheBackMinBytes()
        getMpvCacheBackMaxBytes()
    } else if (activePane.value == 'Track') {
        getTraktInfo()
        getTraktSyncSwitch()
        getTraktProxy()
        getSimklInfo()
        getSimklSyncSwitch()
        getSimklProxy()
        getYamTrackSyncUrl()
        getYamTrackSyncSwitch()
        getYamTrackProxy()
    } else if (activePane.value == 'ProxyServer') {
    } else if (activePane.value == 'ReverseProxyServer') {
        listAllReverseProxyServer()
    } else if (activePane.value == 'EmbyLineProxy') {
        listAllEmbyLine()
    } else if (activePane.value == 'EmbyIconLibrary') {
        getAppProxy()
        listAllEmbyIconLibrary()
    } else if (activePane.value == 'CacheAndLog') {
        getLogStoredDays()
        getDisabledImageCache()
        getDisabledImage()
        getCoverImageStoredDays()
        getIconStoredDays()
    } else if (activePane.value == 'Other') {
        getDangerAcceptInvalidCerts()
    }
}
handlePaneChange()
</script>

<style scoped>
/* 设置页统一布局：内容整体居中、卡片等宽、字段左对齐 */
.setting-tabs {
    height: calc(100vh - var(--title-bar-height, 40px));
    padding: 10px 20px 0 20px;
    box-sizing: border-box;
}

.setting-tabs :deep(.el-tabs__header) {
    margin: 0 0 16px 0;
}

.setting-pane {
    height: calc(100vh - 120px);
    overflow-y: auto;
    padding: 0 10px 24px 0;
    box-sizing: border-box;
}

.setting-pane > * + * {
    margin-top: 14px;
}

.setting-card {
    border-radius: 10px;
}

.setting-card :deep(.el-card__header) {
    padding: 14px 18px;
}

.setting-card :deep(.el-card__body) {
    padding: 18px;
}

.setting-card-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    min-height: 24px;
}

.setting-card-title {
    font-size: 15px;
    font-weight: 600;
    line-height: 24px;
}

.setting-card-desc,
.setting-tip {
    color: var(--el-text-color-secondary);
    font-size: 12px;
    line-height: 1.6;
}

.setting-tip {
    margin: 0 0 14px 0;
}

.setting-form :deep(.el-form-item) {
    margin-bottom: 18px;
}

.setting-form :deep(.el-form-item:last-child) {
    margin-bottom: 0;
}

.setting-form :deep(.el-form-item__label) {
    width: 100%;
    padding: 0 0 6px 0;
    line-height: 1.5;
    color: var(--el-text-color-regular);
}

.setting-form :deep(.el-form-item__content) {
    width: 100%;
}

.setting-inline {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
}

.setting-plain-text {
    color: var(--el-text-color-primary);
    line-height: 32px;
}

.setting-select {
    width: 230px;
}

.setting-select-xl {
    width: 320px;
}

.setting-input-lg {
    width: 360px;
}

.setting-number-200 {
    width: 200px;
}

.setting-table {
    width: 100%;
}

.setting-form-masonry {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    column-gap: 24px;
    align-items: start;
}

.setting-form-masonry :deep(.el-form-item) {
    margin-bottom: 18px;
}

.setting-dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    width: 100%;
}
</style>
