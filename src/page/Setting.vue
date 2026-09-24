<template>
    <div class="setting-container">
        <el-tabs v-model="activePane" @tab-change="handlePaneChange" class="setting-tabs">
            <!-- 1. 常规 -->
            <el-tab-pane label="常规" name="Common">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner">
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">系统与运行信息</span>
                                    <span class="card-desc">应用当前环境配置、版本状态与服务端口</span>
                                </div>
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">应用更新</div>
                                        <div class="setting-desc">检查最新客户端版本与更新补丁</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-tag type="info" size="default" effect="plain" class="info-tag">当前版本: {{ runtimeConfig?.version || '未知' }}</el-tag>
                                        <el-button plain type="primary" size="small" :loading="checkUpdateLoading" @click="checkUpdate()">检查更新</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">日志等级</div>
                                        <div class="setting-desc">当前运行时的系统日志记录级别</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-tag type="warning" size="default" effect="plain" class="info-tag">{{ runtimeConfig?.app_config.log_level || 'INFO' }}</el-tag>
                                        <el-button plain type="primary" size="small" @click="invokeApi.open_folder('config')">打开配置目录</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">数据库类型</div>
                                        <div class="setting-desc">应用本地数据存储驱动模式</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-tag type="success" size="default" effect="plain" class="info-tag">{{ runtimeConfig?.app_config.database_type || 'SQLite' }}</el-tag>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">Web 端口</div>
                                        <div class="setting-desc">内置 Axum HTTP Web 服务监听端口</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-tag type="primary" size="default" effect="plain" class="info-tag">{{ runtimeConfig?.axum_port }}</el-tag>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>
            
            <!-- 2. 播放 -->
            <el-tab-pane label="播放" name="MPV">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner">
                        <!-- 播放策略与参数 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">播放策略与控制</span>
                                    <span class="card-desc">自动版本选择规则、流媒体预载与播放行为</span>
                                </div>
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">播放版本自动选择策略</div>
                                        <div class="setting-desc">多码率或多分辨率源同时存在时的自动优选算法</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-select
                                            v-model="play_version_auto_select_policy"
                                            @change="configValueChange('play_version_auto_select_policy', play_version_auto_select_policy + '', getPlayVersionAutoSelectPolicy, '播放版本自动选择策略')"
                                            style="width: 260px;">
                                            <el-option key="high-resolution" label="高分辨率优先，后高码率优先" value="high-resolution"/>
                                            <el-option key="high-bitrate" label="高码率优先" value="high-bitrate"/>
                                        </el-select>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">预加载下一集</div>
                                        <div class="setting-desc">当设定的缓存范围到达本集末尾时，提前获取并缓存下一集内容</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-switch
                                            v-model="prefetch_playlist"
                                            @change="configValueChange('prefetch_playlist', prefetch_playlist + '', getPrefetchPlaylist, '预加载下一集')"
                                            active-value="yes" inactive-value="no" />
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">播放参数 IsPlayback</div>
                                        <div class="setting-desc">向 Emby 服务器上报实时的播放进度与就绪状态标记</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-switch
                                            v-model="play_param_IsPlayback"
                                            @change="configValueChange('play_param_IsPlayback', play_param_IsPlayback + '', getPlayParamIsPlayback, '播放参数IsPlayback开关')"
                                            active-value="true" inactive-value="false" />
                                    </div>
                                </div>
                                <div class="setting-row align-start">
                                    <div class="setting-info">
                                        <div class="setting-label">网速显示</div>
                                        <div class="setting-desc">在播放器画面显示实时网络缓冲速率，支持自定义 ASS 字幕样式</div>
                                    </div>
                                    <div class="setting-controls-column">
                                        <div class="controls-line">
                                            <el-switch
                                                v-model="cache_speed_enabled"
                                                @change="configValueChange('cache_speed_enabled', cache_speed_enabled + '', getCacheSpeedEnabled, '网速显示启用')"
                                                active-value="yes" inactive-value="no" />
                                        </div>
                                        <el-input
                                            v-model="cache_speed_ass_style"
                                            @change="configValueChange('cache_speed_ass_style', cache_speed_ass_style + '', getCacheSpeedAssStyle, '网速显示ASS样式')"
                                            style="width: 320px; margin-top: 10px;"
                                            placeholder="ASS样式：示例：{\an9\3c&HA066FD&}" />
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- MPV 缓存配置 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">MPV 缓存配置</span>
                                    <span class="card-desc">按秒计算缓存大小（平均码率 / 8 * 秒数 = 实际缓存大小），若超出最大限制则按最大缓存大小生效</span>
                                </div>
                            </div>
                            <div class="cache-grid-wrapper">
                                <!-- 前向缓存区块 -->
                                <div class="cache-box forward-box">
                                    <div class="cache-box-header">
                                        <span class="cache-badge forward-badge">前向缓存</span>
                                        <span class="cache-box-desc">控制当前播放点之后的提前缓冲量</span>
                                    </div>
                                    <div class="cache-inputs-list">
                                        <div class="cache-input-row">
                                            <span class="input-title">前向时长</span>
                                            <el-input-number
                                                v-model="mpv_cache_seconds"
                                                @change="configValueChange('mpv_cache_seconds', mpv_cache_seconds + '', getMpvCacheSeconds, '前向缓存')"
                                                :min="0" :precision="0" :controls="true" style="width: 170px;">
                                                <template #suffix><span>秒</span></template>
                                            </el-input-number>
                                        </div>
                                        <div class="cache-input-row">
                                            <span class="input-title">前向最小缓存</span>
                                            <el-input-number
                                                v-model="mpv_cache_min_bytes"
                                                @change="configValueChange('mpv_cache_min_bytes', mpv_cache_min_bytes + '', getMpvCacheMinBytes, '前向最小缓存')"
                                                :min="0" :precision="0" :controls="true" style="width: 170px;">
                                                <template #suffix><span>MiB</span></template>
                                            </el-input-number>
                                        </div>
                                        <div class="cache-input-row">
                                            <span class="input-title">前向最大缓存</span>
                                            <el-input-number
                                                v-model="mpv_cache_max_bytes"
                                                @change="configValueChange('mpv_cache_max_bytes', mpv_cache_max_bytes + '', getMpvCacheMaxBytes, '前向最大缓存')"
                                                :min="0" :precision="0" :controls="true" style="width: 170px;">
                                                <template #suffix><span>MiB</span></template>
                                            </el-input-number>
                                        </div>
                                    </div>
                                </div>

                                <!-- 后向缓存区块 -->
                                <div class="cache-box backward-box">
                                    <div class="cache-box-header">
                                        <span class="cache-badge backward-badge">后向缓存</span>
                                        <span class="cache-box-desc">控制已播放内容保留在内存中的回退大小</span>
                                    </div>
                                    <div class="cache-inputs-list">
                                        <div class="cache-input-row">
                                            <span class="input-title">后向时长</span>
                                            <el-input-number
                                                v-model="mpv_cache_back_seconds"
                                                @change="configValueChange('mpv_cache_back_seconds', mpv_cache_back_seconds + '', getMpvCacheBackSeconds, '后向缓存')"
                                                :min="0" :precision="0" :controls="true" style="width: 170px;">
                                                <template #suffix><span>秒</span></template>
                                            </el-input-number>
                                        </div>
                                        <div class="cache-input-row">
                                            <span class="input-title">后向最小缓存</span>
                                            <el-input-number
                                                v-model="mpv_cache_back_min_bytes"
                                                @change="configValueChange('mpv_cache_back_min_bytes', mpv_cache_back_min_bytes + '', getMpvCacheBackMinBytes, '后向最小缓存')"
                                                :min="0" :precision="0" :controls="true" style="width: 170px;">
                                                <template #suffix><span>MiB</span></template>
                                            </el-input-number>
                                        </div>
                                        <div class="cache-input-row">
                                            <span class="input-title">后向最大缓存</span>
                                            <el-input-number
                                                v-model="mpv_cache_back_max_bytes"
                                                @change="configValueChange('mpv_cache_back_max_bytes', mpv_cache_back_max_bytes + '', getMpvCacheBackMaxBytes, '后向最大缓存')"
                                                :min="0" :precision="0" :controls="true" style="width: 170px;">
                                                <template #suffix><span>MiB</span></template>
                                            </el-input-number>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- MPV 启动参数 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">MPV 启动参数</span>
                                    <span class="card-desc">配置播放器初始启动选项，每行一条参数</span>
                                </div>
                                <div class="card-header-actions">
                                    <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_url('https://mpv.io/manual/stable/')">官方文档</el-button>
                                    <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_url('https://hooke007.github.io/official_man/mpv.html')">中文文档</el-button>
                                    <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_folder('inner_mpv')">打开内置MPV目录</el-button>
                                    <el-button plain type="primary" size="small" @click.stop.prevent="invokeApi.open_file('keymap')">快捷键示意图</el-button>
                                </div>
                            </div>
                            <div class="card-sub-content">
                                <el-input
                                    v-model="mpv_args"
                                    @change="configValueChange('mpv_args', mpv_args, getMpvArgs, 'MPV参数')"
                                    :rows="4" type="textarea"
                                    class="mono-textarea"
                                    placeholder="每行一个，示例: 
