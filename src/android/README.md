# Android 前端适配与安装包

播放历史及浏览主链路已提供 Android 独立页面：服务器首页、系列详情、单集/电影详情、媒体库与库内列表，配有移动导航和已有服务器选择。搜索、完整设置、服务器管理与线路配置仍为桌面实现，未作为移动端已支持入口。播放/续播已接入仓内 libmpv 原生播放器，架构与构建说明见 `docs/android-mpv.md`。本轮优先对齐详情/选集页面和播放参数，**Android 不提供下载功能**；页面完成不代表原生播放器所有能力已通过真机验证。

## Android 原生配置位置

| 文件 | 用途 |
| --- | --- |
| `src-tauri/tauri.conf.json` | 共用的 Tauri 配置、应用标识和前端构建入口 |
| `src-tauri/tauri.android.conf.json` | Android 覆盖配置：不打包桌面 mpv 资源，不生成桌面更新包 |
| `src-tauri/gen/android/app/build.gradle.kts` | Android 应用的 SDK、包名、构建类型与依赖 |
| `src-tauri/gen/android/build.gradle.kts` | Android Gradle 插件与 Kotlin 插件版本 |
| `src-tauri/gen/android/settings.gradle` | Gradle 模块设置；本工程生成的是 Groovy `.gradle`，不是 `.gradle.kts` |
| `src-tauri/gen/android/gradle.properties` | JVM、AndroidX 等 Gradle 参数 |
| `src-tauri/gen/android/app/src/main/AndroidManifest.xml` | 权限、Activity、应用与组件声明 |
| `src-tauri/gen/android/app/src/main/java/com/loemby/app/MainActivity.kt` | Android Activity 入口 |

`src-tauri/gen/android/` 由 `pnpm exec tauri android init --ci --skip-targets-install` 生成。已初始化后不要为了正常构建反复重新初始化。`local.properties` 是可选的本机 SDK 路径配置，应保持忽略；本次使用环境变量提供 SDK/NDK 路径。

Android 应用图标以 `src-tauri/icons/android/mipmap-*` 为来源。`app/build.gradle.kts` 的 `syncAndroidIcons` 在 `preBuild` 前同步标准、圆形及前景 PNG 到 `app/src/main/res/`，Manifest 使用 `@mipmap/ic_launcher` 和 `@mipmap/ic_launcher_round`。更新源目录后，下一次构建会自动同步；图标不依赖桌面的 `.ico` 或 `.icns`。

### arm64-v8a APK 构建

需要 JDK、Android SDK（本工程 compile/target SDK 36）、Build Tools 36、NDK 27.2、Rust `aarch64-linux-android` 目标，以及 CMake/Make 和已核验的 native bundle（见 `docs/android-mpv.md`）。用本机实际目录设置以下变量：

```sh
export JAVA_HOME=/path/to/jdk
export ANDROID_HOME=/path/to/android-sdk
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export NDK_HOME="$ANDROID_HOME/ndk/<installed-version>"
export ANDROID_NDK_ROOT="$NDK_HOME"
export ANDROID_NDK="$NDK_HOME"

pnpm install --frozen-lockfile
rustup target add aarch64-linux-android
CARGO_BUILD_JOBS=4 CARGO_PROFILE_DEV_DEBUG=0 \
  pnpm exec tauri android build --target aarch64 --debug --apk --ci
```

`CARGO_PROFILE_DEV_DEBUG=0` 仅省略 Rust 调试符号以控制安装包大小，APK 仍是 Debug 构建并使用开发签名。不要用它替代正式发布签名；更换开发密钥后无法直接覆盖安装旧的同包名应用。原始 APK 位于 `src-tauri/gen/android/app/build/outputs/apk/` 的对应变体目录。

首次启动使用编译内置的默认配置，在应用私有配置目录写入 `config/app-config.json` 并通过现有逻辑初始化 `db/loemby.db`。不会自动导入桌面播放历史，也不使用模拟数据。

