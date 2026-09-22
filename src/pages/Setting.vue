<template>
    <div class="page settings">
        <el-tabs v-model="activeTab" class="tabs">
            <!-- ————————— 通用 ————————— -->
            <el-tab-pane label="通用" name="general">
                <SettingBlock title="版本信息">
                    <SettingRow label="应用版本" hint="升级由发布渠道决定，检查到新版本后会提示重启">
                        <span class="num">{{ runtimeConfig?.version }}</span>
                        <el-button type="primary" plain :loading="checkingUpdate" @click="checkUpdate">
                            检查更新
                        </el-button>
                    </SettingRow>
                    <SettingRow label="日志等级" :hint="`配置目录：${configDir}`">
                        <span class="num">{{ runtimeConfig?.app_config.log_level }}</span>
                        <el-button plain @click="invokeApi.open_folder('config')">打开配置目录</el-button>
                    </SettingRow>
                    <SettingRow label="数据库">
                        <span class="num">{{ runtimeConfig?.app_config.database_type }}</span>
                    </SettingRow>
                    <SettingRow label="本地服务端口" hint="图片、字幕与播放中转都走这个端口，只监听本机">
                        <span class="num">{{ runtimeConfig?.axum_port }}</span>
                    </SettingRow>
                </SettingBlock>

                <SettingBlock title="其他">
                    <SettingRow
                        label="忽略 SSL 证书错误"
                        hint="自签名证书的服务器需要打开。重启应用后生效。"
                    >
                        <el-switch
                            v-model="dangerCerts"
                            active-value="true"
                            inactive-value="false"
                            active-text="忽略（有风险）"
                            inactive-text="校验证书"
                            @change="saveSetting('danger_accept_invalid_certs')"
                        />
                    </SettingRow>
                </SettingBlock>
            </el-tab-pane>

            <!-- ————————— 播放 ————————— -->
            <el-tab-pane label="播放" name="playback">
                <SettingBlock title="版本选择" caption="同一部影片有多个媒体源时的默认取舍">
                    <SettingRow label="自动选择策略">
                        <el-select
                            v-model="versionPolicy"
                            style="width: 16rem"
                            @change="saveSetting('play_version_auto_select_policy')"
                        >
                            <el-option label="高分辨率优先，其次高码率" value="high-resolution" />
                            <el-option label="高码率优先" value="high-bitrate" />
                        </el-select>
                    </SettingRow>
                    <SettingRow label="预加载下一集" hint="缓存范围到达本集末尾时提前取下一集，减少等待">
                        <el-switch
                            v-model="prefetch"
                            active-value="yes"
                            inactive-value="no"
                            @change="saveSetting('prefetch_playlist')"
                        />
                    </SettingRow>
                    <SettingRow label="请求携带 IsPlayback" hint="部分服务端依赖这个参数才会返回可播放的媒体源">
                        <el-switch
                            v-model="isPlayback"
                            active-value="true"
                            inactive-value="false"
                            @change="saveSetting('play_param_IsPlayback')"
                        />
                    </SettingRow>
                </SettingBlock>

                <SettingBlock title="缓存" caption="按秒估算，实际大小 = 平均码率 ÷ 8 × 秒数，超过上限时按上限">
                    <SettingRow label="前向缓存" stacked>
                        <div class="cache-grid">
                            <el-input-number v-model="cacheSeconds" :min="0" :controls="false" @change="saveSetting('mpv_cache_seconds')">
                                <template #prefix><span class="unit">前向</span></template>
                                <template #suffix><span class="unit">秒</span></template>
                            </el-input-number>
                            <el-input-number v-model="cacheMinBytes" :min="0" :controls="false" @change="saveSetting('mpv_cache_min_bytes')">
                                <template #prefix><span class="unit">最小</span></template>
                                <template #suffix><span class="unit">MiB</span></template>
                            </el-input-number>
                            <el-input-number v-model="cacheMaxBytes" :min="0" :controls="false" @change="saveSetting('mpv_cache_max_bytes')">
                                <template #prefix><span class="unit">最大</span></template>
                                <template #suffix><span class="unit">MiB</span></template>
                            </el-input-number>
                        </div>
                    </SettingRow>
                    <SettingRow label="后向缓存" stacked>
                        <div class="cache-grid">
                            <el-input-number v-model="cacheBackSeconds" :min="0" :controls="false" @change="saveSetting('mpv_cache_back_seconds')">
                                <template #prefix><span class="unit">后向</span></template>
                                <template #suffix><span class="unit">秒</span></template>
                            </el-input-number>
                            <el-input-number v-model="cacheBackMinBytes" :min="0" :controls="false" @change="saveSetting('mpv_cache_back_min_bytes')">
                                <template #prefix><span class="unit">最小</span></template>
                                <template #suffix><span class="unit">MiB</span></template>
                            </el-input-number>
                            <el-input-number v-model="cacheBackMaxBytes" :min="0" :controls="false" @change="saveSetting('mpv_cache_back_max_bytes')">
                                <template #prefix><span class="unit">最大</span></template>
                                <template #suffix><span class="unit">MiB</span></template>
                            </el-input-number>
                        </div>
                    </SettingRow>
                    <SettingRow label="网速显示" hint="在画面上叠加一行实时网速，位置与颜色由 ASS 样式控制">
                        <el-switch
                            v-model="speedEnabled"
                            active-value="yes"
                            inactive-value="no"
                            @change="saveSetting('cache_speed_enabled')"
                        />
                        <el-input
                            v-model="speedStyle"
                            placeholder="{\an9\3c&HA066FD&}"
                            style="width: 17rem"
                            @change="saveSetting('cache_speed_ass_style')"
                        />
                    </SettingRow>
                </SettingBlock>

                <SettingBlock title="播放器">
                    <SettingRow label="使用外部 MPV" hint="关闭时使用随应用打包的内置 mpv">
                        <el-switch
                            v-model="externalMpv"
                            active-value="on"
                            inactive-value="off"
                            @change="saveSetting('external_mpv_switch')"
                        />
                    </SettingRow>
                    <SettingRow label="MPV 路径与配置目录" stacked hint="每行一条，格式：mpv 可执行文件;[配置目录]。不写配置目录时按 mpv 默认规则查找。">
                        <el-input
                            v-model="mpvPath"
                            type="textarea"
                            :rows="3"
                            :disabled="String(externalMpv) !== 'on'"
                            placeholder="C:\App\mpv\mpv.exe&#10;/usr/bin/mpv;/usr/local/mpv/portable_config"
                            @change="saveSetting('mpv_path')"
                        />
                    </SettingRow>
                    <SettingRow label="MPV 参数" stacked>
                        <template #hint>
                            <span class="t-faint">每行一个 key=value，会追加到启动参数上。</span>
                        </template>
                        <div class="doc-links">
                            <el-button plain size="small" @click="invokeApi.open_url('https://mpv.io/manual/stable/')">官方文档</el-button>
                            <el-button plain size="small" @click="invokeApi.open_url('https://hooke007.github.io/official_man/mpv.html')">中文文档</el-button>
                            <el-button plain size="small" @click="invokeApi.open_folder('inner_mpv')">打开内置 mpv 目录</el-button>
                            <el-button plain size="small" @click="invokeApi.open_file('keymap')">快捷键示意图</el-button>
                        </div>
                        <el-input
                            v-model="mpvArgs"
                            type="textarea"
                            :rows="4"
                            placeholder="ontop=no&#10;volume=130&#10;demuxer-max-bytes=512MiB"
                            @change="saveSetting('mpv_args')"
                        />
                    </SettingRow>
                </SettingBlock>
            </el-tab-pane>

            <!-- ————————— 追踪 ————————— -->
            <el-tab-pane label="追踪" name="track">
                <SettingBlock
                    v-for="provider in trackers"
                    :key="provider.key"
                    :title="provider.title"
                    :caption="provider.caption"
                >
                    <SettingRow label="同步开关" :hint="provider.switchHint">
                        <el-switch
                            v-model="provider.enabled.value"
                            active-value="on"
                            inactive-value="off"
                            @change="saveSetting(provider.key + '.enabled')"
                        />
                    </SettingRow>

                    <SettingRow v-if="provider.needsAuth" label="账号授权" :hint="provider.authHint">
                        <template v-if="provider.username.value">
                            <span class="num">{{ provider.username.value }}</span>
                            <el-button type="danger" plain size="small" @click="provider.revoke()">删除授权</el-button>
                        </template>
                        <el-button
                            type="primary"
                            plain
                            size="small"
                            :loading="provider.authLoading.value"
                            @click="provider.authorize()"
                        >
                            {{ provider.authLabel.value }}
                        </el-button>
                    </SettingRow>

                    <SettingRow v-if="provider.syncUrl" label="同步地址" :hint="provider.syncUrlHint">
                        <el-input
                            v-model="provider.syncUrl.value"
                            placeholder="https://yamtrack.example.com/webhook/emby/xxxx"
                            style="width: min(34rem, 100%)"
                            @change="saveSetting(provider.key + '.syncUrl')"
                        />
                    </SettingRow>

                    <SettingRow label="走哪个代理">
                        <el-select
                            v-model="provider.proxy.value"
                            style="width: 18rem"
                            @change="saveSetting(provider.key + '.proxy')"
                        >
                            <el-option label="不使用代理" value="no" />
                            <el-option :label="`跟随全局浏览代理（${globalBrowseName}）`" value="followBrowse" />
                            <el-option :label="`跟随全局播放代理（${globalPlayName}）`" value="followPlay" />
                            <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                        </el-select>
                    </SettingRow>
                </SettingBlock>
            </el-tab-pane>

            <!-- ————————— 网络 ————————— -->
            <el-tab-pane label="网络" name="network">
                <SettingBlock title="全局默认代理" caption="线路没有单独指定时按这里的设置走">
                    <SettingRow label="媒体库浏览">
                        <el-select v-model="globalBrowseProxyId" style="width: 18rem" @change="onGlobalBrowseChange">
                            <el-option label="不使用代理" value="no" />
                            <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                        </el-select>
                    </SettingRow>
                    <SettingRow label="媒体流播放">
                        <el-select v-model="globalPlayProxyId" style="width: 18rem" @change="onGlobalPlayChange">
                            <el-option label="不使用代理" value="no" />
                            <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                        </el-select>
                    </SettingRow>
                    <SettingRow label="应用数据" hint="图标库、自动更新等应用自身的出网请求">
                        <el-select v-model="appProxyId" style="width: 18rem" @change="onAppProxyChange">
                            <el-option label="不使用代理" value="no" />
                            <el-option :label="`跟随全局浏览代理（${globalBrowseName}）`" value="followBrowse" />
                            <el-option :label="`跟随全局播放代理（${globalPlayName}）`" value="followPlay" />
                            <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                        </el-select>
                    </SettingRow>
                </SettingBlock>

                <SettingBlock title="代理服务器" caption="推荐使用 http 代理；socks5 在部分服务上不稳定">
                    <DataTable
                        :rows="proxies"
                        empty-title="还没有代理服务器"
                        empty-hint="需要走代理才能访问的服务器，先在这里添加一条。"
                    >
                        <template #tools>
                            <el-button type="primary" plain size="small" @click="openProxyDialog()">添加代理</el-button>
                        </template>
                        <template #default="{ row }">
                            <span class="cell cell--name">{{ row.name }}</span>
                            <span class="tag">{{ row.proxy_type }}</span>
                            <span class="cell cell--addr mono">{{ row.addr }}</span>
                            <span v-if="row.username" class="cell t-faint">{{ row.username }}</span>
                            <span v-if="row.location" class="cell t-faint">{{ row.location }}</span>
                        </template>
                        <template #ops="{ row, index }">
                            <el-button plain size="small" :loading="checkingProxy[row.id!]" @click="checkProxy(row.id!)">
                                检测出口
                            </el-button>
                            <el-button plain size="small" @click="openProxyDialog(index)">编辑</el-button>
                            <el-button plain type="danger" size="small" @click="removeProxy(index)">删除</el-button>
                        </template>
                    </DataTable>
                </SettingBlock>

                <SettingBlock title="反代服务器" caption="地址末尾会自动补 /，请求地址 = 反代地址 + 原始路径">
                    <DataTable
                        :rows="reverseProxies"
                        empty-title="还没有反代服务器"
                        empty-hint="常见做法是把内网 Emby 暴露到一个可访问的域名，在这里登记。"
                    >
                        <template #tools>
                            <el-button type="primary" plain size="small" @click="openReverseDialog()">添加反代</el-button>
                        </template>
                        <template #default="{ row }">
                            <span class="cell cell--name">{{ row.name }}</span>
                            <span class="cell cell--addr mono">{{ row.url }}</span>
                            <span v-if="row.location" class="cell t-faint">{{ row.location }}</span>
                        </template>
                        <template #ops="{ row, index }">
                            <el-button plain size="small" :loading="checkingReverse[row.id!]" @click="checkReverse(row.id!)">
                                检测出口
                            </el-button>
                            <el-button plain size="small" @click="openReverseDialog(index)">编辑</el-button>
                            <el-button plain type="danger" size="small" @click="removeReverse(index)">删除</el-button>
                        </template>
                    </DataTable>
                </SettingBlock>

                <SettingBlock title="线路代理" caption="每台服务器的每条线路可以单独指定">
                    <DataTable
                        :rows="lines"
                        empty-title="还没有线路"
                        empty-hint="线路随服务器一起创建，添加服务器后这里就会出现。"
                    >
                        <template #default="{ row }">
                            <span class="cell cell--name">{{ row.emby_server_name }}</span>
                            <span class="cell">{{ row.name }}</span>
                            <span class="cell cell--addr mono">{{ row.base_url }}</span>
                        </template>
                        <template #ops="{ row }">
                            <el-select
                                v-model="row.reverse_proxy_id"
                                size="small"
                                style="width: 9rem"
                                @change="() => saveLineProxy(row)"
                            >
                                <el-option label="无反代" value="no" />
                                <el-option v-for="item in reverseProxies" :key="item.id" :label="item.name" :value="item.id!" />
                            </el-select>
                            <el-select
                                v-model="row.browse_proxy_id"
                                size="small"
                                style="width: 9.5rem"
                                @change="() => saveLineProxy(row)"
                            >
                                <el-option label="不代理" value="no" />
                                <el-option :label="`跟随全局（${globalBrowseName}）`" value="follow" />
                                <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                            </el-select>
                            <el-select
                                v-model="row.play_proxy_id"
                                size="small"
                                style="width: 9.5rem"
                                @change="() => saveLineProxy(row)"
                            >
                                <el-option label="不代理" value="no" />
                                <el-option :label="`跟随全局（${globalPlayName}）`" value="follow" />
                                <el-option v-for="item in proxies" :key="item.id" :label="item.name" :value="item.id!" />
                            </el-select>
                        </template>
                    </DataTable>
                </SettingBlock>

                <SettingBlock title="图标库" caption="服务器图标从这里挑选">
                    <DataTable
                        :rows="iconLibraries"
                        empty-title="还没有图标库"
                        empty-hint="图标库是一份 JSON 清单，添加后即可在服务器右键菜单里换图标。"
                    >
                        <template #tools>
                            <el-button type="primary" plain size="small" @click="openIconLibraryDialog()">添加图标库</el-button>
                        </template>
                        <template #default="{ row }">
                            <span class="cell cell--name">{{ row.name }}</span>
                            <span class="cell cell--addr mono">{{ row.url }}</span>
                        </template>
                        <template #ops="{ index }">
                            <el-button plain size="small" @click="openIconLibraryDialog(index)">编辑</el-button>
                            <el-button plain type="danger" size="small" @click="removeIconLibrary(index)">删除</el-button>
                        </template>
                    </DataTable>
                </SettingBlock>
            </el-tab-pane>

            <!-- ————————— 缓存与日志 ————————— -->
            <el-tab-pane label="存储" name="storage">
                <SettingBlock title="保留策略">
                    <SettingRow label="日志保存天数">
                        <el-input-number v-model="logDays" :min="1" :controls="false" style="width: 8rem" @change="saveSetting('logStoredDays')">
                            <template #suffix><span class="unit">天</span></template>
                        </el-input-number>
                        <el-button plain @click="invokeApi.open_folder('log')">打开日志目录</el-button>
                    </SettingRow>
                    <SettingRow label="封面图保存天数">
                        <el-input-number v-model="coverDays" :min="1" :controls="false" style="width: 8rem" @change="saveSetting('coverImageStoredDays')">
                            <template #suffix><span class="unit">天</span></template>
                        </el-input-number>
                        <el-button plain @click="invokeApi.open_folder('cache')">打开缓存目录</el-button>
                    </SettingRow>
                    <SettingRow label="图标保存天数">
                        <el-input-number v-model="iconDays" :min="1" :controls="false" style="width: 8rem" @change="saveSetting('iconStoredDays')">
                            <template #suffix><span class="unit">天</span></template>
                        </el-input-number>
                        <el-button plain :loading="cleaningIcons" @click="cleanIconCache">清除图标缓存</el-button>
                    </SettingRow>
                </SettingBlock>

                <SettingBlock title="图片" caption="排查加载问题时可以逐个关掉">
                    <SettingRow label="图片缓存" hint="关闭后每次都重新向服务器取图，只影响速度">
                        <el-switch
                            v-model="imageCacheDisabled"
                            active-value="on"
                            inactive-value="off"
                            active-text="禁用缓存"
                            inactive-text="启用缓存"
                            @change="saveSetting('disabled_image_cache')"
                        />
                    </SettingRow>
                    <SettingRow label="图片加载" hint="完全不请求图片，适合极慢网络">
                        <el-switch
                            v-model="imageDisabled"
                            active-value="on"
                            inactive-value="off"
                            active-text="不请求图片"
                            inactive-text="正常加载"
                            @change="saveSetting('disabledImage')"
                        />
                    </SettingRow>
                </SettingBlock>

                <SettingBlock title="按服务器清理" caption="清理后首次打开会重新下载封面">
                    <DataTable
                        :rows="embyServers"
                        empty-title="还没有服务器"
                        empty-hint="添加服务器后才能清理它的图片缓存。"
                    >
                        <template #tools>
                            <el-button type="primary" plain size="small" :loading="cleaningAll" @click="cleanAllCaches">
                                清除全部缓存
                            </el-button>
                        </template>
                        <template #default="{ row }">
                            <span class="cell cell--name">{{ row.server_name }}</span>
                            <span class="cell t-faint">{{ row.username || '未登录' }}</span>
                        </template>
                        <template #ops="{ row }">
                            <el-button plain size="small" :loading="cleaningServer[row.id!]" @click="cleanServerCache(row.id!)">
                                清除缓存
                            </el-button>
                        </template>
                    </DataTable>
                </SettingBlock>
            </el-tab-pane>
        </el-tabs>

        <!-- ——— 代理 / 反代 / 图标库 编辑弹窗 ——— -->
        <el-dialog
            :model-value="!!proxyDraft"
            :title="proxyDraft?.id ? '编辑代理' : '添加代理'"
            width="32rem"
            @update:model-value="(v: boolean) => !v && (proxyDraft = null)"
        >
            <div class="dialog-form">
                <div class="field">
                    <span class="field__label">名称</span>
                    <el-input v-model="proxyDraft!.name" placeholder="例如 香港节点" />
                </div>
                <div class="field">
                    <span class="field__label">类型</span>
                    <el-select v-model="proxyDraft!.proxy_type" style="width: 100%">
                        <el-option label="http" value="http" />
                        <el-option label="https" value="https" />
                        <el-option label="socks5" value="socks5" />
                    </el-select>
                </div>
                <div class="field">
                    <span class="field__label">地址</span>
                    <el-input v-model="proxyDraft!.addr" placeholder="ip:port" />
                </div>
                <div class="field">
                    <span class="field__label">用户名（可选）</span>
                    <el-input v-model="proxyDraft!.username" />
                </div>
                <div class="field">
                    <span class="field__label">密码（可选）</span>
                    <el-input v-model="proxyDraft!.password" type="password" show-password />
                </div>
            </div>
            <template #footer>
                <el-button @click="proxyDraft = null">取消</el-button>
                <el-button type="primary" @click="saveProxy">保存</el-button>
            </template>
        </el-dialog>

        <el-dialog
            :model-value="!!reverseDraft"
            :title="reverseDraft?.id ? '编辑反代' : '添加反代'"
            width="32rem"
            @update:model-value="(v: boolean) => !v && (reverseDraft = null)"
        >
            <div class="dialog-form">
                <div class="field">
                    <span class="field__label">名称</span>
                    <el-input v-model="reverseDraft!.name" placeholder="例如 公网入口" />
                </div>
                <div class="field">
                    <span class="field__label">地址</span>
                    <el-input v-model="reverseDraft!.url" placeholder="https://proxy.example.org/" />
                    <span class="field__hint">末尾会自动补 /</span>
                </div>
            </div>
            <template #footer>
                <el-button @click="reverseDraft = null">取消</el-button>
                <el-button type="primary" @click="saveReverse">保存</el-button>
            </template>
        </el-dialog>

        <el-dialog
            :model-value="!!iconLibraryDraft"
            :title="iconLibraryDraft?.id ? '编辑图标库' : '添加图标库'"
            width="32rem"
            @update:model-value="(v: boolean) => !v && (iconLibraryDraft = null)"
        >
            <div class="dialog-form">
                <div class="field">
                    <span class="field__label">名称</span>
                    <el-input v-model="iconLibraryDraft!.name" placeholder="图标库名称" />
                </div>
                <div class="field">
                    <span class="field__label">清单地址</span>
                    <el-input v-model="iconLibraryDraft!.url" placeholder="https://example.com/icons.json" />
                </div>
            </div>
            <template #footer>
                <el-button @click="iconLibraryDraft = null">取消</el-button>
                <el-button type="primary" @click="saveIconLibrary">保存</el-button>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { computed, h, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { ElButton, ElMessage, ElMessageBox, ElNotification } from 'element-plus';
import { listen } from '@tauri-apps/api/event';

import SettingBlock from '../components/settings/SettingBlock.vue';
import SettingRow from '../components/settings/SettingRow.vue';
import DataTable from '../components/settings/DataTable.vue';

import appApi from '../api/appApi';
import invokeApi from '../api/invokeApi';
import { generateGuid } from '../util/uuid_util';
import { useEventBus } from '../store/eventBus';
import { useGlobalSetting, type GlobalSetting } from '../composables/useGlobalSetting';
import { useRuntimeConfig } from '../store/runtimeConfig';
import { useProxyServer, type ProxyServer } from '../store/db/proxyServer';
import { useReverseProxyServer, type ReverseProxyServer } from '../store/db/reverseProxyServer';
import { useEmbyServer, type EmbyServer } from '../store/db/embyServer';
import { useEmbyLine, type EmbyLine } from '../store/db/embyLine';
import { useEmbyIconLibrary, type EmbyIconLibrary } from '../store/db/embyIconLibrary';
import { useGlobalConfig } from '../store/db/globalConfig';

/**
 * 设置页。
 *
 * 组织原则：
 *  - 每个设置项是「名称 + 说明 + 控件」，说明放名称下方而不是 placeholder，
 *    滚动时上下文不会丢。
 *  - 一个面板一组相关设置，组与组之间只靠间距分开，不套卡片。
 *  - 所有保存都走 useGlobalSetting，默认值、回滚、提示统一。
 */

const runtimeConfig = useRuntimeConfig().runtimeConfig;
const bus = useEventBus();
const config = useGlobalConfig();

const activeTab = ref('general');
const configDir = computed(() => runtimeConfig?.app_config.database_url || '应用配置目录');

/**
 * 设置项登记表：key -> 句柄。
 *
 * 模板里的 ref 会被自动解包，没法把句柄本身传给事件处理函数，
 * 因此模板只写 key，由这里查表保存。key 也正好是配置名，便于排查。
 */
const settings = new Map<string, GlobalSetting<string | number>>();

function registerSetting<T extends string | number>(key: string, handle: GlobalSetting<T>): GlobalSetting<T> {
    settings.set(key, handle as GlobalSetting<string | number>);
    return handle;
}

function saveSetting(key: string): Promise<void> {
    const handle = settings.get(key);
    if (!handle) {
        console.warn('[setting] 未登记的设置项:', key);
        return Promise.resolve();
    }
    return handle.save();
}

/* ————————————————— 通用 ————————————————— */

const dangerCerts = registerSetting('danger_accept_invalid_certs', useGlobalSetting('danger_accept_invalid_certs', 'false'));

const checkingUpdate = ref(false);

async function checkUpdate() {
    checkingUpdate.value = true;
    try {
        const hasUpdate = await invokeApi.updater();
        if (hasUpdate) {
            ElNotification.success({
                title: '新版本已就绪',
                message: h('p', { style: 'display:flex;align-items:center;gap:12px' }, [
                    h('span', null, '重启应用后生效'),
                    h(ElButton, { size: 'small', type: 'primary', onClick: () => invokeApi.restartApp() }, () => '现在重启'),
                ]),
                position: 'bottom-right',
            });
        } else {
            ElMessage.success('已经是最新版本');
        }
    } finally {
        checkingUpdate.value = false;
    }
}

/* ————————————————— 播放 ————————————————— */

const versionPolicy = registerSetting('play_version_auto_select_policy', useGlobalSetting('play_version_auto_select_policy', 'high-resolution'));
const prefetch = registerSetting('prefetch_playlist', useGlobalSetting('prefetch_playlist', 'no'));
const isPlayback = registerSetting('play_param_IsPlayback', useGlobalSetting('play_param_IsPlayback', 'true'));
const speedEnabled = registerSetting('cache_speed_enabled', useGlobalSetting('cache_speed_enabled', 'yes'));
const speedStyle = registerSetting('cache_speed_ass_style', useGlobalSetting('cache_speed_ass_style', ''));

const cacheSeconds = registerSetting('mpv_cache_seconds', useGlobalSetting<number>('mpv_cache_seconds', 0, { parse: Number }));
const cacheMinBytes = registerSetting('mpv_cache_min_bytes', useGlobalSetting<number>('mpv_cache_min_bytes', 0, { parse: Number }));
const cacheMaxBytes = registerSetting('mpv_cache_max_bytes', useGlobalSetting<number>('mpv_cache_max_bytes', 0, { parse: Number }));
const cacheBackSeconds = registerSetting('mpv_cache_back_seconds', useGlobalSetting<number>('mpv_cache_back_seconds', 0, { parse: Number }));
const cacheBackMinBytes = registerSetting('mpv_cache_back_min_bytes', useGlobalSetting<number>('mpv_cache_back_min_bytes', 0, { parse: Number }));
const cacheBackMaxBytes = registerSetting('mpv_cache_back_max_bytes', useGlobalSetting<number>('mpv_cache_back_max_bytes', 0, { parse: Number }));

const externalMpv = registerSetting('external_mpv_switch', useGlobalSetting('external_mpv_switch', 'off'));
const mpvPath = registerSetting('mpv_path', useGlobalSetting('mpv_path', ''));
const mpvArgs = registerSetting('mpv_args', useGlobalSetting('mpv_args', ''));

/* ————————————————— 追踪 ————————————————— */

const globalBrowseName = ref('不使用代理');
const globalPlayName = ref('不使用代理');

/**
 * 注意：useGlobalSetting 返回的就是 ref（额外挂了 save / load）。
 * 模板里 v-model 直接绑它，脚本里用 .value 读写，不要再写 .value.value。
 */
const globalBrowseProxyId = registerSetting('global_browse_proxy_id', useGlobalSetting('global_browse_proxy_id', 'no'));
const globalPlayProxyId = registerSetting('global_play_proxy_id', useGlobalSetting('global_play_proxy_id', 'no'));

const appProxyId = registerSetting('app_proxy_id', useGlobalSetting('app_proxy_id', 'followBrowse'));


/**
 * 全局代理是「线路未单独指定时的兜底」。
 * 改动后要同时刷新名字缓存，并广播一次，让外壳的顶栏下拉跟着更新。
 */
function onGlobalBrowseChange() {
    return updateGlobalProxy('global_browse_proxy_id', globalBrowseProxyId.value, globalBrowseName);
}

function onGlobalPlayChange() {
    return updateGlobalProxy('global_play_proxy_id', globalPlayProxyId.value, globalPlayName);
}

function onAppProxyChange() {
    return saveSetting('app_proxy_id');
}

async function updateGlobalProxy(key: string, value: string, nameRef: { value: string }) {
    try {
        const existing = await config.getGlobalConfig(key);
        if (existing) {
            await config.updateGlobalConfig({ ...existing, config_value: value });
        } else {
            await config.addGlobalConfig({ config_key: key, config_value: value });
        }
        nameRef.value = await useProxyServer().getProxyServerName(value);
        bus.emit('GlobalProxyChanged', {});
    } catch (e) {
        ElMessage.error('保存全局代理失败 ' + e);
    }
}

interface TrackerConfig {
    key: string;
    title: string;
    caption: string;
    switchHint: string;
    enabled: GlobalSetting<string>;
    proxy: GlobalSetting<string>;
    username: { value: string };
    authLabel: { value: string };
    authLoading: { value: boolean };
    usernameKey: string;
    needsAuth: boolean;
    authHint: string;
    tokenKeys: string[];
    revokeUrl: string;
    authorize: () => void;
    revoke: () => void;
    syncUrl?: GlobalSetting<string>;
    syncUrlHint?: string;
}

const traktAuthLoading = ref(false);
const traktUsername = ref('');
const traktAuthLabel = ref('去授权');
const simklAuthLoading = ref(false);
const simklUsername = ref('');
const simklAuthLabel = ref('去授权');

const traktEnabled = registerSetting('trakt_sync_switch', useGlobalSetting<string>('trakt_sync_switch', 'on', { immediate: false }));
const traktProxy = registerSetting('trakt_proxy_id', useGlobalSetting<string>('trakt_proxy_id', 'followBrowse', { immediate: false }));
const simklEnabled = registerSetting('simkl_sync_switch', useGlobalSetting<string>('simkl_sync_switch', 'on', { immediate: false }));
const simklProxy = registerSetting('simkl_proxy_id', useGlobalSetting<string>('simkl_proxy_id', 'followBrowse', { immediate: false }));
const yamtrackEnabled = registerSetting('yamtrack_sync_switch', useGlobalSetting<string>('yamtrack_sync_switch', 'on', { immediate: false }));
const yamtrackProxy = registerSetting('yamtrack_proxy_id', useGlobalSetting<string>('yamtrack_proxy_id', 'followBrowse', { immediate: false }));
const yamtrackUrl = registerSetting('yamtrack_sync_url', useGlobalSetting<string>('yamtrack_sync_url', '', { immediate: false }));

const unlisteners = ref<Array<() => void>>([]);

async function refreshTrackerInfo() {
    try {
        traktUsername.value = await config.getGlobalConfigValue('trakt_username');
        simklUsername.value = await config.getGlobalConfigValue('simkl_username');
        if (!traktAuthLoading.value) {
            traktAuthLabel.value = traktUsername.value ? '换个账号 / 重新授权' : '去授权';
        }
        if (!simklAuthLoading.value) {
            simklAuthLabel.value = simklUsername.value ? '换个账号 / 重新授权' : '去授权';
        }
    } catch (e) {
        ElMessage.error('读取追踪账号失败 ' + e);
    }
}

function makeAuthorize(options: {
    provider: 'trakt' | 'simkl';
    loading: { value: boolean };
    label: { value: string };
    usernameKey: string;
    eventName: string;
}) {
    return async () => {
        options.loading.value = true;
        options.label.value = '等待浏览器授权…';
        const unlisten = await listen<string>(options.eventName, async () => {
            options.label.value = '授权成功';
            await config.refreshCache(options.usernameKey);
            await refreshTrackerInfo();
            options.loading.value = false;
        });
        unlisteners.value.push(unlisten);
        try {
            if (options.provider === 'trakt') {
                await invokeApi.go_trakt_auth();
            } else {
                await invokeApi.go_simkl_auth();
            }
            ElMessage.success('已打开浏览器，完成授权后会自动回到这里');
        } catch (e) {
            ElMessage.error('发起授权失败 ' + e);
            options.loading.value = false;
        }
    };
}

function makeRevoke(options: {
    title: string;
    usernameKey: string;
    tokenKeys: string[];
    revokeUrl: string;
}) {
    return () => {
        ElMessageBox.confirm(`删除「${options.title}」的授权？`, '删除授权', {
            confirmButtonText: '删除',
            cancelButtonText: '取消',
            type: 'warning',
        })
            .then(async () => {
                for (const key of [options.usernameKey, ...options.tokenKeys]) {
                    await config.delGlobalConfig(key);
                }
                await refreshTrackerInfo();
                ElMessageBox.alert(
                    `已删除本地授权。建议同时到 ${options.revokeUrl} 吊销应用授权，这会清掉该应用拿到的全部令牌。`,
                    '还需要一步',
                );
            })
            .catch(() => undefined);
    };
}

/**
 * 追踪服务的句柄也登记进同一张表，key 用 `provider.field`，
 * 这样模板里所有 @change 都能走 saveSetting(key) 这一条路径。
 */
function registerTracker(provider: TrackerConfig): TrackerConfig {
    settings.set(`${provider.key}.enabled`, provider.enabled as GlobalSetting<string | number>);
    settings.set(`${provider.key}.proxy`, provider.proxy as GlobalSetting<string | number>);
    if (provider.syncUrl) {
        settings.set(`${provider.key}.syncUrl`, provider.syncUrl as GlobalSetting<string | number>);
    }
    return provider;
}

const trackers: TrackerConfig[] = [
    {
        key: 'trakt',
        title: 'Trakt',
        caption: '播放时上报开始/停止事件',
        switchHint: '关闭后不再向 Trakt 发送任何请求',
        enabled: traktEnabled,
        proxy: traktProxy,
        username: traktUsername,
        authLabel: traktAuthLabel,
        authLoading: traktAuthLoading,
        usernameKey: 'trakt_username',
        needsAuth: true,
        authHint: '需要浏览器完成 OAuth，播放进度超过 80% 才会在网页端出现记录',
        tokenKeys: ['trakt_refresh_token', 'trakt_expires_in', 'trakt_access_token', 'trakt_redirect_uri'],
        revokeUrl: 'https://trakt.tv/oauth/authorized_applications',
        authorize: makeAuthorize({
            provider: 'trakt',
            loading: traktAuthLoading,
            label: traktAuthLabel,
            usernameKey: 'trakt_username',
            eventName: 'trakt_auth',
        }),
        revoke: makeRevoke({
            title: 'Trakt',
            usernameKey: 'trakt_username',
            tokenKeys: ['trakt_refresh_token', 'trakt_expires_in', 'trakt_access_token', 'trakt_redirect_uri'],
            revokeUrl: 'https://trakt.tv/oauth/authorized_applications',
        }),
    },
    {
        key: 'simkl',
        title: 'Simkl',
        caption: '动画与剧集的另一套记录',
        switchHint: '关闭后不再向 Simkl 发送任何请求',
        enabled: simklEnabled,
        proxy: simklProxy,
        username: simklUsername,
        authLabel: simklAuthLabel,
        authLoading: simklAuthLoading,
        usernameKey: 'simkl_username',
        needsAuth: true,
        authHint: '需要浏览器完成 OAuth',
        tokenKeys: ['simkl_access_token', 'simkl_redirect_uri'],
        revokeUrl: 'https://simkl.com/settings/connected-apps/',
        authorize: makeAuthorize({
            provider: 'simkl',
            loading: simklAuthLoading,
            label: simklAuthLabel,
            usernameKey: 'simkl_username',
            eventName: 'simkl_auth',
        }),
        revoke: makeRevoke({
            title: 'Simkl',
            usernameKey: 'simkl_username',
            tokenKeys: ['simkl_access_token', 'simkl_redirect_uri'],
            revokeUrl: 'https://simkl.com/settings/connected-apps/',
        }),
    },
    {
        key: 'yamtrack',
        title: 'YamTrack',
        caption: '通过 Webhook 上报，无需账号授权',
        switchHint: '关闭后不再调用 Webhook',
        enabled: yamtrackEnabled,
        proxy: yamtrackProxy,
        username: { value: '' },
        authLabel: { value: '' },
        authLoading: { value: false },
        usernameKey: '',
        needsAuth: false,
        authHint: '',
        tokenKeys: [],
        revokeUrl: '',
        syncUrl: yamtrackUrl,
        syncUrlHint: '在 YamTrack 的 Emby Integrations 里复制完整地址',
        authorize: () => undefined,
        revoke: () => undefined,
    },
].map(registerTracker);

/* ————————————————— 列表数据 ————————————————— */

const proxies = ref<ProxyServer[]>([]);
const reverseProxies = ref<ReverseProxyServer[]>([]);
const lines = ref<EmbyLine[]>([]);
const iconLibraries = ref<EmbyIconLibrary[]>([]);
const embyServers = ref<EmbyServer[]>([]);

async function reloadAll() {
    try {
        // 代理名单是「跟随全局」等选项的名称来源，必须最先就绪
        await Promise.all([useProxyServer().initCache(), useReverseProxyServer().initCache()]);
        const [proxyList, reverseList, lineList, libraryList, serverList] = await Promise.all([
            useProxyServer().listAllProxyServer(),
            useReverseProxyServer().listAllReverseProxyServer(),
            useEmbyLine().listAllEmbyLine(),
            useEmbyIconLibrary().listAllEmbyIconLibrary(),
            useEmbyServer().listAllEmbyServer(),
        ]);
        proxies.value = proxyList;
        reverseProxies.value = reverseList;
        lines.value = lineList;
        iconLibraries.value = libraryList;
        embyServers.value = serverList.sort((a, b) => (a.order_by ?? 0) - (b.order_by ?? 0));

        globalBrowseName.value = await useProxyServer().getProxyServerName(globalBrowseProxyId.value);
        globalPlayName.value = await useProxyServer().getProxyServerName(globalPlayProxyId.value);
    } catch (e) {
        ElMessage.error('读取网络配置失败 ' + e);
    }
}

async function saveLineProxy(line: EmbyLine) {
    // 选了反代就不再叠加代理，避免两条链路同时生效
    if (line.reverse_proxy_id && line.reverse_proxy_id !== 'no') {
        line.browse_proxy_id = 'no';
        line.play_proxy_id = 'no';
    }
    try {
        await useEmbyLine().updateEmbyLine(line);
        bus.emit('EmbyLineChanged', {});
        ElMessage.success('已保存');
    } catch (e) {
        ElMessage.error('保存线路代理失败 ' + e);
    }
}

/* ————————————————— 代理 / 反代 / 图标库 增删改 ————————————————— */

const proxyDraft = ref<ProxyServer | null>(null);
const reverseDraft = ref<ReverseProxyServer | null>(null);
const iconLibraryDraft = ref<EmbyIconLibrary | null>(null);

const checkingProxy = reactive<Record<string, boolean>>({});
const checkingReverse = reactive<Record<string, boolean>>({});
const cleaningIcons = ref(false);
const cleaningAll = ref(false);
const cleaningServer = reactive<Record<string, boolean>>({});

function openProxyDialog(index?: number) {
    proxyDraft.value = index === undefined ? { proxy_type: 'http' } : { ...proxies.value[index] };
}

async function saveProxy() {
    const draft = proxyDraft.value;
    if (!draft) {
        return;
    }
    try {
        if (draft.id) {
            await useProxyServer().updateProxyServer(draft);
        } else {
            await useProxyServer().addProxyServer({ ...draft, id: generateGuid() });
        }
        bus.emit('ProxyServerChanged', {});
        await reloadAll();
        proxyDraft.value = null;
        ElMessage.success('已保存');
    } catch (e) {
        ElMessage.error('保存代理失败 ' + e);
    }
}

function removeProxy(index: number) {
    const target = proxies.value[index];
    ElMessageBox.confirm(`删除代理「${target.name}」？正在使用它的线路会退回「不代理」。`, '删除代理', {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
    })
        .then(async () => {
            await useProxyServer().delProxyServer(target.id!);
            bus.emit('ProxyServerChanged', {});
            await reloadAll();
            ElMessage.success('已删除');
        })
        .catch(() => undefined);
}

function openReverseDialog(index?: number) {
    reverseDraft.value = index === undefined ? {} : { ...reverseProxies.value[index] };
}

async function saveReverse() {
    const draft = reverseDraft.value;
    if (!draft) {
        return;
    }
    // 反代地址末尾必须是 /，否则拼接出来的路径会少一层
    if (draft.url && !draft.url.trim().endsWith('/')) {
        draft.url = draft.url.trim() + '/';
    }
    try {
        if (draft.id) {
            await useReverseProxyServer().updateReverseProxyServer(draft);
        } else {
            await useReverseProxyServer().addReverseProxyServer({ ...draft, id: generateGuid() });
        }
        bus.emit('ReverseProxyServerChanged', {});
        await reloadAll();
        reverseDraft.value = null;
        ElMessage.success('已保存');
    } catch (e) {
        ElMessage.error('保存反代失败 ' + e);
    }
}

function removeReverse(index: number) {
    const target = reverseProxies.value[index];
    ElMessageBox.confirm(`删除反代「${target.name}」？`, '删除反代', {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
    })
        .then(async () => {
            await useReverseProxyServer().delReverseProxyServer(target.id!);
            bus.emit('ReverseProxyServerChanged', {});
            await reloadAll();
            ElMessage.success('已删除');
        })
        .catch(() => undefined);
}

function openIconLibraryDialog(index?: number) {
    iconLibraryDraft.value = index === undefined ? {} : { ...iconLibraries.value[index] };
}

async function saveIconLibrary() {
    const draft = iconLibraryDraft.value;
    if (!draft) {
        return;
    }
    try {
        if (draft.id) {
            await useEmbyIconLibrary().updateEmbyIconLibrary(draft);
        } else {
            await useEmbyIconLibrary().addEmbyIconLibrary({ ...draft, id: generateGuid() });
        }
        await reloadAll();
        iconLibraryDraft.value = null;
        ElMessage.success('已保存');
    } catch (e) {
        ElMessage.error('保存图标库失败 ' + e);
    }
}

function removeIconLibrary(index: number) {
    const target = iconLibraries.value[index];
    ElMessageBox.confirm(`删除图标库「${target.name}」？`, '删除图标库', {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
    })
        .then(async () => {
            await useEmbyIconLibrary().delEmbyIconLibrary(target.id!);
            await reloadAll();
            ElMessage.success('已删除');
        })
        .catch(() => undefined);
}

async function checkProxy(id: string) {
    checkingProxy[id] = true;
    try {
        const geo = await appApi.getProxyLocation(id);
        const target = proxies.value.find(item => item.id === id);
        if (target) {
            target.location = `${geo.ip} ${geo.country_code}`;
        }
    } catch (e) {
        ElMessage.error('检测失败，通常是代理地址或账号密码不对：' + e);
    } finally {
        checkingProxy[id] = false;
    }
}

async function checkReverse(id: string) {
    checkingReverse[id] = true;
    try {
        const geo = await appApi.getReverseProxyLocation(id);
        const target = reverseProxies.value.find(item => item.id === id);
        if (target) {
            target.location = `${geo.ip} ${geo.country_code}`;
        }
    } catch (e) {
        ElMessage.error('检测失败，通常是反代地址不可达：' + e);
    } finally {
        checkingReverse[id] = false;
    }
}

/* ————————————————— 存储 ————————————————— */

const logDays = registerSetting('logStoredDays', useGlobalSetting<number>('logStoredDays', 30, { parse: Number }));
const coverDays = registerSetting('coverImageStoredDays', useGlobalSetting<number>('coverImageStoredDays', 30, { parse: Number }));
const iconDays = registerSetting('iconStoredDays', useGlobalSetting<number>('iconStoredDays', 365, { parse: Number }));
const imageCacheDisabled = registerSetting('disabled_image_cache', useGlobalSetting('disabled_image_cache', 'off'));
const imageDisabled = registerSetting('disabledImage', useGlobalSetting('disabledImage', 'off'));

async function cleanIconCache() {
    cleaningIcons.value = true;
    try {
        await invokeApi.clean_icon_cache();
        ElMessage.success('图标缓存已清理');
    } finally {
        cleaningIcons.value = false;
    }
}

async function cleanServerCache(id: string) {
    cleaningServer[id] = true;
    try {
        await invokeApi.clean_emby_image_cache(id);
        ElMessage.success('已清理该服务器的图片缓存');
    } finally {
        cleaningServer[id] = false;
    }
}

async function cleanAllCaches() {
    cleaningAll.value = true;
    try {
        await invokeApi.clean_emby_image_cache();
        ElMessage.success('已清理全部图片缓存');
    } finally {
        cleaningAll.value = false;
    }
}

/* ————————————————— 生命周期 ————————————————— */

/** 延迟到切到某个分组时才读它的配置，避免一次打一堆 IPC */
async function ensureTabLoaded(tab: string) {
    if (tab === 'track') {
        await Promise.all([traktEnabled.load(), traktProxy.load(), simklEnabled.load(), simklProxy.load(), yamtrackEnabled.load(), yamtrackProxy.load(), yamtrackUrl.load()]);
        await refreshTrackerInfo();
    }
}

onMounted(async () => {
    await reloadAll();
    await Promise.all([dangerCerts.load()]);
    await ensureTabLoaded(activeTab.value);
    unlisteners.value.push(
        (await listen('EmbyServerChanged', () => reloadAll())) as unknown as () => void,
    );
});

onUnmounted(() => {
    for (const off of unlisteners.value) {
        off?.();
    }
});

// 切分组时再读该组的配置，避免一进设置页就打一堆 IPC
watch(activeTab, tab => void ensureTabLoaded(tab));
</script>

<style scoped>
.settings {
    max-width: 68rem;
}

.cache-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.cache-grid :deep(.el-input-number) {
    width: 11rem;
}

.unit {
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.doc-links {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
}

.dialog-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.cell {
    font-size: var(--fs-sm);
}

.cell--name {
    min-width: 8rem;
    color: var(--text);
    font-weight: 500;
}

.cell--addr {
    min-width: 12rem;
    color: var(--text-faint);
    font-size: var(--fs-xs);
}
</style>
