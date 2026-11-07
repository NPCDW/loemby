# Android libmpv native bundle

## 播放接入架构

移动详情页沿用 `call_player`。Rust 共用入口读取服务器和播放器配置，由同一份参数构建提供 `mpv_args`、音量、User-Agent、预读取及网速显示选项。桌面以进程启动 mpv，Android 在 `mpv_initialize()` 前加载会话配置并应用相同选项，仅保留原生渲染、IPC 和加载暂停等平台约束；Android 显式加载共用的 `cache_speed.lua`，不加载桌面 uosc/窗口配置。

两端共用播放计划、loopback 媒体/字幕代理和 `playback_process`。每个播放器实例只建立一个 JSON IPC 读循环，在 loadlist 前启动；媒体解析和字幕预缓存在 HTTP 重定向前登记到会话。命令响应使用独立 request_id，播放条目通过 start-file 的 playlist_entry_id 与本地媒体 URL/版本关联，支持预读取、切集和桌面切版本。媒体缓存配置、初始化、进度轮询与结束结算共用；Android 加载完成仍以每次加载独立 token 通知原生界面。退出只清理所属会话的请求、计划和临时文件。

`src-tauri/plugins/tauri-plugin-mpv` 是仓内 Tauri Android 插件。Kotlin 的独立播放器 Activity 提供播放/暂停、seek、上下集与退出；JNI 注册 JavaVM、管理 libmpv 及 owner 事件线程。界面采用 TextureView，在 SurfaceTexture 销毁回调返回 false，保留底层纹理到工作线程的 `mpv_terminate_destroy()` 完成后再释放。首版返回、后台或纹理失效时结束播放，不支持后台播放/画中画/下载。

首版只支持 `arm64-v8a`。普通 Tauri/Gradle 应用构建不下载 mpv 或 FFmpeg：先显式准备 native bundle，插件的 `preBuild` 再调用离线 `check`；缺失产物会明确失败。没有从 APK 提取 `.so`，也没有二进制下载服务或 floating `latest`。

## 固定输入与兼容矩阵

完整源码 URL、版本、Git commit、release archive SHA256 和递归子模块写在 `scripts/android-mpv.lock.json`。

- 官方构建配方：mpv-android `255901dd586fa750000fa0075f69d6ffdd26cd9c`。
- mpv：`902818753849d9ad8f0096fad6ca5aeaa37113b9`。
- FFmpeg：官方 `n9.0.2`，实际 commit `946fcce07b6dcd0331c8cc609192aeff5e1924f8`。
- 依赖：mbedTLS 3.6.7、libxml2 2.15.4、FreeType 2.15.0、Fontconfig 2.18.3、FriBidi 1.0.17、HarfBuzz 15.0.0、libunibreak 8.0、Lua 5.2.4、curl 8.23.0；dav1d/libass/libplacebo 及 libplacebo/FreeType 子模块均固定实际 commit，不在构建时重新解析 HEAD。
- Linux x86_64 host，Python >=3.12（建议 3.13），Meson **1.11.0**。
- NDK **27.2.12479018 (r27c)**，native API **24**，与应用及插件一致。官方该配方声明 r30；本地使用已可获得的 r27c，未悄悄冒充 r30。
- compileSdk/targetSdk **36**、build-tools **36.0.0**、AGP **8.10.1**、Gradle **8.11.1**、JDK >=17。实际 Tauri 2.12.5 `mobile/android/build.gradle.kts` 使用 compileSdk36；AGP8.5/8.9不在API36正式支持矩阵内。

脚本保留官方功能取舍：FFmpeg JNI/MediaCodec/mbedTLS/dav1d/libxml2、GPL/version3、共享 FFmpeg；Lua/libass/libplacebo/curl 静态链接到共享 mpv；Vulkan 关闭，Android EGL/OpenSLES 显式启用。Meson `wrap_mode=nodownload`，禁止缺失依赖时隐式联网。官方 URL 不通时只换经相同 commit 核验的官方 GitHub mirror（FreeType/Fontconfig/libxml2）。

