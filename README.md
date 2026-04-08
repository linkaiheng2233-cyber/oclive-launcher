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

## 新用户：从下载到第一次对话（推荐路径）

三软件分工：**编写器**产出角色包 → 磁盘上的 **`roles/` 根** → **运行时**加载并对话；**启动器**负责配置路径、注入环境变量并一键启动前两者。

| 步骤 | 做什么 |
|------|--------|
| 1 | 安装 **Node.js LTS**、**Ollama**（本地对话默认依赖本机模型）。 |
| 2 | 获取三个应用：克隆或下载 [oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew)、[oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor)、本仓库；或使用各仓库 **Release** 构建的 `.exe`。 |
| 3 | 打开本启动器，在 **「启动」** 中填写 **编写器**与 **oclive 运行时** 的项目根（开发模式）或可执行文件路径（exe 模式）。 |
| 4 | 在同一页的 **oclive 运行时** 卡片中填写 **角色包根目录**（环境变量 **`OCLIVE_ROLES_DIR`**）。若 oclive 项目为克隆的 `oclivenewnew`，可点 **「从 oclive 仓库填入」** 自动填入仓库内 `roles/`。启动 oclive 时由启动器注入该变量；留空则不在此注入（你也可在系统环境中自行设置）。 |
| 5 | 用编写器编辑或导入包并 **导出 zip**，解压到 `OCLIVE_ROLES_DIR/某角色id/`；或使用编写器 **「写入文件夹」** 直接写入该根目录。 |
| 6 | 在启动器 **启动 oclive**，在应用内选择角色并开始对话。 |

**环境与排障** 页会检测 Node / npm、Ollama CLI 与 `127.0.0.1:11434`、项目路径及角色包根目录；若 Ollama 未就绪会显示 **醒目提示** 并链接官方下载页。

### 你可以提前准备的材料（可选）

- 三个仓库在本机的 **绝对路径**（便于一次填对）。
- 一个空的 **文件夹** 作为自定义 `roles` 根，或直接使用 **oclivenewnew 仓库内的 `roles/`**（内含示例 `mumu`）。
- 若使用 Ollama 自定义模型：记下 **模型名**（在 oclive 的 `settings`/环境变量中配置，见 oclivenewnew 文档）。

### GitHub「owner/repo」是什么（不是法律协议，是使用约定）

- **owner**：GitHub 上的用户或组织名；**repo**：仓库短名。合起来唯一对应 `https://github.com/owner/repo`。
- 启动器用它们拼 **Releases** 地址并调用 **GitHub 公开 API**（`GET /repos/{owner}/{repo}/releases/latest`）检查版本，因此仓库需对你**可读**（公开仓库最省事）。
- **默认占位**：首次使用或两项均为空时，会填入当前生态的**上游** `owner/repo`，减少手填；若你用自己的 **fork**，请改成你的仓库名。
- **API 限流**：匿名请求有频率上限；仅偶尔点「检查更新」一般足够。若将来需要高频或私有仓库，再考虑 **Personal Access Token**（当前未实现）。

### Ollama：程序下载 vs 模型下载

- **安装 Ollama**：从 [ollama.com/download](https://ollama.com/download) 安装的是**程序本体**；装好后需保持服务运行（托盘图标）。
- **模型**：安装 ≠ 已有模型。需在终端执行 `ollama pull <模型名>`，模型列表见 [Ollama Library](https://ollama.com/library)。角色包里的 `settings.json` 会写默认模型名（生态默认推荐 `qwen2.5:7b`），请确保本地已 `ollama pull` 对应名称。
- **网络**：安装包与 `pull` 若较慢，可重试或查阅 Ollama 官方文档；国内环境以官方说明为准。

### 「仅玩家」整合包与 Release（规划向）

若将来提供**整合安装包**（运行时 + 空 `roles` + 启动器），需要与各仓库 **Release 资产命名**、版本号对齐；编写器与运行时的 **Release 下载入口**已在本启动器「版本」页通过 **Releases** 链接提供。

**资产命名约定（生态一致）**：按软件区分前缀，再加平台与语义化版本，例如 `oclive-pack-editor-windows-v1.2.3.zip`、`oclivenewnew-windows-v1.2.3.zip`、`oclive-launcher-windows-v1.2.3.zip`（具体以各仓库 CI 为准）。

## 功能概览

| 区域 | 说明 |
|------|------|
| **公告栏** | 本地 Markdown/纯文本，保存到应用配置目录下的 `announcements.md` |
| **版本与更新** | 分别为编写器、oclive 填写 GitHub `owner/repo`，检查远端最新 Release；本地版本从各项目根目录 `package.json` 读取 |
| **启动** | 每个应用可选 **开发模式**（在项目根执行 `npm run <脚本>`，默认 `tauri:dev`）或 **exe 模式**（直接运行已构建的 `.exe`） |
| **第一次使用** | 上手步骤与三仓库关系；鼓励玩家与创作者自由尝试 |
| **首次启动** | 自动跑一次环境检测一次（本地记忆），状态栏提示欢迎语 |
| **环境与排障** | **一键检测** Node / npm / Ollama（CLI 与 `127.0.0.1:11434` API）、编写器/oclive 项目目录、`OCLIVE_ROLES_DIR` 是否有效；Ollama 未就绪时 **横幅提示**；**打开配置目录**；**一键重置**损坏的 `launcher-config.json`（原文件尽量备份为 `launcher-config.json.corrupt.bak`）；附 Node / Ollama 官方下载链接 |
| **启动** | 为 **oclive** 子进程注入 **`OCLIVE_ROLES_DIR`**（若已填写）；支持从 oclivenewnew 仓库根 **一键填入 `roles/`** |
| **推理后端（大脑）** | 可选 **本机 Ollama** 或 **云端 Remote LLM**：注入 **`OCLIVE_LLM_BACKEND`**（`ollama` / `remote`），运行时覆盖角色包内 `plugin_backends.llm`；云端需填 JSON-RPC 端点 URL，可选 Token 与超时（见 oclivenewnew **`REMOTE_PLUGIN_PROTOCOL.md`**） |
| **角色包 zip 安装** | 「从 zip 安装角色包」：解压编写器导出的包到 `roles/` 下，对话框选择 **Ollama 模型**（默认 `qwen2.5:7b`）、本机已拉取列表、或 **手动输入**；可选是否 **覆盖** `settings.json` 里已有 `model`；可一键 **`ollama pull`**（日志筛选「ollama」） |
| **运行日志** | 子进程在 Windows 上使用 **无控制台窗口** 启动，输出汇总到下方日志区，可按应用筛选（含 **ollama pull**） |

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
