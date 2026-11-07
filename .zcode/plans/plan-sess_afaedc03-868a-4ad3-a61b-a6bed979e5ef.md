## Context
项目 Android 的 `call_player` 目前是未适配桩，移动详情页播放按钮也被禁用。桌面已有 Rust JSON IPC、媒体代理和播放同步。实施第一阶段 Android 前台播放：**仓内 Tauri Android 插件 + 原生 PlayerActivity/Surface + libmpv；Rust 继续使用 Unix socket JSON IPC**，保持桌面行为。

首版支持 **arm64-v8a**、播放/续播、暂停、seek、上下集、退出和现有播放进度同步。暂不做后台播放、画中画、下载及完整轨道选择界面。切后台或失去 Surface 时安全结束会话；音频焦点丢失时暂停。无需新增 Vue 播放路由。

## 1. 原生插件与 libmpv
新增 `src-tauri/plugins/tauri-plugin-mpv/` 本地 crate：
- Rust 注册 Android plugin，提供 prepare/close 等类型化调用；通过 Cargo path dependency 和 `.android_path("android")` 接入，不手改生成的 Tauri Gradle glue。
- Android library 内实现 `MpvPlugin.kt`、`MpvPlayerActivity.kt`、`MpvNative.kt`、Manifest、consumer ProGuard rules，以及 `cpp/CMakeLists.txt`、薄 JNI 实现。
- 原生层注册 JavaVM，创建 libmpv，设置 Android 专属选项和私有目录 socket；使用 JNI global ref 绑定 Surface。保留 owner handle 的单一事件消费者。
- prepare 仅准备播放器，不加载媒体；Surface 可用、libmpv 初始化成功后返回。Rust 再确认 socket 实际可连接。
- 原生界面提供基础播放控制。返回、后台、加载失败和 native shutdown 均进入按 sessionId 管理的幂等清理；先停止并销毁播放器，再释放 Surface 引用，避免照搬官方 detach 竞态。阻塞的初始化/销毁不放在 UI 线程。

## 2. 共享播放计划与 Rust IPC
修改 `src-tauri/src/service/player_svc.rs`、`service/axum_svc.rs`，按需新增 Android 会话/IPC 模块：
- 从现有 `call_player` 提取服务器校验、剧集列表、M3U、起始项、UUID 和 `MediaPlaylistParam` 的共享准备逻辑。桌面保留现有路径、配置与 spawn 启动。
- Android 替换 mobile 错误桩；其他未支持的移动平台继续明确报错。
- Android socket、配置和列表使用应用私有目录及短文件名，不使用 `/tmp` 或桌面 portable_config。
- Android 每个播放器实例保留一个 IPC 读循环和事件分发器，复用当前 Unix socket、mpv 命令/事件及播放业务。读取器必须在 loadlist 前就绪；媒体解析与当前条目会话注册在返回播放地址前完成，避免漏掉快速媒体的 `file-loaded`。
- 用 Serde 构造新增 JSON 请求，关联 request_id，确认 loadlist/控制命令响应并设置超时；关联当前播放条目，避免预取或旧会话事件误处理。
- 复用现有 `play_info_init`、历史与 Emby/第三方同步规则，剥离 Android 不需要的 uosc 消息与桌面窗口操作。中间条目结束不销毁播放器；最终退出清理 IPC、轮询、计划和会话缓存。原生释放不依赖网络上报成功。
- 保留 Axum 的 loopback 媒体/字幕代理，不让 mpv 重复实现项目鉴权和代理逻辑。

主要应用接入文件：`src-tauri/Cargo.toml`、`src-tauri/src/lib.rs`、`src-tauri/src/service/mod.rs`。

## 3. 移动详情入口
修改 `src/android/pages/emby/Episodes.vue`、`src/api/invokeApi.ts`，复用统一 `call_player` 参数：
- 启用播放和续播，正确转换页面零起始版本索引与后端版本选择参数。
- 增加准备中状态、防重复调用、错误展示；下载保持禁用并更新提示。
- 监听与当前服务器/媒体相关的播放结束通知，返回后刷新观看进度，处理卸载和路由变化，避免旧请求更新新页面。
- 将 `call_player` 返回类型修正为 `Promise<void>`。

## 4. Native 依赖与 Android 构建
新增 native 准备/校验脚本及 `docs/android-mpv.md`：
- 基于固定提交的官方 mpv-android 构建配方准备 arm64 libmpv、FFmpeg、匹配头文件及完整动态依赖闭包；记录实际版本、构建参数、校验和和许可证。不从 APK 提取库、不使用 floating latest、不编造二进制下载地址。
- native 源码与构建目录独立存放；先检查下载内容及构建配方，再执行必要构建。应用 Gradle/CMake 只消费已准备产物，不在普通 app 构建时自动下载。
- 缺失 ABI、头文件或依赖时构建明确失败并给出准备命令；运行时 native 加载失败返回可读错误，不拖垮浏览功能。
- 按实际 ELF DT_NEEDED 校验依赖、C++ runtime 和 16KB ELF 对齐，避免重复打包或用 pickFirst 掩盖冲突。
- 调整 `src-tauri/gen/android/app/build.gradle.kts` 及必要的根 Gradle 配置，使插件与当前 Tauri 所需 SDK/AGP/NDK 兼容，限定首版 ABI。补充可手动运行的 Android 构建验证 CI，仅产出构建 artifacts，不发布 release。
- 更新忽略规则，native 二进制和构建缓存不提交。文档明确实际 mpv/FFmpeg 的 GPL 等分发义务。

## 5. 验证
- 前端：运行现有 Android 浏览测试，补充播放/续播参数、版本索引、防重复、失败恢复及进度刷新测试；运行类型检查和前端构建。
- Rust：共享计划及 IPC 编解码、请求超时、快速 file-loaded、早期 end-file、切集与旧 session 隔离测试；Android target 检查及可用的桌面回归检查。
- Android：JNI/Kotlin 编译、debug/release arm64 APK，检查混淆保留规则、所有 native 依赖、ELF 与 APK 对齐。
- 有设备时验证：Rust IPC 查询/暂停/seek → loopback 媒体播放 → 续播/字幕/自动下一集 → 返回/后台/Surface失效 → 进度保存；release 和16KB设备分别验证，不以编译成功代替运行验证。

**当前限制：**环境尚无 mpv native 产物、完整前端/Gradle依赖，也未确认真机或模拟器。会安装必要构建依赖并尝试实际构建；若工具、网络或设备阻塞，明确报告完成部分、失败输出和未验证项，不宣称已真机跑通。注意现有 `src-tauri/build.rs` 会改写版本文件，构建后检查并避免留下无关改动。此次不提交、不推送、不对外发布。