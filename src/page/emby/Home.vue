<template>
    <div>
        <el-input v-model="search_str" autofocus @keyup.enter="search" :disabled="search_loading" style="padding: 10px;">
            <template #append>
                <el-button type="primary" @click="search" :loading="search_loading"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
    </div>

    <el-scrollbar style="height: calc(100vh - 52px); padding: 0 20px;">
        <div>
            <h1>继续观看</h1>
            <el-skeleton :loading="episodesLoading" animated>
                <template #template>
                    <div class="note-item" v-for="i in 5" :key="i">
                        <el-skeleton-item variant="h3" style="width: 50%; margin-top: 10px;" />
                        <p><el-skeleton-item variant="text" style="width: 30%" /></p>
                    </div>
                </template>
                <div style="display: flex; flex-wrap: wrap; flex-direction: row;">
                    <el-card style="width: 300px; margin: 5px;" v-for="episodesItem in episodesList">
                        <el-link :underline="false" @click="gotoSeries(episodesItem.SeriesId)"><h2>{{ episodesItem.SeriesName }}</h2></el-link>
                        <p>{{ 'S' + episodesItem.ParentIndexNumber + 'E' + episodesItem.IndexNumber + '. ' + episodesItem.Name }}</p>
                        <p><el-progress :percentage="episodesItem.UserData?.Played ? 100 : episodesItem.UserData?.PlayedPercentage" :format="(percentage: number) => Math.trunc(percentage) + '%'" /></p>
                        <p>{{ episodesItem.PremiereDate ? episodesItem.PremiereDate.substring(0, 10) : '' }} <el-tag disable-transitions>{{ episodesItem.MediaSources ? formatBytes(maxMediaSources(episodesItem.MediaSources)?.Size!) : 0 }}</el-tag></p>
                        <p><el-button type="primary" @click="gotoEpisodes(episodesItem.Id)">详情</el-button></p>
                    </el-card>
                </div>
            </el-skeleton>
            <el-pagination
                v-model:current-page="episodesCurrentPage"
                v-model:page-size="episodesPageSize"
                layout="total, prev, pager, next, jumper"
                :total="episodesTotal"
                @current-change="handleEpisodesPageChange"
                hide-on-single-page
            />
        </div>
    </el-scrollbar>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import embyApi, { EmbyPageList, EpisodesItems } from '../../api/embyApi';
import { useConfig } from '../../store/config';
import { ElMessage } from 'element-plus';
import { formatBytes } from '../../util/str_util'
import { maxMediaSources } from '../../util/play_info_util'

const router = useRouter()
const route = useRoute()

let embyServer = useConfig().getEmbyServer(<string>route.params.id)!

watch(() => route.params.id, (newId, _oldId) => {
    embyServer = useConfig().getEmbyServer(<string>newId)!

    episodesList.value = []
    episodesCurrentPage.value = 1
    episodesPageSize.value = 6
    episodesTotal.value = 0
    continuePlay(episodesCurrentPage.value, episodesPageSize.value)
})

const search_str = ref('')
const search_loading = ref(false)
const search = async () => {
    search_loading.value = true
    search_loading.value = false
}

const episodesLoading = ref(false)
const episodesList = ref<EpisodesItems[]>([])
const episodesCurrentPage = ref(1)
const episodesPageSize = ref(6)
const episodesTotal = ref(0)
const handleEpisodesPageChange = (val: number) => {
    episodesCurrentPage.value = val
    continuePlay(val, episodesPageSize.value)
}

function continuePlay(currentPage: number, pageSize: number) {
    episodesLoading.value = true
    episodesCurrentPage.value = currentPage
    episodesPageSize.value = pageSize
    return embyApi.continuePlay(embyServer, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status_code != 200) {
            ElMessage.error({
                message: 'response status' + response.status_code + ' ' + response.status_text
            })
            return
        }
        let json: EmbyPageList<EpisodesItems> = JSON.parse(response.body);
        episodesList.value = json.Items
        episodesTotal.value = json.TotalRecordCount
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    }).finally(() => episodesLoading.value = false)
}
continuePlay(episodesCurrentPage.value, episodesPageSize.value)

function gotoEpisodes(episodesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/episodes/' + episodesId)
}
function gotoSeries(seriesId: string) {
    router.push('/nav/emby/' + embyServer.id + '/series/' + seriesId)
}
</script>

<style scoped>
.note-container {
  display: flex;
  height: 500px;
}

.note-sidebar {
  width: 30%;
  border-right: 1px solid #18222C;
  padding-right: 20px;
  overflow-y: auto;
}

.note-item {
  padding: 3px 10px;
  cursor: pointer;
  border-bottom: 1px solid #18222C;
}

.note-item:hover {
  background-color: #18222C;
}

.note-item.active {
  color: #409EFF;
}

.note-content {
  width: 70%;
  padding-left: 20px;
}

h2 {
  margin-top: 0;
}

.el-scrollbar {
  height: 100%;
}
</style>