## 准备

下载源代码是执行第三方构建之前独立的一步。源码和 archive 放进独立新目录，脚本/构建工具/工作目录在不同位置；不要从下载目录运行 Python（需 `-I`）或不经检查执行其脚本。

```sh
sudo apt-get update
sudo apt-get install -y git curl autoconf automake libtool ninja-build pkg-config gperf nasm cmake python3-venv
sdkmanager 'platforms;android-36' 'build-tools;36.0.0' 'ndk;27.2.12479018'
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export NDK_HOME="$ANDROID_NDK_HOME"

# 工具环境与源码/下载独立；Meson wheel 下载后核验再安装。
python3 -I -m venv /tmp/loemby-mpv-tools
mkdir -p /tmp/loemby-mpv-tools-download
/tmp/loemby-mpv-tools/bin/python3 -I -m pip download --no-deps meson==1.11.0 -d /tmp/loemby-mpv-tools-download
printf '%s  %s\n' 9774eb62bbf2cb8043aa3feb771c09b510d112c0c382221e29aea7453ecce753 /tmp/loemby-mpv-tools-download/meson-1.11.0-py3-none-any.whl | sha256sum -c -
/tmp/loemby-mpv-tools/bin/python3 -I -m pip install --no-deps /tmp/loemby-mpv-tools-download/meson-1.11.0-py3-none-any.whl
export PATH="/tmp/loemby-mpv-tools/bin:$PATH"

python3 -I scripts/android-mpv.py fetch
python3 -I scripts/android-mpv.py audit --print-recipes
# 检查上面的官方配方，以及源码 configure/meson/Makefile，再明确允许源构建。
python3 -I scripts/android-mpv.py build --accept-reviewed-recipes 2>&1 | tee /tmp/android-mpv-build.log
python3 -I scripts/android-mpv.py check --abi arm64-v8a
```

默认目录：

- 源码 `/tmp/loemby-android-mpv-sources`（每个依赖独立，archive另有 `*-download` 目录）。
- 工作/安装 prefix `/tmp/loemby-android-mpv-build`。
- bundle `src-tauri/plugins/tauri-plugin-mpv/native`。

使用 `--sources`、`--work`、`--bundle` 可覆盖。插件和脚本共同读取 `LOEMBY_MPV_NATIVE_DIR`。`prepare --accept-reviewed-recipes` 是 fetch+build 的快捷入口，第一次推荐仍分开执行审核。build 的完成 stamp 只在同一个 lock/NDK 工作目录复用；换输入请使用新工作目录。现有非空 bundle 不会被静默覆盖，先明确移走旧目录。

```text
native/
  include/mpv/client.h
  include/libavcodec/jni.h
  jniLibs/arm64-v8a/libmpv.so
  jniLibs/arm64-v8a/libavcodec.so
  jniLibs/arm64-v8a/其他实际 DT_NEEDED 依赖.so
  licenses/每个源码与子模块的原始许可证/notice
  manifest.json
```

manifest 包含源码/version/commit/archive SHA256、官方配方 SHA256、NDK/API、构建/link参数、每个 bundle 文件 SHA256/大小、许可证索引，以及每个 ELF 的 SONAME/DT_NEEDED/LOAD 对齐。manifest 自身没有自指 hash；artifact/传输层应另核验整包 hash。源码与 artifact 可追踪不等于位级可复现：host autotools/compiler版本、时间戳和绝对路径尚可能影响字节。

## 核验与应用构建

`check --bundle PATH --abi arm64-v8a [--ndk PATH]` 不联网，也不需要 host readelf（使用标准库 ELF64 解析）。核验锁、文件精确清单与 hash、必需头文件/库、AArch64 ET_DYN、SONAME与文件名一致、每个 DT_NEEDED 的闭包、禁止TEXTREL与系统库重复打包、每个 LOAD 至少16KB且offset/vaddr模16KB一致。`libc++_shared.so` 不是系统库；出现于 DT_NEEDED 时必须携带匹配 NDK runtime。FFmpeg Android configure 使用无版本 SONAME，不能只重命名版本化 Linux `.so`。

