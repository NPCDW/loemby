# 前端架构与设计系统

本文说明 `src/` 的分层、页面的写法约定，以及如何在纯浏览器里预览与核版。

## 分层

```
src/
├── api/            与 Rust 侧的边界。所有 JSON 字符串在这里解析成类型对象
├── components/
│   ├── base/       与业务无关的原子件：空态、面板、段落、骨架
│   ├── item/       条目级：海报卡、继续观看卡、单集行
│   ├── media/      场景级：横向天桥、详情主视觉、章节条、外链
│   ├── server/     服务器上下文：导航、顶栏、线路、代理、弹窗、消息中心
│   └── settings/   设置页专用：设置项、设置分组、轻量列表
├── composables/    跨页复用的状态逻辑（见下）
├── layout/         AppShell：外壳骨架 + 服务器上下文装配
├── pages/          页面，只负责取数、排版与派发
├── router/         路由表
├── store/          Pinia：数据库缓存、事件总线、图片地址中心、消息中心
├── styles/         （预留）
├── util/           纯函数：格式化、分辨率识别、URL 判定
└── style.css       设计系统唯一入口（设计令牌 + 重置 + 原子 + EP 覆盖）
```

### 依赖方向

`pages` → `components` / `composables` → `api` / `store` / `util`。
反向不允许：组件里不出现页面级取数，`api` 不依赖任何 Vue 代码。

## 关键约定

### 1. 在边界处解析

Rust 侧把 Emby 响应作为 JSON 字符串返回。`src/api/embyApi.ts` 在唯一的地方
做 `JSON.parse` 并给出类型，页面只面对对象：

```ts
const result = await embyApi.episodes(serverId, seriesId, seasonId, 0, 30);
result.Items.forEach(item => /* 已经是 EpisodeItem */);
```

需要收窄联合类型时用 `embyApi.asSeries` / `asLibraryItem`，不要到处写 `as any`。

### 2. 复用的状态逻辑放 composables

| composable | 解决的问题 |
| --- | --- |
| `useEmbyServers` | 服务器列表的单一来源 + 排序 + 订阅变更 |
| `useItemActions` | 收藏 / 标记已播放，统一在途标记与错误提示 |
| `usePlaybackTracks` | 版本、视频/音频/字幕轨道的编排与自动选择 |
| `useGlobalSetting` | 全局配置项的双向绑定 + 默认值 + 失败回滚 |

**`useGlobalSetting` 的用法要点**：它返回的就是一个 ref（额外挂了 `save`/`load`）。

- 模板里 `v-model="xxx"` 直接用，**不要**写 `xxx.value`
- 脚本里用 `xxx.value` 读写，**不要**写 `xxx.value.value`
- 需要从模板触发保存时，用 `@change="saveSetting('配置名')"`。
  模板中的 ref 会被自动解包，拿不到挂在 ref 上的 `save`，所以走 key 查表。

### 3. 页面的固定形状

```vue
<template>
  <div class="page">
    <div class="toolrow">…筛选/搜索…</div>
    <AppSection title="…" :count="n">…</AppSection>
  </div>
</template>
```

页面标题由 `ServerTopBar` 统一渲染，页面里不再重复大标题。
内容区高度由外壳的滚动容器管理，页面里**不要**写 `calc(100vh - Npx)`。

## 设计系统

主题取自产品本身：loemby 是在暗房里放片子的客户端。

- 底色冷调墨蓝近黑 `--surface`，唯一强调色是放映机暖光琥珀 `--lamp`
- 琥珀只出现在「正在发生的事」上：进度、激活项、主按钮
- 层级用一条发丝线 `--line` 表达，**不用投影、不叠白光**
- 圆角只给控件与封面（`--r-md`），不给整张卡片套壳
- 数字一律等宽（`.mono` / `.tag`），列表滚动时列不会跳动

`style.css` 的分节顺序：设计令牌 → 重置 → 排版原子 → 布局对象 →
通用组件对象 → Element Plus 覆盖层 → 动效与可访问性。
改样式前先确认要改的是哪一节，避免又退回逐页写内联样式。

可访问性下限：统一可见焦点环、尊重 `prefers-reduced-motion`、正文对比度达标。

## 纯浏览器预览与核版

页面依赖 Tauri IPC，普通浏览器里会白屏。用 mock 模式可以起一个假后端核排版：

```bash
pnpm build:mock      # 产出 dist/，其中 tauri-stub 先于 main.ts 执行
pnpm preview         # 打开 http://127.0.0.1:4173
```

工作原理见 `vite.mock-tauri.ts` 与 `mock/tauri-stub.ts`：在 `--mode mock` 下
把 `@tauri-apps/api` alias 到桩模块，桩里给每个 IPC 命令一份有真实感的假数据，
并把本地 axum 的 `/image` 请求换成本地生成的 SVG 占位图。

生产构建（默认 mode）不会引入任何 mock 代码。

### 核版检查清单

- [ ] 每页无控制台报错、无意外弹出的对话框
- [ ] 加载中显示与真实内容同形状的骨架，加载完成不出现布局重排
- [ ] 空态与失败态都给出「下一步做什么」
- [ ] 键盘 Tab 一圈，焦点环可见且顺序合理
- [ ] 系统开启减少动效后，没有残留的位移与缩放
