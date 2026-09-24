<template>
    <el-scrollbar class="history-scrollbar" ref="scrollbarRef">
        <div class="history-pane">
            <!-- 筛选卡片：与设置页卡片风格统一 -->
            <div class="table-section-card">
                <div class="table-card-header">
                    <div class="card-title-group">
                        <span class="card-title">播放历史</span>
                        <span class="card-desc">按服务器、剧名、集名筛选播放记录</span>
                    </div>
                </div>
                <div class="table-card-body">
                    <div class="history-filter-bar">
                        <el-select
                            v-model="query.emby_server_id"
                            @change="getPlayHistory"
                            clearable
                            placeholder="筛选服务器"
                            class="filter-server">
                            <el-option v-for="embyServer in embyServers" :key="embyServer.id" :label="embyServer.server_name" :value="embyServer.id"/>
                        </el-select>
                        <el-input v-model="query.series_name" @keyup.enter="getPlayHistory" placeholder="剧名" clearable class="filter-input">
                            <template #prefix>
                                <span class="filter-prefix">剧</span>
                            </template>
                        </el-input>
                        <el-input v-model="query.item_name" @keyup.enter="getPlayHistory" placeholder="集名" clearable class="filter-input">
                            <template #prefix>
                                <span class="filter-prefix">集</span>
                            </template>
                        </el-input>
                    </div>
                </div>
            </div>

            <!-- 表格卡片 -->
            <div class="table-section-card">
                <div class="table-card-body">
                    <el-table :data="list" :row-style="highlightRowFunction" class="custom-data-table history-table">
                        <el-table-column prop="emby_server_name" label="服务器" min-width="140" show-overflow-tooltip />
                        <el-table-column prop="series_name" label="剧" min-width="200" show-overflow-tooltip>
                            <template #default="scope">
                                <el-link @click.prevent="gotoSeries(scope.row.emby_server_id, scope.row.series_id)" :type="scope.row.pinned == 1 ? 'primary' : scope.row.pinned == -1 ? 'danger' : 'default'">{{ scope.row.series_name }}</el-link>
                            </template>
                        </el-table-column>
                        <el-table-column prop="item_name" label="集、电影" min-width="280" show-overflow-tooltip>
                            <template #default="scope">
                                <el-link @click.prevent="gotoEpisodes(scope.row.emby_server_id, scope.row.item_id)" :type="scope.row.pinned == 1 ? 'primary' : scope.row.pinned == -1 ? 'danger' : 'default'">{{ scope.row.item_name }}</el-link>
                            </template>
                        </el-table-column>
                        <el-table-column prop="played_duration" label="播放时长" :formatter="played_duration_formatter" width="110" align="center" />
                        <el-table-column fixed="right" label="Pin" width="80" align="center">
                            <template #default="scope">
                                <el-link :underline="false" @click="pin(scope.row)">
                                    <el-icon :size="16" v-if="scope.row.pinned == 1"><svg-icon name="pin" /></el-icon>
                                    <el-icon :size="16" v-else-if="scope.row.pinned == -1"><svg-icon name="hidden" /></el-icon>
                                    <el-icon :size="16" v-else><svg-icon name="unpin" /></el-icon>
                                </el-link>
                            </template>
                        </el-table-column>
                    </el-table>
                </div>
                <el-pagination
                    class="history-pagination"
                    v-model:current-page="query.page_number"
                    v-model:page-size="query.page_size"
                    layout="total, prev, pager, next, jumper"
                    :total="total"
                    @current-change="handlePageChange"
                    hide-on-single-page
                />
            </div>
        </div>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router';
import { onMounted, ref } from 'vue';
import { ElMessage, ScrollbarInstance } from 'element-plus';
import { PagePlayHistoryParam, PlayHistory, usePlayHistory } from '../store/db/playHistory';
import { secondsToHMS } from '../util/str_util'
import { EmbyServer, useEmbyServer } from '../store/db/embyServer';

const router = useRouter()
function gotoEpisodes(embyServerId: string, episodesId: string) {
    router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
}
function gotoSeries(embyServerId: string, seriesId: string) {
    router.push('/nav/emby/' + embyServerId + '/series/' + seriesId)
}
const scrollbarRef = ref<ScrollbarInstance>()

const embyServers = ref<EmbyServer[]>([])
function listAllEmbyServer() {
    useEmbyServer().listAllEmbyServer().then(list => {
        embyServers.value = list.sort((a, b) => a.order_by! - b.order_by!);
    }).catch(e => ElMessage.error('获取Emby服务器失败' + e))
}
listAllEmbyServer()

