<script setup lang="ts">
import { watch } from 'vue';
import { useRouter } from 'vue-router';
import { useDbStatus } from './store/dbStatus';

// 数据库初始化失败的事件可能在路由守卫跑完之后才到达，这里补一次跳转
const router = useRouter()
watch(() => useDbStatus().fatalReason, (reason) => {
  if (reason && router.currentRoute.value.name !== 'fatalError') {
    router.replace({ name: 'fatalError' })
  }
})
</script>

<template>
  <router-view></router-view>
</template>

<style scoped>
</style>
