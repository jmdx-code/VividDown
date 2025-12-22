# VividDown

一款基于 **Tauri**、**Rust** 和 **Svelte** 构建的功能强大、现代且易于使用的视频下载器。致力于为下载视频、播放列表和 Shorts 短视频提供高级体验。

> [!IMPORTANT]
> **平台支持说明**：本软件目前仅支持 **Windows** 系统。对 macOS 和 Linux 的支持正在规划中。

## 功能特性

- **全能支持**：支持下载标准 YouTube 视频、Shorts 短视频以及完整的播放列表。
- **高性能加速**：集成 aria2 实现多线程并行下载，极大提升带宽利用率。
- **认证登录**：支持通过集成的浏览器登录或导入 Cookie（Netscape 格式），轻松下载受限或私有内容。
- **安全校验**：内置 Cookie 智能验证，确保账号安全且令牌有效。
- **队列管理**：实时跟踪下载进度，支持暂停、恢复或取消任务。
- **现代 UI**：基于 Svelte 5 构建的响应式界面，支持深色模式与微动画。
- **智能设置**：可自定义并发限额、默认分辨率及管理外部工具路径。

## 技术栈

- 前端: Svelte 5 + Vite
- 后端: Rust + Tauri 2
- 下载引擎: yt-dlp + aria2
- 多媒体处理: FFmpeg

## 快速上手

### 环境准备 (仅限 Windows)

- Rust (最新稳定版)
- Node.js (v18+)
- Tauri 2 依赖

### 安装与运行

1. 克隆仓库:
   git clone https://github.com/yourusername/vivid-down.git
   cd vivid-down

2. 安装依赖:
   npm install

3. 启动开发环境:
   npm run tauri dev

## 使用指南

1. 粘贴链接: 复制视频/播放列表链接到输入框。
2. 开始下载: 点击下载图标或按 Ctrl + Enter。

## 许可协议

本项目采用 MIT 许可协议。
