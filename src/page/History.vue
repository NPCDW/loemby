<template>
    <el-scrollbar style="padding: 10px;" ref="scrollbarRef">
        <div style="display: flex; gap: 10px; margin-bottom: 10px;">
            <el-select
                v-model="query.emby_server_id"
                @change="getPlayHistory"
                clearable
                placeholder="筛选服务器">
                <el-option v-for="embyServer in embyServers" :key="embyServer.id" :label="embyServer.server_name" :value="embyServer.id"/>
            </el-select>
            <el-input v-model="query.series_name" @keyup.enter="getPlayHistory">
                <template #prefix>
                    <span>剧</span>
                </template>
            </el-input>
            <el-input v-model="query.item_name" @keyup.enter="getPlayHistory">
                <template #prefix>
                    <span>集</span>
                </template>
            </el-input>
        </div>
        <el-table :data="list" :row-style="highlightRowFunction" class="custom-data-table history-table">
            <el-table-column prop="emby_server_name" label="服务器" show-overflow-tooltip />
            <el-table-column prop="series_name" label="剧" show-overflow-tooltip>
                <template #default="scope">
                    <el-link @click.prevent="gotoSeries(scope.row.emby_server_id, scope.row.series_id)" :type="scope.row.pinned == 1 ? 'primary' : scope.row.pinned == -1 ? 'danger' : 'default'">{{ scope.row.series_name }}</el-link>
                </template>
            </el-table-column>
            <el-table-column prop="item_name" label="集、电影" show-overflow-tooltip>
                <template #default="scope">
                    <el-link @click.prevent="gotoEpisodes(scope.row.emby_server_id, scope.row.item_id)" :type="scope.row.pinned == 1 ? 'primary' : scope.row.pinned == -1 ? 'danger' : 'default'">{{ scope.row.item_name }}</el-link>
                </template>
            </el-table-column>
            <el-table-column prop="played_duration" label="播放时长" :formatter="played_duration_formatter" width="100px" />
            <el-table-column fixed="right" label="Pin" width="50px">
                <template #default="scope">
                    <el-link :underline="false" @click="pin(scope.row)" style="margin-left: 5px;">
                        <el-icon :size="16" v-if="scope.row.pinned == 1"><svg-icon name="pin" /></el-icon>
                        <el-icon :size="16" v-else-if="scope.row.pinned == -1"><svg-icon name="hidden" /></el-icon>
                        <el-icon :size="16" v-else><svg-icon name="unpin" /></el-icon>
                    </el-link>
                </template>
            </el-table-column>
        </el-table>
        <el-pagination
            style="margin: 10px 0 0 0;"
            v-model:current-page="query.page_number"
            v-model:page-size="query.page_size"
            layout="total, prev, pager, next, jumper"
            :total="total"
            @current-change="handlePageChange"
            hide-on-single-page
        />
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
        scrollbarRef.value!.setScrollTop(0)
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
 * 配色对齐 PR #10 设置页方案：
 * 表格背景透明跟随页面底色，表头用 --el-fill-color-light(#262727) 做一级区分，
 * 斑马纹用 --el-fill-color-lighter(#1d1d1d) 兜底，边框用 --el-border-color-extra-light(#2b2b2c)，
 * 避免出现纯黑/亮灰色块。
 */
.custom-data-table {
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: var(--el-fill-color-light, #262727);
    --el-table-row-hover-bg-color: var(--el-fill-color-light, #262727);
    --el-table-border-color: var(--el-border-color-extra-light, #2b2b2c);
    --el-table-text-color: var(--el-text-color-regular, #cfd3dc);
    --el-table-header-text-color: var(--el-text-color-primary, #dcdfe6);
}

:deep(.custom-data-table .el-table__header-wrapper th) {
    background-color: var(--el-fill-color-light, #262727);
    font-weight: 600;
    color: var(--el-text-color-primary, #dcdfe6);
    height: 44px;
}

:deep(.custom-data-table .el-table__body tr > td.el-table__cell) {
    background-color: transparent;
    height: 48px;
    border-bottom-color: var(--el-border-color-extra-light, #2b2b2c);
}

/* 顶部筛选区：输入框/选择框贴合底色，去掉亮灰对比 */
:deep(.el-input__wrapper),
:deep(.el-select__wrapper) {
    background-color: var(--el-fill-color-light, #262727);
    box-shadow: 0 0 0 1px var(--el-border-color-extra-light, #2b2b2c) inset;
    transition: box-shadow 0.2s ease;
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
