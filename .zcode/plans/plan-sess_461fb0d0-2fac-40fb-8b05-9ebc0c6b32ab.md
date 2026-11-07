## Context
实机已确认：使用 PostgreSQL 配置时，页面提前调用 `get_runtime_config`，而 Rust 尚未注册 `AppState`，触发 `state not managed`。前端在挂载前等待该请求且没有错误处理，因此黑屏。数据库连接、迁移和配置读取本身成功。

## 修复方案
1. **建立可提前查询的启动状态**
   - 在 `src-tauri/src/config/app_state.rs` 增加独立的 `Pending / Ready / Failed` 启动状态，并在 Builder 阶段注册，早于窗口和 setup。
   - 使用 Tokio `watch` 保存状态，避免就绪通知丢失；等待有固定超时，不无限挂起。
2. **让后端初始化与配置请求有明确顺序**
   - 修改 `src-tauri/src/lib.rs`：数据库、Axum、四组配置缓存及缓存目录初始化全部成功后才标记 Ready；后台清理计划不阻塞就绪。
   - 可恢复的初始化错误记录为 Failed，保留启动错误界面，不直接 panic；错误提示不包含数据库 URL、密码或密钥。
   - 修改 `src-tauri/src/controller/invoke_ctl.rs` 和 `src-tauri/src/config/runtime_config.rs`：入口改用 `AppHandle`，等待启动就绪后通过 `try_state` 读取真实 `AppState`，不再提前注入尚未存在的 State。
3. **先显示启动界面，再开放业务页面**
   - 修改 `src/main.ts`、`src/App.vue`、`src/store/runtimeConfig.ts`：先挂载启动壳，显示“正在初始化”；成功后显示路由并初始化缓存和通知监听。
   - 请求失败或超时显示可见错误与重试入口；后端初始化失败时提示检查配置并重启。限制重复请求，忽略过期响应，业务初始化与监听仅注册一次。
   - 保留既有 `get_runtime_config` 成功返回结构、服务器配置和 mpv 行为，不引入固定延时碰运气。

## 验证
- 新增 Rust 就绪状态测试：提前请求、先完成后订阅、多等待者、失败、超时及通知不丢失。
- 复用现有前端测试方式，覆盖加载、失败、重试、迟到响应和初始化不重复；运行现有 27 项前端回归、Rust 测试、`pnpm build` 与 diff 检查。
- 构建普通包名 `com.loemby.app` 的 debug APK，核验原生依赖/对齐与签名后覆盖安装；不卸载、不清数据，保留手机上的 PostgreSQL 配置。
- 在同一实机用 PostgreSQL 配置重复冷启动，验证不再黑屏、服务器和历史页能加载；必要时短暂对照 SQLite 并立即恢复原配置。

本次不扩展到 mpv 播放逻辑，也不改动你已有的其他未提交工作。