# loemby

一个毛坯 Emby 客户端。

当前仅支持 Windows 平台，理论上兼容 Linux 和 macOS，但尚未经过测试，不保证可用性。

# 使用
前往 [Release](https://github.com/NPCDW/loemby/releases) 下载对应平台的安装包，安装后即可使用。

该软件内置了一个 mpv 播放器，也可以使用外部播放器，一些 mpv 播放器以及整合包：
1. [mpv.io](https://mpv.io/installation/)
2. [MPV_lazy](https://github.com/hooke007/MPV_lazy)
3. [mpv-config](https://github.com/dyphire/mpv-config)
4. [mpv.net-DW](https://github.com/diana7127/mpv.net-DW)

## 开发

安装依赖
```bash
pnpm i
```

启动
```bash
# 开发
pnpm run tauri dev --config src-tauri/tauri.dev.conf.json
# 测试预览
pnpm run tauri dev --config src-tauri/tauri.beta.conf.json
# 测试生产
pnpm run tauri dev
# 生产
pnpm run tauri build
```

## 生成图标
```bash
pnpm run tauri icon app-icon.svg
```

## 相关文档
- Emby Github [https://github.com/MediaBrowser/Emby/wiki](https://github.com/MediaBrowser/Emby/wiki)
- Emby Swagger [https://swagger.emby.media/?staticview=true#/](https://swagger.emby.media/?staticview=true#/) 或者 `http[s]://自建地址[:8096]/emby/[openapi | swagger]`
- Vue [https://vuejs.org/](https://vuejs.org/)
- Element Plus [https://element-plus.org/zh-CN/](https://element-plus.org/zh-CN/)
- Tauri V2 [https://v2.tauri.app/](https://v2.tauri.app/)
- Mpv [官方文档](https://mpv.io/manual/stable/) [中文文档](https://hooke007.github.io/official_man/mpv.html)
- Mpv osc [修改版](https://github.com/NPCDW/uosc.git) forked from [tomasklaen/uosc](https://github.com/tomasklaen/uosc)
- Mpv windows build [shinchiro/mpv-winbuild-cmake](https://github.com/shinchiro/mpv-winbuild-cmake)
