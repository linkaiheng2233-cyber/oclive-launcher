# oclive 启动器

[![CI](https://github.com/linkaiheng2233-cyber/oclive-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclive-launcher/actions/workflows/ci.yml)

独立 **桌面启动器**（Tauri 1.x + Vue 3 + TypeScript）：作为 **oclive 工具链的统一入口**，集中管理 **角色包编写器**（`oclive-pack-editor`）与 **oclive 运行时**（例如 `oclivenewnew`）的启动方式，把子进程 **stdout / stderr** 收到本窗口，避免弹出多个控制台；支持 **公告栏**、**GitHub Release 版本检查** 与 **打开发布页**。

设计取向：在参考「绘世」等成熟启动器 **开箱即用、保姆式排障、一站式管理** 的思路的同时，结合 oclive **开源、角色包为纽带、双软件分工** 的特点，逐步增强——**不追求像素级复刻**，优先把 **路径配置、依赖可见性、错误可恢复** 做扎实。

## 生态仓库（GitHub）

| 仓库 | 说明 |
|------|------|
| [oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew) | 运行时、HTTP API、`creator-docs` 与角色契约 |
| [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) | 角色包编写器（导出与校验） |

本地开发时可将三仓 **同级克隆**（例如 `D:\oclivenewnew`、`D:\oclive-pack-editor`、`D:\oclive-launcher`），在启动器里填写相对路径即可。

## 功能概览

| 区域 | 说明 |
|------|------|
| **公告栏** | 本地 Markdown/纯文本，保存到应用配置目录下的 `announcements.md` |
| **版本与更新** | 分别为编写器、oclive 填写 GitHub `owner/repo`，检查远端最新 Release；本地版本从各项目根目录 `package.json` 读取 |
| **启动** | 每个应用可选 **开发模式**（在项目根执行 `npm run <脚本>`，默认 `tauri:dev`）或 **exe 模式**（直接运行已构建的 `.exe`） |
| **环境与排障** | **一键检测** Node / npm / Ollama（CLI 与 `127.0.0.1:11434` API）、编写器/oclive 项目目录是否存在且含 `package.json`；**打开配置目录**；**一键重置**损坏的 `launcher-config.json`（原文件尽量备份为 `launcher-config.json.corrupt.bak`）；附 Node / Ollama 官方下载链接 |
| **运行日志** | 子进程在 Windows 上使用 **无控制台窗口** 启动，输出汇总到下方日志区，可按应用筛选 |

## 与「整合包 / 一键装模型」等方向的路线（长期）

以下能力在 **oclivenewnew** 主仓库文档 **`creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md`**（体验差异化 backlog）中与愿景一并维护；若本仓库与 `oclivenewnew` **同级克隆**，本地相对路径为 `../oclivenewnew/creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md`。启动器可逐步承接其中「玩家侧上手」部分：

| 方向 | 说明 |
|------|------|
| **整合包 / 离线安装** | 将启动器、Ollama、基础模型、示例角色打成一个安装包，实现「下载即聊」——需安装包工程与许可证策略，与 CI 发版流程配合。 |
| **更智能的依赖** | 在现有检测基础上，可扩展：引导安装 Ollama、拉取推荐模型、低配机策略等（须避免静默破坏用户环境）。 |
| **包 / 插件「商店」** | 浏览与一键安装社区角色包——依赖索引服务、签名与版本契约，见主仓库路线图。 |
| **开源协作** | 角色包、插件与文档由社区贡献，与主仓库 `CONTRIBUTING`、扩展点文档一致。 |

若本地未克隆 `oclivenewnew`，也可在浏览器打开主仓库中的同路径文档（以你托管的 URL 为准）。

## 环境

- Node.js（建议 LTS）、`npm install`
- 桌面开发与打包：**Rust**、**Tauri 1.x** 与 Windows 上 **WebView2**（与编写器相同）
- 使用 **oclive 对话** 时，本机通常需要 **Ollama**（或其它在 oclive 内配置的推理后端）；「环境与排障」页会检测常见情况并给出链接。

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

配置保存在系统应用配置目录下的 `launcher-config.json`（随平台而异，Windows 一般在 `%APPDATA%\com.oclive.launcher\` 或类似路径）。若 JSON **损坏无法解析**，启动时会加载默认配置，并尽量将原文件复制为 **`launcher-config.json.corrupt.bak`**；可在 **环境与排障** 中 **一键重置** 或手动恢复备份。

## 与双开浏览器的关系

本仓库 **不包含** 编写器或 oclive 源码；请在界面中填写本地克隆路径。若仅用 `npm run dev` 启动编写器且使用 `vite --open`，可能与 Tauri 子窗口同时出现浏览器——编写器仓库已提供 **`dev`（无 `--open`）** 与 **`dev:browser`** 的区分，详见该仓库 README。

## 许可

MIT（与 oclive 生态其他仓库对齐时可再统一）。