Android 额外接入 `rustls-platform-verifier` 的 JNI 初始化和其配套 AAR，继续使用系统证书校验。Gradle 通过 Cargo 元数据定位 AAR 的本地 Maven 仓库，不硬编码开发机 Cargo 缓存路径。首屏挂载前完成本地 HTTP 服务端口初始化，避免读取未就绪状态。`build.rs` 为 Android 原生库设置 16 KB 加载段对齐。

若当前 Tauri CLI 没有生成 `app/tauri.properties`，Gradle 会从 `src-tauri/tauri.conf.json` 读取版本；版本码按 `major × 1,000,000 + minor × 1,000 + patch` 计算。

## 开发预览

```sh
pnpm install --frozen-lockfile
pnpm dev:android
```

访问 `http://localhost:3420/src/android/preview/index.html`。

顶部明确标注模拟数据，使用与生产相同的移动路由和页面。展开开发预览工具可切换空记录、查询失败、服务器列表失败、标记失败、慢请求和深浅主题；切换后触发查询/重试/标记，或重新挂载当前页面。可从历史进入系列/单集详情，或选择模拟服务器浏览首页、媒体库、分页与嵌套合集。目录覆盖特别季、电影、缺少媒体源/UserData、长标题；海报为本地生成的模拟 SVG，不代表真实媒体封面。

预览不导入 `src/main.ts` 或全局样式，也不使用真实数据库。真实页面默认调用现有 Pinia/Tauri 接口，不在接口失败时回退到模拟数据。生产构建只包含根 `index.html`，不包含开发预览入口及模拟数据。

## 平台与路由

`platform.ts` 优先读取 `TAURI_ENV_PLATFORM`，未设置时读取 `TAURI_PLATFORM`，仅当值为 `android` 才启用 `routes.ts` 中的移动浏览路由。不以屏幕宽度、自定义 User-Agent 或预览参数判断平台。

Android 历史及五个浏览页统一使用 `Layout.vue`，绕开桌面 `NavMenu`。桌面路由保持原样；未适配搜索与设置路由仍走原桌面布局。移动导航仅提供已支持入口，无服务器时明确提示尚未适配服务器管理。真实入口已设置 `viewport-fit=cover`，安全区仍需真机检查。

历史正常 → 置顶 → 隐藏 → 正常仍保存到现有接口，但成功后只更新对应记录显示，不重新读取或排序，不修改当前页、总数和列表顺序。隐藏仅是标记，不删除记录；主动查询、翻页或重进页面时仍接受服务端最新排序。在途旧查询返回时会合并其发起之后成功的本地标记。

## 自动化验证

```sh
node --test src/android/tests/*.test.mjs
pnpm exec vue-tsc --noEmit
pnpm build
TAURI_ENV_PLATFORM=android pnpm build --outDir /tmp/loemby-android-build
```

测试通过 Vue 无 DOM 宿主和 SSR 检查状态与内容，并编译实际路由表对照 `HEAD` 中桌面原有路由。历史覆盖旧请求竞态、加载、筛选与重置、错误重试、分页边界、标记循环、防重复、失败及零受影响行，并断言标记不增加读取次数/不替换列表。浏览测试使用真实页面的客户端模板编译与无 DOM 宿主，覆盖加载后文本、季切换、分页、电影与缺源/缺 UserData、路由参数复用、合集跳转、局部写入及资源竞态。

这些测试**不是浏览器视觉或 GUI 验证**。仍需在 360px/390px 及横屏中实际检查筛选、分页、标记、失败重试、44px 触控区域、横向溢出、深浅主题、安全区和长标题。

## 详情/选集页面对齐（2026-11-06）

