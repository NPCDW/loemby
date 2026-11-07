<script setup lang="ts">
import { useRuntimeConfig } from './store/runtimeConfig'

const runtime = useRuntimeConfig()
</script>

<template>
  <router-view v-if="runtime.startupStatus === 'ready'"></router-view>
  <main v-else class="startup" :aria-busy="runtime.startupStatus === 'pending'">
    <section class="startup-panel">
      <p class="startup-brand">LOEMBY</p>
      <template v-if="runtime.startupStatus === 'pending'">
        <h1 role="status">正在初始化应用…</h1>
        <p>正在连接数据库并加载配置，请稍候。</p>
      </template>
      <template v-else>
        <h1>应用初始化失败</h1>
        <p role="alert">{{ runtime.startupError }}</p>
        <p>可以重试获取启动状态；如果配置或初始化失败，请检查配置后关闭并重新打开应用。</p>
        <button type="button" @click="runtime.getRuntimeConfig()">重试</button>
      </template>
    </section>
  </main>
</template>

<style scoped>
.startup {
  min-height: 100dvh;
  display: grid;
  place-items: center;
  padding: 24px;
  box-sizing: border-box;
  background: #1d1d1d;
  color: #ececec;
}

.startup-panel {
  width: min(100%, 480px);
}

.startup-brand {
  color: #84c0ff;
  letter-spacing: 0.18em;
  font-weight: 700;
}

h1 {
  font-size: 24px;
  margin: 20px 0 12px;
}

p {
  line-height: 1.6;
  overflow-wrap: anywhere;
}

button {
  margin-top: 12px;
  min-height: 44px;
  padding: 10px 28px;
  border: 1px solid #84c0ff;
  border-radius: 8px;
  background: #84c0ff;
  color: #152535;
  font: inherit;
  cursor: pointer;
}

button:focus-visible {
  outline: 2px solid #fff;
  outline-offset: 4px;
}
</style>
