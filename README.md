# oclive 启动器

独立 **桌面启动器**（Tauri 1.x + Vue 3 + TypeScript）：集中管理 **角色包编写器**（`oclive-pack-editor`）与 **oclive 运行时**（例如 `oclivenewnew`）的启动方式，把子进程 **stdout / stderr** 收到本窗口，避免弹出多个控制台；支持 **公告栏**、**GitHub Release 版本检查** 与 **打开发布页**（便于下载安装包）。界面与 **oclive-pack-editor** 共用 **Fluent Design** 变量（与工作区 **ok-ww / qfluentwidgets** 的层次、强调色一脉）。

## 功能概览

| 区域 | 说明 |
|------|------|
| **公告栏** | 本地 Markdown/纯文本，保存到应用配置目录下的 `announcements.md` |
| **版本与更新** | 分别为编写器、oclive 填写 GitHub `owner/repo`，检查远端最新 Release；本地版本从各项目根目录 `package.json` 读取 |
| **启动** | 每个应用可选 **开发模式**（在项目根执行 `npm run <脚本>`，默认 `tauri:dev`）或 **exe 模式**（直接运行已构建的 `.exe`） |
| **运行日志** | 子进程在 Windows 上使用 **无控制台窗口** 启动，输出汇总到下方日志区，可按应用筛选 |

## 环境

- Node.js（建议 LTS）、`npm install`
- 桌面开发与打包：**Rust**、**Tauri 1.x** 与 Windows 上 **WebView2**（与编写器相同）

## 命令

**Windows**：根目录 **`start.bat`** 双击即可（默认 `npm run tauri:dev`）；`start.bat web` 仅浏览器预览 `http://127.0.0.1:5174`（无 Tauri 能力）。

```bash
npm install
npm run tauri:dev    # 开发：Vite 端口 5174
npm run build        # 前端产物 → dist/
npm run tauri:build  # 安装包 / 可执行文件
```

首次若缺少 `dist/`，`predev` / `tauri:dev` 前会自动执行 `scripts/ensure-dist.mjs` 生成占位文件，满足 Tauri 对 `distDir` 的检查。

## 配置存储

配置保存在系统应用配置目录下的 `launcher-config.json`（随平台而异，Windows 一般在 `%APPDATA%\com.oclive.launcher\` 或类似路径）。

## 与双开浏览器的关系

本仓库 **不包含** 编写器或 oclive 源码；请在界面中填写本地克隆路径。若仅用 `npm run dev` 启动编写器且使用 `vite --open`，可能与 Tauri 子窗口同时出现浏览器——编写器仓库已提供 **`dev`（无 `--open`）** 与 **`dev:browser`** 的区分，详见该仓库 README。

## 许可

MIT（与 oclive 生态其他仓库对齐时可再统一）。