- 电影/单集详情：Logo、外部 IDs/资料链接、版本大小/码率/分辨率、自动/手动策略、视频/音频/字幕选择与关闭、直链开关、从头/续播及章节起播入口。下拉操作自动进入手动模式；自动预选沿用桌面现有版本排序及轨道优选规则，不另定义新策略。
- 播放复用 `call_player`，按现有协议传递版本、三类轨道、起始 ticks 和直链；`download` 始终为 false，页面没有下载入口。
- 同服务器同系列的开始通知跟随切集并携带选择；停止通知刷新 UserData，保留版本/轨道并提供重试。观看标记与进度刷新通过版本计数隔离，旧响应不覆盖新标记。
- 系列页补齐外部资料、季与逐集标记、未看数量、集号/日期/版本规格。季与页码通过 query 恢复；全部剧集的空季选择同样保留。系列/季已看操作成功后刷新相关列表。
- 前端平台操作通过 `platformDataKey` 注入；预览/测试不访问真实 Tauri，也不启动播放器或系统浏览器。缺源、缺 UserData 和故障模拟保留。
- 41 项 Node 回归测试、Vue 类型检查、Vite 配置 TypeScript 检查、桌面和 Android 前端构建通过。产物在 `/tmp/loemby-detail-parity-desktop-20261106` 与 `/tmp/loemby-detail-parity-android-20261106`。仍有原有 `@tailwind` 未处理及主包超过 500 kB 警告。
- 本轮未修改 Rust、Kotlin/JNI 或原生播放器，也未构建 APK、安装设备或验证真实播放；旧 APK 不含这些页面变更。
- 开发预览成功启动，HTML、入口、详情/系列页面及资料组件 HTTP 编译检查均返回 200。自动打开浏览器失败于缺少 `xdg-open`，当前会话无可调用的浏览器控制工具，未进行窄屏/横屏/主题及点击的 GUI 验收；HTTP 和无 DOM 测试不能替代浏览器/真机验收。

### 后续原生/后端待办

1. Android `open_url` 系统浏览器 Intent 桥接：当前页面提供资料入口与失败反馈，但已有 Rust 实现使用桌面打开方式，不能宣称 Android 已可打开。
2. 真机验证开播前的轨道关闭/选择、外挂字幕/音轨、章节起始位置、直链与同季连播，以及返回详情后的进度。
3. 原生播放器内补齐桌面对应的版本/轨道/章节菜单和播放控制；本轮仅实现详情页面，未更改原生控制窗口。
4. 补接异步播放错误到详情；共享前后向缓存、字幕预缓存、预取及可移植的播放配置/脚本能力；版本选择按 MediaSource.Id 绑定，避免两次服务端返回排序变化造成错配。
5. 下载已按需求从 Android 功能范围取消，不作为后续待办。桌面操作系统特有能力需要 Android 等价实现，不能用前端 mock 通过替代原生验收。

## 本轮浏览主链路验证（2026-11-05）

以下为历史记录，不代表当前播放能力或本轮验证结果。

- 21 项 Node 回归测试、TypeScript 检查、桌面与 Android 平台前端构建已通过，构建输出位于 `/tmp/loemby-desktop-browse-build` 和 `/tmp/loemby-android-browse-build`。
- 开发预览 HTML、入口、模拟目录、导航布局与五个页面的 HTTP 编译检查均返回 200；两套生产构建未包含模拟目录。
- 后端已跟踪文件差异与本轮开始时逐字一致，本轮未修改 `src-tauri/`；不执行原生或 APK 构建。
- 构建仍提示主包超过 500 kB，未屏蔽该提示。
- 当前会话缺少浏览器控制调用工具，未进行 360px/390px、横屏、主题切换与实际点击的 GUI/视觉验收；HTTP 和无 DOM 测试不能替代它们。
- 未验证真实 Android 设备、Emby 服务连接与配置的可用性。此前 APK 不包含本轮页面变更，需要后续重新构建才能在 APK 中使用。

## 前端依赖升级兼容验证（2026-11-05）