所有源码共享链接使用 `-Wl,-z,max-page-size=16384` 与 `-Wl,-z,common-page-size=16384`；插件另开启 `ANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON`。Rust最终共享库也必须传同样参数，否则bundle合格不代表最终APK合格。

```sh
ndk="$ANDROID_NDK_HOME"
toolchain="$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin"
export CC_aarch64_linux_android="$toolchain/aarch64-linux-android24-clang"
export CXX_aarch64_linux_android="$toolchain/aarch64-linux-android24-clang++"
export AR_aarch64_linux_android="$toolchain/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$toolchain/aarch64-linux-android24-clang"
export CMAKE_GENERATOR=Ninja
export RUSTFLAGS='-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-z,common-page-size=16384'
rustup target add aarch64-linux-android
pnpm install --frozen-lockfile
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
pnpm tauri android build --target aarch64 --apk --debug
pnpm tauri android build --target aarch64 --apk
python3 -I scripts/android-mpv.py check-apk --apk path/to/app.apk
```

`check-apk` 对**本项目生成**APK验证所有native库的ABI/DT_NEEDED闭包/16KB ELF，拒绝重复native ZIP项，再用build-tools36的 `zipalign -c -P 16 -v 4` 验证ZIP页面对齐。这不是从第三方APK准备native bundle。缺少私有 `keystore.properties` 时release无签名，只用于编译验证；正式安装/分发需要自行签名，不在仓库/CI提交密钥。

`.github/workflows/android.yml` 仅 `workflow_dispatch`，编译native与可选debug/release APK，上传artifact和失败日志，不触发release，不改桌面`build.yml`。CI不得把编译成功当设备测试成功。

## 分发义务与验证边界

此配方启用 FFmpeg `--enable-gpl --enable-version3`，组合分发按 GPL-3.0-or-later 义务处理；仅附license文件不能代替对应源代码。分发者需提供准确对应源码（含所有修改）、构建/安装信息和许可证notice，遵守各静态依赖条款；Android系统库不拷入bundle，NDK libc++ runtime附其notice。应用自己的MIT声明不覆盖mpv/FFmpeg。发布前还需法律/codec专利评估。

本次环境已安装必要host工具、下载核验固定源码，**实际完成**NDK27.2 arm64全部依赖和libmpv编译/链接/安装及bundle核验：9个共享库（libmpv、FFmpeg7库、匹配NDK libc++ runtime）、58个文件的hash/许可证索引、SONAME/DT_NEEDED闭包和16KB ELF通过。实际版本包括mpv `0.42.0-UNKNOWN`（浅clone无tag名，精确commit已记录）、dav1d1.5.5、libass0.17.5、libplacebo7.387.0。

首次真实构建失败于Fontconfig要求Meson>=1.11.0（旧固定1.9.2），已升级到经hash核验的1.11.0，不patch降低上游约束；首次打包失败于Lua许可证不在独立LICENSE文件，已按原始doc/readme.html/src/lua.h收集。最后成功日志 `/tmp/loemby-android-mpv-build-final.log`，最后脚本/来源核验日志 `/tmp/loemby-native-final-script-validation.log`。

主应用 **debug/release APK已实际构建成功**，Kotlin/JNI编译通过，两个包的 `check-apk` 全部 native ABI/闭包/16KB ELF和ZIP对齐通过。release混淆映射确认 `MpvPlugin`、`MpvNative` 和参数类保留原名；本环境已有签名配置，release签名核验通过。设备未连接：播放、MediaCodec、Surface生命周期、release运行和16KB真机运行未经验证，不宣称播放器已真机可用。
