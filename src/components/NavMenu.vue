<template>
    <div class="flex flex-row" style="height: 100vh">
        <div class="flex-none">
            <el-menu class="el-menu" :collapse="true" :router="true" :default-active="active">
                <el-menu-item index="/nav/search">
                    <el-icon><i-ep-Search /></el-icon>聚合搜索
                </el-menu-item>
                <el-menu-item index="/nav/emby">
                    <el-icon></el-icon>服务器列表
                </el-menu-item>
            </el-menu>
        </div>
        <router-view class="flex-auto"></router-view>
    </div>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useRoute } from 'vue-router'

const active = ref("");
const route = useRoute();
watchEffect(() => {
    active.value = route.path;
    console.log(active.value)
})
</script>

<style scoped>
.el-menu {
    width: var(--nav-menu-width);
    height: calc(100vh - var(--title-bar-height));
    background-color: var(--dark-background-color);
    border-right: none;
}
</style>