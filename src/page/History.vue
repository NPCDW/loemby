<template>
    <el-scrollbar ref="scrollbarRef">
    <div class="roe-page">
        <!-- 工具行：筛选与检索永远在同一行，位置固定 -->
        <div class="roe-toolrow">
            <el-select
                v-model="query.emby_server_id"
                @change="getPlayHistory"
                clearable
                placeholder="全部服务器"
                class="filters__server">
                <el-option v-for="embyServer in embyServers" :key="embyServer.id" :label="embyServer.server_name" :value="embyServer.id"/>
            </el-select>
            <el-input v-model="query.series_name" @keyup.enter="getPlayHistory" placeholder="按剧名筛选" class="filters__text">
                <template #prefix><span class="filters__key">剧</span></template>
            </el-input>
            <el-input v-model="query.item_name" @keyup.enter="getPlayHistory" placeholder="按集名筛选" class="filters__text">
                <template #prefix><span class="filters__key">集</span></template>
            </el-input>
            <el-button type="primary" plain @click="getPlayHistory">筛选</el-button>
        </div>

        <el-table class="history-table" :data="list" :row-class-name="rowClassName" :row-style="highlightRowFunction">
            <el-table-column prop="emby_server_name" label="服务器" width="150" show-overflow-tooltip />
            <el-table-column prop="series_name" label="剧" show-overflow-tooltip>
                <template #default="scope">
                    <el-link v-if="scope.row.series_id" @click.prevent="gotoSeries(scope.row.emby_server_id, scope.row.series_id)" :type="scope.row.pinned ? 'primary' : 'default'">{{ scope.row.series_name }}</el-link>
                    <span v-else>{{ scope.row.series_name || '—' }}</span>
                </template>
            </el-table-column>
            <el-table-column prop="item_name" label="集 / 电影" show-overflow-tooltip>
                <template #default="scope">
                    <el-link @click.prevent="gotoEpisodes(scope.row.emby_server_id, scope.row.item_id)" :type="scope.row.pinned ? 'primary' : 'default'">{{ scope.row.item_name }}</el-link>
                </template>
            </el-table-column>
            <el-table-column prop="played_duration" label="播放时长" :formatter="played_duration_formatter" width="110">
                <template #default="scope">
                    <span class="mono">{{ played_duration_formatter(scope.row) }}</span>
                </template>
            </el-table-column>
            <el-table-column fixed="right" label="Pin" width="64" align="center">
                <template #default="scope">
                    <el-link :underline="false" @click="pin(scope.row)" :title="scope.row.pinned ? '取消置顶' : '置顶'">
                        <el-icon :size="15" :class="{ 'pin-on': scope.row.pinned }">
                            <svg-icon v-if="scope.row.pinned" name="pin" size="15" color="#F2A13B" />
                            <svg-icon v-else name="unpin" size="15" color="currentColor" />
                        </el-icon>
                    </el-link>
                </template>
            </el-table-column>
        </el-table>

        <div v-if="!list.length" class="roe-empty">
            <span class="roe-empty__line">还没有播放记录</span>
            <span>从左侧打开一台服务器，开始播放后这里会出现记录。</span>
        </div>

        <el-pagination
            v-model:current-page="query.page_number"
            v-model:page-size="query.page_size"
            layout="total, prev, pager, next, jumper"
            :total="total"
            @current-change="handlePageChange"
            hide-on-single-page
        />
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
    return row.pinned ? 'color: #F2A13B' : ''
}

/* 仅样式钩子：Pin 行用左侧灯条表示，不改动任何数据逻辑 */
function rowClassName({row}: {row: PlayHistory}) {
    return row.pinned ? 'roe-row-pinned' : ''
}
</script>

<style scoped>
</style>
