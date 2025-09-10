# loemby

Emby 客户端主要针对目标导向型用户设计，能为目标明确的内容检索提供良好体验，但对被动浏览型用户的支持较为有限。

当前仅官方支持 Windows 平台，理论上兼容 Linux 和 macOS，但尚未经过完整测试，不保证稳定性。

# 使用
前往 [Release](https://github.com/NPCDW/loemby/releases) 下载对应平台的安装包，安装后即可使用。

该软件仅支持浏览搜索等，如果需要观看视频，需要自行下载 mpv 视频播放器，并在设置中进行配置。推荐以下 mpv 播放器以及整合包：
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
