<template>
    <el-scrollbar style="padding: 10px;" ref="scrollbarRef">
        <el-form :inline="true">
            <el-form-item>
                <el-select
                    v-model="query.emby_server_id"
                    @change="getPlayHistory"
                    style="width: 260px;">
                    <el-option key="all" label="全部" value="all"/>
                    <el-option v-for="embyServer in embyServers" :key="embyServer.id" :label="embyServer.server_name" :value="embyServer.id"/>
                </el-select>
            </el-form-item>
            <el-form-item>
                <el-input v-model="query.series_name" @keyup.enter="getPlayHistory" style="width: 260px;">
                    <template #prefix>
                        <span>剧</span>
                    </template>
                </el-input>
            </el-form-item>
            <el-form-item>
                <el-input v-model="query.item_name" @keyup.enter="getPlayHistory" style="width: 260px;">
                    <template #prefix>
                        <span>集</span>
                    </template>
                </el-input>
            </el-form-item>
        </el-form>
        <el-table :data="list" :row-style="highlightRowFunction">
            <el-table-column prop="emby_server_name" label="服务器" show-overflow-tooltip />
            <el-table-column prop="series_name" label="剧" show-overflow-tooltip>
                <template #default="scope">
                    <el-link @click.prevent="gotoSeries(scope.row.emby_server_id, scope.row.series_id)" :type="scope.row.pinned ? 'primary' : 'default'">{{ scope.row.series_name }}</el-link>
                </template>
            </el-table-column>
            <el-table-column prop="item_name" label="集、电影" show-overflow-tooltip>
                <template #default="scope">
                    <el-link @click.prevent="gotoEpisodes(scope.row.emby_server_id, scope.row.item_id)" :type="scope.row.pinned ? 'primary' : 'default'">{{ scope.row.item_name }}</el-link>
                </template>
            </el-table-column>
            <el-table-column prop="played_duration" label="播放时长" :formatter="played_duration_formatter" width="100px" />
            <el-table-column fixed="right" label="Pin" width="50px">
                <template #default="scope">
                    <el-link :underline="false" @click="pin(scope.row)" style="margin-left: 5px;">
                        <el-icon color="#E6A23C" :size="16" v-if=" scope.row.pinned"><svg-icon name="pin" color="#E6A23C" /></el-icon>
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
    let pinned = row.pinned ? 0 : 1
    usePlayHistory().updatePlayHistory({id: row.id, pinned: pinned}).then(() => {
        row.pinned = pinned
    }).catch(e => ElMessage.error('更新失败' + e))
}

function highlightRowFunction({row}: {row: PlayHistory}) {
    return row.pinned ? 'color: #409EFF' : ''
}
</script>

<style scoped>
</style>