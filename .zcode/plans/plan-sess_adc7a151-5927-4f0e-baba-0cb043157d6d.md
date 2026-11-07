## Context
安卓实机已连接，详情页点击“从头播放”时 mpv 报错。目标是找出实际错误并修复，继续复用桌面端的媒体解析、代理及播放会话逻辑。目前已定位调用链，但计划模式拦截了 adb，尚未取得设备日志，不能认定根因。

## 实施步骤
1. **先取实机证据**：检查 `adb devices -l`、当前 Activity 和现有 logcat，重点查看 `LoembyMPV`、JNI、Rust 播放日志。必要时读取应用私有日志；不清除数据、不修改服务器配置，不在回复中暴露令牌或媒体直链。若日志不足，再复现一次“从头播放”。
2. **按实际报错做最小修复**：区分原生初始化、IPC、媒体代理与解码失败。主要检查文件为：
   - `src-tauri/plugins/tauri-plugin-mpv/android/src/main/cpp/mpv_jni.cpp`：libmpv 选项及初始化。
   - `src-tauri/plugins/tauri-plugin-mpv/android/src/main/java/com/loemby/mpv/MpvPlayerActivity.kt`：Surface 与原生错误处理。
   - `src-tauri/src/service/android_player.rs`：播放器准备与 IPC 启动。
   - 若证据指向共用业务，则修改对应的 `player_svc.rs`、`playback_session.rs` 或 `axum_svc.rs`，复用现有 `PlayerOptions`、`mpv_ipc::Client::request` 和播放会话实现，不另写安卓播放逻辑。
   只修改根因涉及的文件；若证据显示需要架构调整或改变用户配置，再确认。
3. **回归与打包**：运行涉及的测试、类型/编译检查，构建仅 arm64 的小 APK，去除不必要调试符号并压缩；安装前核验签名、native 依赖和 16 KiB 对齐。保留应用数据及现有签名，不卸载重装。
4. **实机验证**：安装修复包，验证从头播放、退出后重试、续播及播放进度，结合日志确认错误消失。若设备未检测到，按 `AGENT.md` 弹窗询问重新检测或跳过。必要的维护事项记入 `AGENT.md`；若调整容器环境，同步 `android-Dockerfile`。保留当前已有删除项，不提交或推送。

## 验收
以实机成功进入播放、出现正常画面/声音且日志无对应错误为最终依据；构建通过不等同于播放验证通过。报告具体根因、修改点和实际验证结果。