- 保留 Vite 8、Router 5 和 Pinia 4，Vue 运行时与编译器对齐至 3.5.45；显式声明 SVG 插件所需 `fast-glob` 和路由测试所需 `esbuild`。
- 浏览测试改用 Vite 公开 `transformWithOxc`，全部 21 项回归通过；Vue 类型检查和 Vite 配置的独立 TypeScript 检查通过。
- 桌面/Android 正式前端构建脚本通过，输出在 `/tmp/loemby-upgrade-desktop-final` 和 `/tmp/loemby-upgrade-android-final`；开发预览及 SVG 注册模块返回 200。
- 冻结锁文件离线安装与严格 peer 检查通过，未回退已升级依赖，未修改 Rust/Android 原生文件或打包 APK。
- Vite 8 CSS 压缩仍提示全局样式中的 `@tailwind` 未处理，主包仍超过 500 kB；这两项为非阻塞警告，未通过关闭检查掩盖。
- 没有浏览器控制调用工具，未验证实际拖拽、线路选择或真机 GUI；HTTP/无 DOM 回归不是视觉验收。

## 最新 APK 与真机安装记录（2026-11-05）

- 已重新构建包含最新浏览页面、历史标记修复、依赖升级和指定 Android 图标的 arm64 Debug APK，包名 `com.loemby.app`，版本 `0.45.3`（45003），最低 API 24，目标 API 34。
- 原始 Gradle APK 位于 `src-tauri/gen/android/app/build/outputs/apk/arm64/debug/app-arm64-debug.apk`，329,155,212 bytes。发现大量空白填充后另行重打 ZIP、16 KB 对齐并使用同一开发密钥签名；原始文件保留，所有 ZIP 文件条目内容逐项一致。
- 交付：`artifacts/android/build-20261105-223148/loemby-0.45.3-arm64-v8a-debug.apk`，69,972,196 bytes（66.73 MiB），旁附 `.apk.sha256`。
- SHA-256：`cd0614ffdfa44de1b7618c8959466fb4e90f68d97fb8305d9b6a9061bcf65aba`。
- 已通过 APK v2 签名、16 KB ZIP 对齐与 native LOAD 段对齐检查；15 个源图标同步一致，标准/圆形图标清单引用有效，仅包含 `arm64-v8a` 原生库。
- 已在连接的 BVL-AN16（Android 16）上使用 `adb install -r` 成功覆盖安装，首次安装时间不变，没有卸载或清空数据。开发签名与旧应用一致。
- `am start -W` 冷启动返回 `Status: ok`，应用持续在前台运行；启动日志未发现 FATAL EXCEPTION、Fatal signal 或 Rust panic，原生初始化与历史查询日志正常出现。
- 未做完整真机页面点击/横屏/视觉验收，也未验证媒体服务器 HTTPS 连通性或原生播放；播放、下载仍暂未适配。
- 此为开发签名测试包。现有 DEBUG 日志会输出敏感服务器配置字段；不要公开完整运行日志，正式发布前应修复日志脱敏，本轮未修改后端日志行为。

## 此前安装包验证记录（2026-11-05）

- 锁文件依赖安装、类型检查、桌面和 Android 平台前端构建已通过。
- 10 项状态、路由与 SSR 内容回归测试已通过；Vite 开发预览入口 HTTP 检查已通过。
- Android `aarch64-linux-android` 原生编译和 Gradle APK 打包已通过。
- APK 为 `com.loemby.app`、版本 `0.45.3`（versionCode `45003`），仅含 `arm64-v8a`，最低 Android 7.0（API 24），目标 API 34。
- 已验证 APK v2 开发签名、ZIP 对齐、原生库 16 KB LOAD 段对齐，以及 Android TLS verifier 已打入 DEX。
- 交付：`artifacts/android/loemby-0.45.3-arm64-v8a-debug.apk`（70,589,700 bytes，67.32 MiB），旁附 `.apk.sha256`。
- SHA-256：`7ffe03c60b4ae8e07ba548db2c963adf3746c3273f1634e55430b54a21597fcb`。
- 构建提示包括前端主包超过 500 kB、Tauri Android 库的 compileSdk 36 超出 AGP 8.5.1 已测试范围、Java 8 编译目标弃用和少量 Rust 未使用代码；未作为错误屏蔽。
- 当前会话没有浏览器控制工具，未完成窄屏/横屏 GUI 和视觉验收。
- 没有设备或模拟器安装启动测试，未验证真实设备上的 SQLite 初始化或 HTTPS 请求；Android 原生播放仍未适配。本包是测试包，不是商店发布包。
