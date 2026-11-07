## Context
本次仅适配**前端播放历史页面**，为后续 Android 应用做准备；暂不生成 APK、不改 Rust 后端、不适配其他页面。Android 页面与相关专用代码统一放在 `src/android/`，原桌面页面保持不变。

现有 `src/page/History.vue` 使用宽表格，且 `/nav/history` 位于 `NavMenu` 的 200px 桌面侧栏内，因此 Android 播放历史需要独立页面和不经过桌面侧栏的路由。

## 实施方案
1. **新增 `src/android/pages/History.vue`**
   - 独立全宽布局：标题和记录数、可收起的筛选区、纵向历史卡片、紧凑分页。
   - 保留服务器、剧名、集名筛选，展示服务器、剧/电影/集名称、累计播放时长。
   - 保留正常 → 置顶 → 隐藏 → 正常的标记操作，提供明确状态文字及至少 44px 的触控区域；不把“隐藏”误实现为删除。
   - 添加加载、空记录、查询失败重试和逐条更新中的反馈；筛选时回到第一页，防止旧请求覆盖新结果和重复点击更新。
   - 处理长标题、电影没有剧集信息、缺失 ID 等情况；适配窄屏、横屏、深色主题和安全区，避免横向滚动。
   - 名称点击继续使用现有详情路由，详情页保持桌面实现；不增加虚假的“继续播放”功能。

2. **复用现有数据接口，保持桌面实现不动**
   - 使用 `src/store/db/playHistory.ts` 的 `pagePlayHistory`、`updatePlayHistory` 和现有类型。
   - 使用 `src/store/db/embyServer.ts` 的服务器列表以及 `src/util/str_util.ts` 的 `secondsToHMS`。
   - Android 专用辅助逻辑、样式和必要组件都放在 `src/android/`，不修改桌面 History、NavMenu 或全局样式，不为此重构桌面逻辑。

3. **最小化接入现有路由**
   - 新增 `src/android/platform.ts`，优先用 Tauri 构建平台变量识别 Android，不依赖屏幕宽度或已被自定义的 User-Agent。
   - 小幅调整 `src/router/router.ts`：Android 的 `/nav/history` 使用独立页面并绕开 `NavMenu`；桌面仍使用原来的历史子路由。
   - `/` 的历史入口和所有非历史路由保留原有行为。

4. **提供隔离的前端预览**
   - 现有 `src/main.ts` 在挂载前必须调用原生 Tauri 接口，普通浏览器无法直接预览。
   - 在 `src/android/preview/` 增加仅供开发使用的预览入口和明确标注的模拟数据，通过现有 Vite 服务访问；真实应用仍调用现有 Tauri 数据接口。
   - 模拟数据和失败场景仅存在于显式预览入口，不修改共享 store、不污染桌面启动流程、不在真实应用中自动用假数据兜底。
   - 在 `package.json` 增加 Android 前端预览命令，便于单独查看这一页。

## 主要文件
- 新增：`src/android/pages/History.vue`、`src/android/platform.ts`、`src/android/preview/` 下的预览入口与测试数据，以及必要的 Android 专用辅助文件。
- 修改：`src/router/router.ts`、`package.json`。
- 不改：`src/page/History.vue`、`src/components/NavMenu.vue`、其他页面、`src/main.ts`、全局 CSS、Tauri/Rust/Android 打包配置。

## 验证
- 安装锁文件对应的前端依赖，运行 `vue-tsc` 和前端生产构建，并验证 Android 平台选择下的构建。
- 使用浏览器测试 360px/390px 窄屏和横屏：筛选、分页、三种标记状态、长标题、电影、加载、空状态、接口失败及重试，检查触控尺寸和横向溢出。
- 检查真实应用数据入口不使用模拟数据；检查 Android 历史路由不包含桌面侧栏，桌面历史路由仍指向原页面，其他路由不变。
- 本阶段没有原生 Android 运行与实际数据库环境时，明确区分浏览器模拟验证与尚未进行的真机验证；不声称已完成 APK 或原生播放适配。

## 续作进度（2026-11-05）
- 已完成独立 Android 历史页面、Android 平台检测、最小路由接入和隔离模拟预览。
- 新增 Android 专用 composable 处理请求竞态、筛选回首页、分页回退、逐条标记更新及错误重试；真实数据接口保持不变。
- 已完成锁文件依赖安装、`vue-tsc`、桌面与 Android 平台前端构建，以及状态/路由/SSR 内容自动化回归验证。
- 开发预览命令：`pnpm dev:android`；用法及验证限制见 `src/android/README.md`。
- **未完成的验收项**：当前会话未提供浏览器控制工具，360px/390px、横屏、深浅主题、44px 实际触控尺寸、横向溢出及安全区仍需浏览器验证。自动化状态/SSR 检查不代替视觉或 GUI 验证。
- 未进行真实数据库或 Android 真机验证，未生成 APK，未修改桌面页面、全局 CSS、原始启动流程或 Rust 后端。
