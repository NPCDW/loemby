<template>
    <el-table :data="list" style="width: 100%">
        <el-table-column prop="name" label="服务器" show-overflow-tooltip />
        <el-table-column prop="proxy_type" label="剧" />
        <el-table-column prop="addr" label="集、电影" show-overflow-tooltip />
        <el-table-column prop="username" label="播放时长" />
        <el-table-column fixed="right" label="Go">
            <template #default="scope">
                <el-button plain type="success" size="small" @click.prevent="gotoEpisodes(scope.row.emby_server_id, scope.row.item_id)">Go</el-button>
            </template>
        </el-table-column>
    </el-table>
    <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        layout="total, prev, pager, next, jumper"
        :total="total"
        @current-change="handlePageChange"
        hide-on-single-page
    />
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router';
import { onMounted, ref } from 'vue';
import { ElButton, ElMessage } from 'element-plus';
import { PlayHistory, usePlayHistory } from '../store/db/playHistory';

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
        total.value = response.count
    }).catch(e => {
        ElMessage.error('获取播放历史失败' + e)
    })
}
function handlePageChange(pageNumber: number) {
    getEpisodes(pageNumber)
}
onMounted(() => getEpisodes())
</script>

<style scoped>
</style>