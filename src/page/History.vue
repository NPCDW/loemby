<template>
    <el-scrollbar style="padding: 10px;">
        <el-table :data="list">
            <el-table-column prop="emby_server_name" label="服务器" show-overflow-tooltip />
            <el-table-column prop="series_name" label="剧" show-overflow-tooltip />
            <el-table-column prop="item_name" label="集、电影" show-overflow-tooltip />
            <el-table-column prop="played_duration" label="播放时长" :formatter="played_duration_formatter" width="100px" />
            <el-table-column fixed="right" label="操作" width="100px">
                <template #default="scope">
                    <el-button plain type="success" size="small" @click.prevent="gotoEpisodes(scope.row.emby_server_id, scope.row.item_id)">Go</el-button>
                    <el-link :underline="false" @click="pin(scope.row)" style="margin-left: 5px;">
                        <el-icon color="#E6A23C" :size="16" v-if=" scope.row.pinned"><svg-icon name="pin" color="#E6A23C" /></el-icon>
                        <el-icon :size="16" v-else><svg-icon name="unpin" /></el-icon>
                    </el-link>
                </template>
            </el-table-column>
        </el-table>
        <el-pagination
            style="margin: 10px 0 0 0;"
            v-model:current-page="currentPage"
            v-model:page-size="pageSize"
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
import { ElButton, ElMessage } from 'element-plus';
import { PlayHistory, usePlayHistory } from '../store/db/playHistory';
import { secondsToHMS } from '../util/str_util'

const router = useRouter()
function gotoEpisodes(embyServerId: string, episodesId: string) {
    router.push('/nav/emby/' + embyServerId + '/episodes/' + episodesId)
}

const list = ref<PlayHistory[]>([])
const currentPage = ref<number>(1)
const pageSize = ref<number>(30)
const total = ref<number>(0)
async function getEpisodes(pageNumber: number = 1, pageSize: number = 30) {
    return usePlayHistory().pagePlayHistory(pageNumber, pageSize).then(async response => {
        list.value = response.list
        total.value = response.total
    }).catch(e => {
        ElMessage.error('获取播放历史失败' + e)
    })
}
function handlePageChange(pageNumber: number) {
    getEpisodes(pageNumber)
}
onMounted(() => getEpisodes())

function played_duration_formatter(row: PlayHistory) {
    return secondsToHMS(row.played_duration!)
}

function pin(row: PlayHistory) {
    let pinned = row.pinned ? 0 : 1
    usePlayHistory().updatePlayHistory({id: row.id, pinned: pinned}).then(() => {
        row.pinned = pinned
    }).catch(e => {
        ElMessage.error('更新失败' + e)
    })
}
</script>

<style scoped>
</style>