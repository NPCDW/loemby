<template>
    <div style="height: 100vh; display: flex; flex-direction: row;">
        <div style="flex: none; width: 200px;">
            <el-menu class="el-menu" :collapse="true" :router="true" :default-active="active">
                <el-menu-item index="/nav/search">
                    <el-icon><i-ep-Search /></el-icon>聚合搜索
                </el-menu-item>
                <el-menu-item>
                    <el-icon class="is-loading" style="color: cyan;"><i-ep-Loading /></el-icon>服务器1
                </el-menu-item>
                <el-menu-item>
                    <el-icon style="color: green;"><i-ep-SuccessFilled /></el-icon>服务器2
                </el-menu-item>
                <el-menu-item>
                    <el-icon style="color: red;"><i-ep-WarningFilled /></el-icon>服务器3
                </el-menu-item>
                <el-menu-item>
                    <el-icon><i-ep-Plus /></el-icon>添加服务器
                </el-menu-item>
            </el-menu>
        </div>
        <div style="flex: auto;">
            <router-view></router-view>
        </div>
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
    width: 100%;
    height: 100vh;
    background-color: var(--dark-background-color);
}
</style>