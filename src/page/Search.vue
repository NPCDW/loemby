<template>
    <div>
        <el-input v-model="search_str" @keyup.enter="search">
            <template #append>
                <el-button @click="search"><el-icon><i-ep-Search /></el-icon></el-button>
            </template>
        </el-input>
        <div v-for="val in root_search_result">
            <div>{{ val.server.server_name }}</div>
            <v-card class="mx-auto" max-width="344" v-for="rootItem in val.result.Items">
                <!-- <v-img height="200px" src="https://cdn.vuetifyjs.com/images/cards/sunshine.jpg" cover></v-img> -->

                <v-card-title>
                    {{ rootItem.Name }}
                </v-card-title>

                <v-card-subtitle v-if="rootItem.Type == 'Series'">
                    {{ rootItem.ProductionYear + '-' + rootItem.EndDate.substring(0, 4) }} 总集数：{{ rootItem.UserData.PlayCount + rootItem.UserData.UnplayedItemCount }}
                </v-card-subtitle>
                <v-card-subtitle v-else>
                    {{ rootItem.ProductionYear }} 最大媒体流：{{ formatBytes(maxMediaSources(rootItem.MediaSources)) }}
                </v-card-subtitle>

                <v-card-actions>
                    <v-btn color="orange-lighten-2" text="Explore"></v-btn>

                    <v-spacer></v-spacer>

                    <v-btn @click="val.showSeasons = !val.showSeasons">
                        <el-icon v-if="val.showSeasons"><i-ep-ArrowUpBold /></el-icon>
                        <el-icon v-else><i-ep-ArrowDownBold /></el-icon>
                    </v-btn>
                </v-card-actions>

                <v-expand-transition>
                    <div v-show="val.showSeasons">
                        <v-divider></v-divider>

                        <v-card-text>
                            I'm a thing. But, like most politicians, he promised more than he could deliver. You won't have time
                            for sleeping, soldier, not with all the bed making you'll be doing. Then we'll go with that data
                            file! Hey, you add a one and two zeros to that or we walk! You're going to do his laundry? I've got
                            to find a way to escape.
                        </v-card-text>
                    </div>
                </v-expand-transition>
            </v-card>
            <el-pagination
                v-model:current-page="val.currentPage"
                v-model:page-size="val.pageSize"
                layout="total, prev, pager, next, jumper"
                :total="val.result.TotalRecordCount"
                @current-change="handleRootSearchPageChange(val.currentPage, val.server)"
                hide-on-single-page
            />
        </div>
    </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import { useConfig, EmbyServerConfig } from '../store/config'
import embyApi from '../api/embyApi'
import { ElMessage } from 'element-plus'

const search_str = ref('')
const root_search_result = ref<{[key: string]: {server: EmbyServerConfig, currentPage: number, pageSize: number, showSeasons: boolean, result: SearchResult}}>({})
const seasons_result = ref<{[key: string]: {}}>({})
const episodes_result = ref<{[key: string]: {[key: string]: {currentPage: number, pageSize: number, result: {}}}}>({})

interface SearchResult {
    TotalRecordCount: number,
    Items: {
        Name: string,
        Id: string,
        ProductionYear: number,
        EndDate: string,
        Type: string,
        MediaSources: {
            Size: number,
            Name: string
        }[],
        UserData: {
            UnplayedItemCount: number,
            PlayCount: number
        }
    }[]
}

async function search() {
    let config = await useConfig().get_config();
    for (let embyServer of config.emby_server!) {
        singleEmbySearch(embyServer, 1, 10);
    }
}
async function handleRootSearchPageChange(val: number, embyServer: EmbyServerConfig) {
    await singleEmbySearch(embyServer, val, 10)
}
async function singleEmbySearch(embyServer: EmbyServerConfig, currentPage: number, pageSize: number) {
    let search_result = {server: embyServer, currentPage: currentPage, pageSize: pageSize, showSeasons: false, result: {} as SearchResult}
    console.log('search', embyServer, search_str.value, (currentPage - 1) * pageSize, pageSize)
    return embyApi.search(embyServer, search_str.value, (currentPage - 1) * pageSize, pageSize).then(async response => {
        if (response.status != 200) {
            ElMessage.error({
                message: 'response status' + response.status + ' ' + response.statusText
            })
            return
        }
        let json: SearchResult = await response.json();
        search_result.result = json
        root_search_result.value[embyServer.id!] = search_result
    }).catch(e => {
        ElMessage.error({
            message: e
        })
    })
}


const maxMediaSources = (mediaSources: {Size: number}[]) => {
    let max = 0
    for (let mediaSource of mediaSources) {
        if (mediaSource.Size > max) {
            max = mediaSource.Size
        }
    }
    return max
}
const formatBytes = (size: number) => {
  const units: string[] = ['KB', 'MB', 'GB', 'TB'];
  for (let index = 0; index < units.length; index++) {
    size /= 1024;
    if (size < 1024) {
      return size.toFixed(2) + " " + units[index];
    }
  }
  return size.toFixed(2) + units[units.length - 1]
}

</script>