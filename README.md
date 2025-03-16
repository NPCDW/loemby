# loemby

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

## 相关文档
- Emby Github [https://github.com/MediaBrowser/Emby/wiki](https://github.com/MediaBrowser/Emby/wiki)
- Emby Swagger [https://swagger.emby.media/?staticview=true#/](https://swagger.emby.media/?staticview=true#/) 或者 `http[s]://自建地址[:8096]/emby/[openapi | swagger]`
- Vue [https://vuejs.org/](https://vuejs.org/)
- Element Plus [https://element-plus.org/zh-CN/](https://element-plus.org/zh-CN/)
- Tauri V2 [https://v2.tauri.app/](https://v2.tauri.app/)
