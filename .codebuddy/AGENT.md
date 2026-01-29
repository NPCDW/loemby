# Loemby - CodeBuddy Agent 配置

## 项目概述
Loemby 是一个基于 Tauri + Vue 3 的 Emby/Jellyfin 客户端应用，支持 Windows、Linux 和 macOS 平台。

## 技术栈
- **前端框架**: Vue 3 (Composition API)
- **UI 组件库**: Element Plus
- **桌面框架**: Tauri V2
- **语言**: TypeScript
- **状态管理**: Pinia
- **路由**: Vue Router 4
- **构建工具**: Vite
- **后端语言**: Rust (Tauri)

## 项目结构
```
/workspace/
├── src/                    # Vue 前端源码
│   ├── api/               # API 调用
│   ├── components/        # Vue 组件
│   ├── page/             # 页面组件
│   ├── router/           # 路由配置
│   ├── store/            # Pinia 状态管理
│   ├── types/            # TypeScript 类型定义
│   └── util/             # 工具函数
├── src-tauri/            # Rust 后端源码
│   ├── src/
│   │   ├── service/      # 业务服务层
│   │   ├── mapper/        # 数据模型
│   │   └── config/         # 配置
│   ├── capabilities/     # Tauri 能力配置
│   └── resources/        # 资源文件
└── public/               # 静态资源
```

## 编码规范

### Vue/TypeScript
- 使用 Composition API 和 `<script setup>` 语法
- 使用 TypeScript 类型注解
- 组件文件使用 PascalCase 命名（如 `UserInfo.vue`）
- 使用 Element Plus 组件库
- 自动导入已配置：Element Plus 组件和图标可直接使用

### Rust (Tauri)
- 遵循 Rust 2021 edition 标准
- 使用 `anyhow` 进行错误处理
- 使用 `tokio` 进行异步处理
- 使用 `tauri` 相关 API 与前端通信

## 开发工作流

### 依赖管理
使用 pnpm 管理前端依赖：
```bash
pnpm install
```

### 开发启动
```bash
# 开发环境
pnpm run tauri dev --config src-tauri/tauri.dev.conf.json

# 测试预览
pnpm run tauri dev --config src-tauri/tauri.beta.conf.json

# 生产构建
pnpm run tauri build
```

## 注意事项
- 项目当前主要面向 Windows 平台，但理论上支持 Linux 和 macOS
- 内置 mpv 播放器，也支持外部 mpv
- 集成了 Trakt 和 Simkl 的授权功能
- 数据库使用 Tauri 的本地存储能力

## 外部依赖
- Emby/Jellyfin API
- Trakt API
- Simkl API
- mpv 播放器

## 文档参考
- Emby: https://github.com/MediaBrowser/Emby/wiki
- Tauri V2: https://v2.tauri.app/
- Vue 3: https://vuejs.org/
- Element Plus: https://element-plus.org/zh-CN/