ontop=no
volume=130
demuxer-max-bytes=512MiB
demuxer-max-back-bytes=512MiB" />
                            </div>
                        </div>

                        <!-- 外部 MPV 播放器 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">使用外部 MPV 播放器</span>
                                    <span class="card-desc">启用后将调用外部独立安装的 MPV 进程进行音视频播放</span>
                                </div>
                                <el-switch
                                    v-model="external_mpv_switch"
                                    @change="configValueChange('external_mpv_switch', external_mpv_switch + '', getExternalMpvSwitch, '使用外部MPV播放器开关')"
                                    active-value="on" inactive-value="off" />
                            </div>
                            <div class="card-sub-content">
                                <div class="field-caption">MPV 文件路径和配置目录</div>
                                <el-input
                                    v-model="mpv_path"
                                    @change="configValueChange('mpv_path', mpv_path, getMpvPath, 'MPV文件路径和启动目录')"
                                    :disabled="external_mpv_switch != 'on'"
                                    :rows="4" type="textarea"
                                    class="mono-textarea"
                                    placeholder="每行一个mpv路径和配置目录，以英文分号;隔开，不写配置目录默认为mpv同级的portable_config目录或~/.config/mpv目录，示例: 
C:\App\mpv_config-2024.12.04\mpv.exe
/usr/bin/mpv;/usr/local/mpv/portable_config" />
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>
            
            <!-- 3. 追踪 -->
            <el-tab-pane label="追踪" name="Track">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner">
                        <!-- Trakt -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">Trakt 同步</span>
                                    <span class="card-desc">播放进度 >80% 即可在 Trakt 网页端看到记录</span>
                                </div>
                                <el-switch
                                    v-model="trakt_sync_switch"
                                    @change="configValueChange('trakt_sync_switch', trakt_sync_switch + '', getTraktSyncSwitch, 'Trakt同步开关')"
                                    active-value="on" inactive-value="off" inline-prompt active-text="同步已开启" inactive-text="同步已关闭" />
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">Trakt 授权</div>
                                        <div class="setting-desc">绑定您的 Trakt.tv 账号授权信息</div>
                                    </div>
                                    <div class="setting-controls">
                                        <div v-if="trakt_username" class="user-auth-badge">
                                            <el-tag type="success" size="default" effect="plain">{{ trakt_username }}</el-tag>
                                            <el-button plain type="danger" @click="delAuthTrakt()" size="small">删除授权</el-button>
                                        </div>
                                        <el-button plain type="primary" :loading="traktAuthLoading" @click="goAuthTrakt()" size="small">{{ traktAuthStatus }}</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">Trakt 代理</div>
                                        <div class="setting-desc">Trakt API 通讯出站节点配置</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-select
                                            v-model="trakt_proxy_id"
                                            @change="configValueChange('trakt_proxy_id', trakt_proxy_id + '', getTraktProxy, 'Trakt代理')"
                                            style="width: 280px;">
                                            <el-option key="no" label="不使用代理" value="no"/>
                                            <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                            <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                        </el-select>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Simkl -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">Simkl 同步</span>
                                    <span class="card-desc">自动将观影与追剧状态记录同步至 Simkl 平台</span>
                                </div>
                                <el-switch
                                    v-model="simkl_sync_switch"
                                    @change="configValueChange('simkl_sync_switch', simkl_sync_switch + '', getSimklSyncSwitch, 'Simkl同步开关')"
                                    active-value="on" inactive-value="off" inline-prompt active-text="同步已开启" inactive-text="同步已关闭" />
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">Simkl 授权</div>
                                        <div class="setting-desc">绑定您的 Simkl 账号授权信息</div>
                                    </div>
                                    <div class="setting-controls">
                                        <div v-if="simkl_username" class="user-auth-badge">
                                            <el-tag type="success" size="default" effect="plain">{{ simkl_username }}</el-tag>
                                            <el-button plain type="danger" @click="delAuthSimkl()" size="small">删除授权</el-button>
                                        </div>
                                        <el-button plain type="primary" :loading="simklAuthLoading" @click="goAuthSimkl()" size="small">{{ simklAuthStatus }}</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">Simkl 代理</div>
                                        <div class="setting-desc">Simkl API 通讯出站节点配置</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-select
                                            v-model="simkl_proxy_id"
                                            @change="configValueChange('simkl_proxy_id', simkl_proxy_id + '', getSimklProxy, 'Simkl代理')"
                                            style="width: 280px;">
                                            <el-option key="no" label="不使用代理" value="no"/>
                                            <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                            <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                        </el-select>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- YamTrack -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">YamTrack 同步</span>
                                    <span class="card-desc">基于 Emby Integrations Webhook 协议与 YamTrack 同步</span>
                                </div>
                                <el-switch
                                    v-model="yamtrack_sync_switch"
                                    @change="configValueChange('yamtrack_sync_switch', yamtrack_sync_switch + '', getYamTrackSyncSwitch, 'YamTrack同步开关')"
                                    active-value="on" inactive-value="off" inline-prompt active-text="同步已开启" inactive-text="同步已关闭" />
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row align-start">
                                    <div class="setting-info">
                                        <div class="setting-label">YamTrack 同步地址 (Emby Integrations)</div>
                                        <div class="setting-desc">YamTrack 服务端生成的专属 Webhook 接收地址</div>
                                    </div>
                                    <div class="setting-controls flex-grow-control">
                                        <el-input
                                            v-model="yamtrack_sync_url"
                                            @change="configValueChange('yamtrack_sync_url', yamtrack_sync_url, getYamTrackSyncUrl, 'YamTrack同步地址')"
                                            placeholder="请输入YamTrack同步地址，示例： https://yamtrack.example.com/webhook/emby/xxx" />
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">YamTrack 代理</div>
                                        <div class="setting-desc">YamTrack Webhook 请求出站代理节点配置</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-select
                                            v-model="yamtrack_proxy_id"
                                            @change="configValueChange('yamtrack_proxy_id', yamtrack_proxy_id + '', getYamTrackProxy, 'YamTrack代理')"
                                            style="width: 280px;">
                                            <el-option key="no" label="不使用代理" value="no"/>
                                            <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                            <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                        </el-select>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>

            <!-- 4. 代理服务器 -->
            <el-tab-pane label="代理服务器" name="ProxyServer">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner wide">
                        <div class="table-section-card">
                            <div class="table-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">代理服务器</span>
                                    <span class="card-desc">推荐使用 HTTP 代理；reqwest 库的 socks5 代理在某些服务下可能存在兼容性问题</span>
                                </div>
                                <el-button plain type="primary" size="default" @click.prevent="addProxy()">添加代理服务器</el-button>
                            </div>
                            <div class="table-card-body">
                                <el-table :data="proxyServers" style="width: 100%" class="custom-data-table">
                                    <el-table-column prop="name" label="名称" width="160" show-overflow-tooltip />
                                    <el-table-column prop="proxy_type" label="类型" width="100" align="center">
                                        <template #default="scope">
                                            <el-tag size="small" :type="scope.row.proxy_type === 'https' ? 'success' : scope.row.proxy_type === 'http' ? 'primary' : 'warning'">
                                                {{ scope.row.proxy_type }}
                                            </el-tag>
                                        </template>
                                    </el-table-column>
                                    <el-table-column prop="addr" label="服务器地址" min-width="180" show-overflow-tooltip />
                                    <el-table-column prop="username" label="用户名" width="130" show-overflow-tooltip>
                                        <template #default="scope">
                                            <span>{{ scope.row.username || '-' }}</span>
                                        </template>
                                    </el-table-column>
                                    <el-table-column prop="location" label="节点归属地 / IP" min-width="160" show-overflow-tooltip>
                                        <template #default="scope">
                                            <span>{{ scope.row.location || '-' }}</span>
                                        </template>
                                    </el-table-column>
                                    <el-table-column label="操作" width="210" align="center" fixed="right">
                                        <template #default="scope">
                                            <el-button plain :loading="checkProxyLoading[scope.row.id]" type="success" size="small" @click.prevent="checkProxy(scope.row.id)">检测</el-button>
                                            <el-button plain type="primary" size="small" @click.prevent="editProxy(scope.$index)">编辑</el-button>
                                            <el-button plain type="danger" size="small" @click.prevent="delProxy(scope.$index)">删除</el-button>
                                        </template>
                                    </el-table-column>
                                </el-table>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>

            <!-- 5. 反代服务器 -->
            <el-tab-pane label="反代服务器" name="ReverseProxyServer">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner wide">
                        <div class="table-section-card">
                            <div class="table-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">反代服务器</span>
                                    <span class="card-desc">反代服务器地址末尾会自动补充 /，请求地址将拼接为：反代服务器地址 + 原始地址</span>
                                </div>
                                <el-button plain type="primary" size="default" @click.prevent="addReverseProxy()">添加反代服务器</el-button>
                            </div>
                            <div class="table-card-body">
                                <el-table :data="reverseProxyServers" style="width: 100%" class="custom-data-table">
                                    <el-table-column prop="name" label="反代名称" width="180" show-overflow-tooltip />
                                    <el-table-column prop="url" label="服务器地址" min-width="260" show-overflow-tooltip />
                                    <el-table-column prop="location" label="节点归属地 / IP" min-width="180" show-overflow-tooltip>
                                        <template #default="scope">
                                            <span>{{ scope.row.location || '-' }}</span>
                                        </template>
                                    </el-table-column>
                                    <el-table-column label="操作" width="210" align="center" fixed="right">
                                        <template #default="scope">
                                            <el-button plain :loading="checkReverseProxyLoading[scope.row.id]" type="success" size="small" @click.prevent="checkReverseProxy(scope.row.id)">检测</el-button>
                                            <el-button plain type="primary" size="small" @click.prevent="editReverseProxy(scope.$index)">编辑</el-button>
                                            <el-button plain type="danger" size="small" @click.prevent="delReverseProxy(scope.$index)">删除</el-button>
                                        </template>
                                    </el-table-column>
                                </el-table>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>

            <!-- 6. Emby线路代理 -->
            <el-tab-pane label="Emby线路代理" name="EmbyLineProxy">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner wide">
                        <!-- 全局默认配置 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">全局代理默认规则</span>
                                    <span class="card-desc">当各 Emby 线路设置为“跟随全局代理”时所采用的默认出站规则</span>
                                </div>
                            </div>
                            <div class="setting-card-body">
                                <div class="proxy-rule-grid">
                                    <div class="proxy-rule-box">
                                        <span class="proxy-rule-label">全局媒体库浏览代理</span>
                                        <el-select
                                            v-model="global_browse_proxy_id"
                                            @change="configValueChange('global_browse_proxy_id', global_browse_proxy_id + '', () => {getGlobalBrowseProxy(); useEventBus().emit('GlobalProxyChanged', {})}, '全局媒体库浏览代理')"
                                            style="width: 100%;">
                                            <template #label="{ label }">
                                                <span class="label-strong">全局配置: </span>
                                                <span>{{ label }}</span>
                                            </template>
                                            <el-option key="no" label="不使用代理" value="no"/>
                                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                        </el-select>
                                    </div>
                                    <div class="proxy-rule-box">
                                        <span class="proxy-rule-label">全局媒体流播放代理</span>
                                        <el-select
                                            v-model="global_play_proxy_id"
                                            @change="configValueChange('global_play_proxy_id', global_play_proxy_id + '', () => {getGlobalPlayProxy(); useEventBus().emit('GlobalProxyChanged', {})}, '全局媒体流播放代理')"
                                            style="width: 100%;">
                                            <template #label="{ label }">
                                                <span class="label-strong">全局配置: </span>
                                                <span>{{ label }}</span>
                                            </template>
                                            <el-option key="no" label="不使用代理" value="no"/>
                                            <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                        </el-select>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- 线路分流表格 -->
                        <div class="table-section-card">
                            <div class="table-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">线路分流与代理配置</span>
                                    <span class="card-desc">配置各 Emby 服务器及其所属线路独立的反代规则与媒体流代理</span>
                                </div>
                            </div>
                            <div class="table-card-body">
                                <el-table :data="embyLines" style="width: 100%" :span-method="lineSpanMethod" class="custom-data-table">
                                    <el-table-column prop="emby_server_name" label="Emby 服务器" min-width="160" show-overflow-tooltip />
                                    <el-table-column prop="name" label="线路" min-width="140" show-overflow-tooltip />
                                    <el-table-column label="反代服务器" min-width="210">
                                        <template #default="scope">
                                            <el-select v-model="scope.row.reverse_proxy_id" @change="proxyChange(scope.row)" style="width: 100%;">
                                                <el-option key="no" label="不使用反代" value="no"/>
                                                <el-option v-for="reverseProxyServer in reverseProxyServers" :key="reverseProxyServer.id" :label="reverseProxyServer.name" :value="reverseProxyServer.id"/>
                                            </el-select>
                                        </template>
                                    </el-table-column>
                                    <el-table-column label="媒体库浏览" min-width="230">
                                        <template #default="scope">
                                            <el-select v-model="scope.row.browse_proxy_id" @change="proxyChange(scope.row)" style="width: 100%;">
                                                <el-option key="no" label="不使用代理" value="no"/>
                                                <el-option key="follow" :label="'跟随全局代理(' + global_browse_proxy_name + ')'" value="follow"/>
                                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                            </el-select>
                                        </template>
                                    </el-table-column>
                                    <el-table-column label="媒体流播放" min-width="230">
                                        <template #default="scope">
                                            <el-select v-model="scope.row.play_proxy_id" @change="proxyChange(scope.row)" style="width: 100%;">
                                                <el-option key="no" label="不使用代理" value="no"/>
                                                <el-option key="follow" :label="'跟随全局代理(' + global_play_proxy_name + ')'" value="follow"/>
                                                <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                            </el-select>
                                        </template>
                                    </el-table-column>
                                </el-table>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>

            <!-- 7. Emby图标库 -->
            <el-tab-pane label="Emby图标库" name="EmbyIconLibrary">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner wide">
                        <!-- 数据代理配置 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">应用数据代理</span>
                                    <span class="card-desc">拉取远程图标、应用自动更新等后台请求使用的代理网络</span>
                                </div>
                                <div class="setting-controls">
                                    <el-select
                                        v-model="app_proxy_id"
                                        @change="configValueChange('app_proxy_id', app_proxy_id + '', getAppProxy, '应用数据代理')"
                                        style="width: 280px;">
                                        <el-option key="no" label="不使用代理" value="no"/>
                                        <el-option key="followBrowse" :label="'跟随全局媒体库浏览代理(' + global_browse_proxy_name + ')'" value="followBrowse"/>
                                        <el-option key="followPlay" :label="'跟随全局媒体流播放代理(' + global_play_proxy_name + ')'" value="followPlay"/>
                                        <el-option v-for="proxyServer in proxyServers" :key="proxyServer.id" :label="proxyServer.name" :value="proxyServer.id"/>
                                    </el-select>
                                </div>
                            </div>
                        </div>

                        <!-- 图标库列表 -->
                        <div class="table-section-card">
                            <div class="table-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">Emby 图标库源</span>
                                    <span class="card-desc">配置在线图标库源地址，用于获取各服务器分类图标与定制徽标</span>
                                </div>
                                <el-button plain type="primary" size="default" @click.prevent="addEmbyIconLibrary()">添加图标库</el-button>
                            </div>
                            <div class="table-card-body">
                                <el-table :data="embyIconLibrary" style="width: 100%" class="custom-data-table">
                                    <el-table-column prop="name" label="图标库名称" width="200" show-overflow-tooltip />
                                    <el-table-column prop="url" label="地址 (URL)" min-width="320" show-overflow-tooltip />
                                    <el-table-column label="操作" width="180" align="center" fixed="right">
                                        <template #default="scope">
                                            <el-button plain type="primary" size="small" @click.prevent="editEmbyIconLibrary(scope.$index)">编辑</el-button>
                                            <el-button plain type="danger" size="small" @click.prevent="delEmbyIconLibrary(scope.$index)">删除</el-button>
                                        </template>
                                    </el-table-column>
                                </el-table>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>

            <!-- 8. 缓存与日志 -->
            <el-tab-pane label="缓存与日志" name="CacheAndLog">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner wide">
                        <!-- 存储策略与开关 -->
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">缓存与日志清理规则</span>
                                    <span class="card-desc">配置文件与资源保留天数，控制本地磁盘占用及图片加载策略</span>
                                </div>
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">日志保存天数</div>
                                        <div class="setting-desc">应用运行时日志的本地保留天数，超出后自动清理</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-input-number
                                            v-model="logStoredDays"
                                            @change="configValueChange('logStoredDays', logStoredDays + '', getLogStoredDays, '日志保存天数')"
                                            :min="1" :precision="0" style="width: 140px;">
                                            <template #suffix><span>天</span></template>
                                        </el-input-number>
                                        <el-button plain type="primary" size="small" @click="invokeApi.open_folder('log')">打开日志目录</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">封面图保存天数</div>
                                        <div class="setting-desc">已缓存的电影与剧集海报图片在本地磁盘保留的有效天数</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-input-number
                                            v-model="coverImageStoredDays"
                                            @change="configValueChange('coverImageStoredDays', coverImageStoredDays + '', getCoverImageStoredDays, '封面图保存天数')"
                                            :min="1" :precision="0" style="width: 140px;">
                                            <template #suffix><span>天</span></template>
                                        </el-input-number>
                                        <el-button plain type="primary" size="small" @click="invokeApi.open_folder('cache')">打开缓存目录</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">图标保存天数</div>
                                        <div class="setting-desc">自定义服务器图标与分类徽标在本地磁盘保留的有效天数</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-input-number
                                            v-model="iconStoredDays"
                                            @change="configValueChange('iconStoredDays', iconStoredDays + '', getIconStoredDays, '图标保存天数')"
                                            :min="1" :precision="0" style="width: 140px;">
                                            <template #suffix><span>天</span></template>
                                        </el-input-number>
                                        <el-button plain type="primary" size="small" :loading="cleanIconCacheLoading" @click="cleanIconCache()">清除所有图标缓存</el-button>
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">禁用图片缓存</div>
                                        <div class="setting-desc">控制是否将加载的远程图片保存到本地磁盘缓存</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-switch 
                                            v-model="disabled_image_cache"
                                            @change="configValueChange('disabled_image_cache', disabled_image_cache + '', getDisabledImageCache, '禁用图片缓存')"
                                            active-value="off" inactive-value="on"
                                            active-text="使用图片缓存" inactive-text="禁用图片缓存" />
                                    </div>
                                </div>
                                <div class="setting-row">
                                    <div class="setting-info">
                                        <div class="setting-label">禁用图片加载</div>
                                        <div class="setting-desc">完全关闭所有海报与图片网络请求，进入纯文本急速列表模式</div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-switch 
                                            v-model="disabledImage"
                                            @change="configValueChange('disabledImage', disabledImage + '', getDisabledImage, '禁用图片加载')"
                                            active-value="off" inactive-value="on"
                                            active-text="正常显示图片" inactive-text="不请求任何图片" />
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- 各服务器图片缓存管理 -->
                        <div class="table-section-card">
                            <div class="table-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">Emby 服务器图片缓存清理</span>
                                    <span class="card-desc">单独管理或一键重置特定 Emby 实例的所有海报与背景缓存</span>
                                </div>
                                <el-button plain type="danger" :loading="cleanAllEmbyCacheLoading" size="default" @click.prevent="cleanAllEmbyCache()">清除所有缓存</el-button>
                            </div>
                            <div class="table-card-body">
                                <el-table :data="embyServers" style="width: 100%" class="custom-data-table">
                                    <el-table-column prop="server_name" label="服务名" min-width="180" show-overflow-tooltip />
                                    <el-table-column prop="username" label="用户名" min-width="160" show-overflow-tooltip />
                                    <el-table-column label="操作" width="180" align="center" fixed="right">
                                        <template #default="scope">
                                            <el-button plain type="primary" :loading="cleanEmbyCacheLoading" size="small" @click.prevent="cleanEmbyCache(scope.row)">清除缓存</el-button>
                                        </template>
                                    </el-table-column>
                                </el-table>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>

            <!-- 9. 其他 -->
            <el-tab-pane label="其他" name="Other">
                <el-scrollbar class="setting-scrollbar">
                    <div class="setting-pane-inner">
                        <div class="setting-card">
                            <div class="setting-card-header">
                                <div class="card-title-group">
                                    <span class="card-title">网络与安全设置</span>
                                    <span class="card-desc">配置底层网络请求安全校验规则及证书忽略选项</span>
                                </div>
                            </div>
                            <div class="setting-card-body">
                                <div class="setting-row align-start">
                                    <div class="setting-info">
                                        <div class="setting-label">忽略 SSL 证书错误（重启应用生效）</div>
                                        <div class="setting-desc">
                                            忽略 HTTPS 自签名证书或证书链无效错误。适用于私有部署或内网自签名服务，但在公开网络环境下可能存在中间人安全风险。
                                        </div>
                                    </div>
                                    <div class="setting-controls">
                                        <el-switch
                                            v-model="danger_accept_invalid_certs"
                                            @change="configValueChange('danger_accept_invalid_certs', danger_accept_invalid_certs + '', getDangerAcceptInvalidCerts, '忽略SSL证书错误')"
                                            active-value="true" inactive-value="false"
                                            style="--el-switch-on-color: #F56C6C; --el-switch-off-color: #67C23A"
                                            active-text="忽略（危险的）" inactive-text="不忽略（安全）" />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </el-scrollbar>
            </el-tab-pane>
        </el-tabs>

        <!-- 模态框：代理服务器 -->
        <el-dialog
            v-model="dialogProxyServerVisible"
            title="代理服务器"
            width="560px"
            destroy-on-close
            class="setting-dialog"
        >
            <el-scrollbar max-height="450px" style="padding: 0 10px;">
                <el-form label-position="top" class="dialog-form">
                    <el-form-item label="代理名称">
                        <el-input v-model="dialogProxyServer.name" placeholder="请输入代理名称" />
                    </el-form-item>
                    <el-form-item label="代理类型">
                        <el-select v-model="dialogProxyServer.proxy_type" style="width: 100%;">
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
                        <el-input v-model="dialogProxyServer.password" type="password" show-password placeholder="密码（可选）" />
                    </el-form-item>
                </el-form>
            </el-scrollbar>
            <template #footer>
                <div class="dialog-footer">
                    <el-button @click="dialogProxyServerVisible = false">取消</el-button>
                    <el-button @click="saveProxyServer" type="primary">保存</el-button>
                </div>
            </template>
        </el-dialog>

        <!-- 模态框：反代服务器 -->
        <el-dialog
            v-model="dialogReverseProxyServerVisible"
            title="反代服务器"
            width="560px"
            destroy-on-close
            class="setting-dialog"
        >
            <el-scrollbar max-height="450px" style="padding: 0 10px;">
                <el-form label-position="top" class="dialog-form">
                    <el-form-item label="反代名称">
                        <el-input v-model="dialogReverseProxyServer.name" placeholder="请输入反代名称" />
                    </el-form-item>
                    <el-form-item label="服务器地址">
                        <el-input v-model="dialogReverseProxyServer.url" placeholder="反代服务器地址，例如 https://proxy.example.org/" />
                    </el-form-item>
                </el-form>
            </el-scrollbar>
            <template #footer>
                <div class="dialog-footer">
                    <el-button @click="dialogReverseProxyServerVisible = false">取消</el-button>
                    <el-button @click="saveReverseProxyServer" type="primary">保存</el-button>
                </div>
            </template>
        </el-dialog>

        <!-- 模态框：Emby图标库 -->
        <el-dialog
            v-model="dialogEmbyIconLibraryVisible"
            title="Emby 图标库"
            width="560px"
            destroy-on-close
            class="setting-dialog"
        >
            <el-scrollbar max-height="450px" style="padding: 0 10px;">
                <el-form label-position="top" class="dialog-form">
                    <el-form-item label="名称">
                        <el-input v-model="dialogEmbyIconLibrary.name" placeholder="请输入图标库名称" />
                    </el-form-item>
                    <el-form-item label="地址">
                        <el-input v-model="dialogEmbyIconLibrary.url" placeholder="图标库 http 地址" />
                    </el-form-item>
                </el-form>
            </el-scrollbar>
            <template #footer>
                <div class="dialog-footer">
                    <el-button @click="dialogEmbyIconLibraryVisible = false">取消</el-button>
                    <el-button @click="saveEmbyIconLibrary" type="primary">保存</el-button>
                </div>
            </template>
        </el-dialog>
    </div>
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
.setting-container {
    height: calc(100vh - 40px);
    padding: 12px 24px 0 24px;
    box-sizing: border-box;
}