const list = ref<PlayHistory[]>([])
const total = ref<number>(0)
const query = ref<PagePlayHistoryParam>({page_number: 1, page_size: 30})
async function getPlayHistory() {
    if (query.value.emby_server_id === '') {
        query.value.emby_server_id = undefined
    }
    if (query.value.series_name === '') {
        query.value.series_name = undefined
    }
    if (query.value.item_name === '') {
        query.value.item_name = undefined
    }
    return usePlayHistory().pagePlayHistory(query.value).then(async response => {
        list.value = response[1]
        total.value = response[0]
        scrollbarRef.value?.setScrollTop(0)
    }).catch(e => ElMessage.error('获取播放历史失败' + e))
}
function handlePageChange(pageNumber: number) {
    query.value.page_number = pageNumber
    getPlayHistory()
}
onMounted(() => getPlayHistory())

function played_duration_formatter(row: PlayHistory) {
    return secondsToHMS(row.played_duration!)
}

function pin(row: PlayHistory) {
    let pinned = row.pinned == 1 ? -1 : row.pinned! + 1
    usePlayHistory().updatePlayHistory({id: row.id, pinned: pinned}).then(() => {
        row.pinned = pinned
    }).catch(e => ElMessage.error('更新失败' + e))
}

function highlightRowFunction({row}: {row: PlayHistory}) {
    return row.pinned == 1 ? 'color: #409EFF' : row.pinned == -1 ? 'color: #F56C6C' : ''
}
</script>

<style scoped>
/*
 * 配色与结构对齐设置页（Setting.vue）的卡片式表格方案：
 * 筛选区与表格各自放入圆角卡片，表格背景透明跟随卡片底色，
 * 表头用 --el-fill-color-light 做一级区分，边框用 --el-border-color-extra-light，
 * 避免出现「悬空」的亮灰色块与突兀方角。
 */
.history-scrollbar {
    padding: 10px;
}

.history-pane {
    width: 100%;
}

/* 卡片容器（与设置页 .table-section-card 保持一致） */
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

.card-title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.card-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--el-text-color-primary, #dcdfe6);
}

.card-desc {
    font-size: 12px;
    color: var(--el-text-color-secondary, #909399);
}

.table-card-body {
    width: 100%;
}

/* 筛选区：等宽自适应，间距与卡片内边距统一 */
.history-filter-bar {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
}

.history-filter-bar .filter-server {
    width: 220px;
}

.history-filter-bar .filter-input {
    flex: 1;
    min-width: 180px;
}

.filter-prefix {
    color: var(--el-text-color-secondary, #909399);
}

/* 表格：圆角裁剪，与卡片融为一体 */
.custom-data-table {
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: var(--el-fill-color-light, #262727);
    --el-table-row-hover-bg-color: var(--el-fill-color-light, #262727);
    --el-table-border-color: var(--el-border-color-extra-light, #2b2b2c);
    --el-table-text-color: var(--el-text-color-regular, #cfd3dc);
    --el-table-header-text-color: var(--el-text-color-primary, #dcdfe6);
    border-radius: 8px;
    overflow: hidden;
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

:deep(.custom-data-table .el-table__body tr > td.el-table__cell) {
    background-color: transparent;
    border-bottom-color: var(--el-border-color-extra-light, #2b2b2c);
}

/* 分页：与表格右对齐，去掉悬空感 */
.history-pagination {
    margin-top: 16px;
    justify-content: flex-end;
}

/*
 * 顶部筛选区内输入框/选择框：背景透明跟随卡片底色，只保留可见边框，
 * 与设置页的输入框风格保持一致，避免亮灰块。
 */
:deep(.el-input__wrapper),
:deep(.el-select__wrapper) {
    background-color: transparent;
    box-shadow: 0 0 0 1px var(--el-border-color-extra-light, #2b2b2c) inset;
    transition: box-shadow 0.2s ease;
}

/* 下拉浮层不透明，保证选项可读 */
:deep(.el-select__popper) {
    background-color: var(--el-bg-color-overlay, #1c1d1f);
}

:deep(.el-input__wrapper:hover),
:deep(.el-select__wrapper:hover) {
    box-shadow: 0 0 0 1px var(--el-border-color, #4c4d4f) inset;
}

:deep(.el-input__wrapper.is-focus),
:deep(.el-select__wrapper.is-focused) {
    box-shadow: 0 0 0 1px var(--el-color-primary, #409eff) inset;
}
</style>
