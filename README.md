# VividDown

一款基于 **Tauri**、**Rust** 和 **Svelte** 构建的功能强大、现代且易于使用的视频下载器。它致力于为下载视频、播放列表和 Shorts 短视频提供高级体验，支持完整的身份验证与高性能多线程下载。

## ? 功能特性

- ? **全能支持**：支持下载标准 YouTube 视频、Shorts 短视频以及完整的播放列表。
- ? **高性能加速**：集成 **aria2** 实现多线程下载，极大提升带宽利用率。
- ? **身份验证支持**：支持通过集成的浏览器登录或导入 Cookie（Netscape 格式），轻松下载受限或私有内容。
- ?? **Cookie 智能验证**：内置 Cookie 校验机制，确保导入的 YouTube 身份验证令牌有效且未过期。
- ? **队列管理**：实时跟踪下载进度，支持暂停、恢复或取消任务。
- ? **现代 UI**：基于 Svelte 5 构建的极简交互界面，支持深色模式、玻璃拟态及流畅的微动画。
- ?? **智能设置**：支持自定义并发下载限额、默认分辨率，并可在应用内直接管理外部工具（yt-dlp, FFmpeg, aria2）。
- ? **快速访问**：支持一键打开下载文件夹或复制视频链接。

## ?? 技术栈

- **前端**: [Svelte 5](https://svelte.dev/) + [Vite](https://vitejs.dev/)
- **后端**: [Rust](https://www.rust-lang.org/) + [Tauri 2](https://tauri.app/)
- **下载核心**: [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- **加速引擎**: [aria2](https://aria2.github.io/)
- **媒体处理**: [FFmpeg](https://ffmpeg.org/)

## ? 快速上手

### 环境准备

- [Rust](https://www.rust-lang.org/tools/install) (最新稳定版)
- [Node.js](https://nodejs.org/) (v18+)
- [Tauri 依赖环境](https://tauri.app/v2/guides/getting-started/prerequisites/)

### 安装与开发

1. **克隆仓库**:
   ```bash
   git clone https://github.com/yourusername/vivid-down.git
   cd vivid-down
   ```

2. **安装依赖**:
   ```bash
   npm install
   ```

3. **运行开发模式**:
   ```bash
   npm run tauri dev
   ```

### 生产环境构建

```bash
npm run tauri build
```

## ? 使用指南

1. **粘贴链接**: 复制视频 URL（视频、播放列表或 Shorts）并粘贴到输入框中。
2. **选择画质**: 选择您偏好的分辨率或“仅下载音频”。
3. **开始下载**: 点击下载按钮或按下 `Ctrl + Enter`。
4. **登录（可选）**: 如需下载受限内容，可通过“设置”面板导入 `cookies.txt` 文件或通过内置浏览器登录。

## ? 参与贡献

欢迎任何形式的贡献！请随时提交 Pull Request。

## ? 许可协议

本项目采用 MIT 许可协议。详情请参阅 `LICENSE` 文件。