.setting-tabs {
    height: 100%;
}

:deep(.el-tabs__header) {
    margin-bottom: 16px;
    border-bottom: 1px solid var(--el-border-color-lighter, #303236);
}

:deep(.el-tabs__nav-wrap::after) {
    height: 1px;
    background-color: var(--el-border-color-lighter, #303236);
}

:deep(.el-tabs__item) {
    font-size: 14px;
    padding: 0 18px;
    font-weight: 500;
    transition: color 0.2s ease;
}

:deep(.el-tabs__item.is-active) {
    font-weight: 600;
}

.setting-scrollbar {
    height: calc(100vh - 110px);
}

.setting-pane-inner {
    /* 卡片内容宽 920 + 卡片左右内边距 24*2 + 边框 2 = 970，
       与默认状态下 el-tabs 下划线宽度（920px）视觉对齐 */
    max-width: 970px;
    margin: 0 auto;
    padding: 6px 0 40px 0;
}

.setting-pane-inner.wide {
    max-width: 1140px;
}

/* 通用卡片容器 */
.setting-card {
    background-color: var(--el-bg-color-overlay, #1c1d1f);
    border: 1px solid var(--el-border-color-lighter, #2e3034);
    border-radius: 10px;
    padding: 20px 24px;
    margin-bottom: 18px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    transition: border-color 0.2s ease;
}

.setting-card:hover {
    border-color: var(--el-border-color, #414449);
}

.setting-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--el-border-color-extra-light, #282a2e);
    margin-bottom: 12px;
}

.card-title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.card-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--el-text-color-primary, #e5eaf3);
    line-height: 1.4;
}

.card-desc {
    font-size: 12px;
    color: var(--el-text-color-secondary, #909399);
    line-height: 1.4;
}

.card-header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

/* 统一设置行 */
.setting-card-body {
    display: flex;
    flex-direction: column;
}

.setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-height: 48px;
    padding: 12px 0;
    border-bottom: 1px solid var(--el-border-color-extra-light, #262729);
}

.setting-row:last-child {
    border-bottom: none;
    padding-bottom: 4px;
}

.setting-row.align-start {
    align-items: flex-start;
}

.setting-info {
    flex: 1;
    padding-right: 24px;
}

.setting-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--el-text-color-primary, #dcdfe6);
    line-height: 1.4;
}

.setting-desc {
    font-size: 12px;
    color: var(--el-text-color-secondary, #8d9095);
    margin-top: 4px;
    line-height: 1.5;
}

.setting-controls {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    flex-shrink: 0;
}

.setting-controls.flex-grow-control {
    flex: 1;
    max-width: 460px;
}

.setting-controls-column {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    flex-shrink: 0;
}

.controls-line {
    display: flex;
    align-items: center;
    height: 32px;
}

.info-tag {
    font-family: inherit;
    font-size: 13px;
    border-radius: 4px;
    padding: 0 10px;
}

.user-auth-badge {
    display: flex;
    align-items: center;
    gap: 10px;
}

/* MPV 缓存两列网格 */
.cache-grid-wrapper {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-top: 4px;
}

@media (max-width: 820px) {
    .cache-grid-wrapper {
        grid-template-columns: 1fr;
    }
}

.cache-box {
    /* 与卡片同级：仅用极浅填充 + 细边框区分分组，避免灰色块破坏整体布局 */
    background-color: transparent;
    border: 1px solid var(--el-border-color-extra-light, #2b2b2c);
    border-radius: 8px;
    padding: 16px;
}

.cache-box-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--el-border-color-extra-light, #242528);
}

.cache-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
}

.forward-badge {
    background-color: rgba(64, 158, 255, 0.12);
    color: #409eff;
    border: 1px solid rgba(64, 158, 255, 0.25);
}

.backward-badge {
    background-color: rgba(103, 194, 58, 0.12);
    color: #67c23a;
    border: 1px solid rgba(103, 194, 58, 0.25);
}

.cache-box-desc {
    font-size: 12px;
    color: var(--el-text-color-secondary, #909399);
}

.cache-inputs-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.cache-input-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
}

.input-title {
    font-size: 13px;
    color: var(--el-text-color-regular, #cfd3dc);
    white-space: nowrap;
}

/* 代码文本框 */
.card-sub-content {
    margin-top: 12px;
}

.field-caption {
    font-size: 13px;
    font-weight: 500;
    color: var(--el-text-color-regular, #cfd3dc);
    margin-bottom: 8px;
}

.mono-textarea :deep(.el-textarea__inner) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 13px;
    line-height: 1.5;
    background-color: var(--el-fill-color-darker, #151617);
    border-color: var(--el-border-color-lighter, #2e3034);
    border-radius: 6px;
    padding: 10px 12px;
}

.mono-textarea :deep(.el-textarea__inner:focus) {
    border-color: var(--el-color-primary, #409eff);
}

/* 表格卡片容器 */
.table-section-card {
    background-color: var(--el-bg-color-overlay, #1c1d1f);
    border: 1px solid var(--el-border-color-lighter, #2e3034);
    border-radius: 10px;
    padding: 20px 24px;
    margin-bottom: 18px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.table-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.table-card-body {
    width: 100%;
}

.custom-data-table {
    border-radius: 8px;
    overflow: hidden;
}

/* 表格沿用卡片背景，避免出现纯黑/透明区域 */
.custom-data-table {
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: var(--el-fill-color-light, #262727);
    --el-table-row-hover-bg-color: var(--el-fill-color-light, #262727);
    --el-table-border-color: var(--el-border-color-extra-light, #2b2b2c);
}

:deep(.custom-data-table .el-table__header-wrapper th) {
    background-color: var(--el-fill-color-light, #262727);
    font-weight: 600;
    color: var(--el-text-color-primary, #dcdfe6);
    height: 44px;
}

:deep(.custom-data-table .el-table__row) {
    height: 48px;
}

/* 行背景透明后补齐单元格与斑马纹底色 */
:deep(.custom-data-table .el-table__body tr > td.el-table__cell) {
    background-color: transparent;
}

:deep(.custom-data-table.el-table--striped .el-table__body tr.el-table__row--striped > td.el-table__cell) {
    background-color: var(--el-fill-color-lighter, #1d1d1d);
}

/* 全局代理网格 */
.proxy-rule-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    padding: 6px 0;
}

@media (max-width: 768px) {
    .proxy-rule-grid {
        grid-template-columns: 1fr;
    }
}

.proxy-rule-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.proxy-rule-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--el-text-color-regular, #cfd3dc);
}

.label-strong {
    font-weight: 600;
    color: var(--el-text-color-secondary, #909399);
    margin-right: 4px;
}

/* 对话框美化 */
:deep(.setting-dialog) {
    border-radius: 10px;
    overflow: hidden;
}

:deep(.setting-dialog .el-dialog__header) {
    margin-right: 0;
    padding: 16px 20px;
    border-bottom: 1px solid var(--el-border-color-lighter, #2e3034);
}

:deep(.setting-dialog .el-dialog__title) {
    font-size: 16px;
    font-weight: 600;
}

:deep(.setting-dialog .el-dialog__body) {
    padding: 16px 20px;
}

:deep(.setting-dialog .el-dialog__footer) {
    padding: 12px 20px;
    border-top: 1px solid var(--el-border-color-lighter, #2e3034);
}

.dialog-form {
    padding: 4px 0;
}

.dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
}
</style>