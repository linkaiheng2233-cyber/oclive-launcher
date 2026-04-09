# oclive 启动器

[![CI](https://github.com/linkaiheng2233-cyber/oclive-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclive-launcher/actions/workflows/ci.yml)

独立 **桌面启动器**（Tauri 1.x + Vue 3 + TypeScript）：作为 **oclive 工具链的统一入口**，集中管理 **角色包编写器**（`oclive-pack-editor`）与 **oclive 运行时**（例如 `oclivenewnew`）的启动方式，把子进程 **stdout / stderr** 收到本窗口，避免弹出多个控制台；支持 **公告栏**、**GitHub Release 版本检查** 与 **打开发布页**。

设计取向：在参考「绘世」等成熟启动器 **开箱即用、保姆式排障、一站式管理** 的思路的同时，结合 oclive **开源、角色包为纽带、双软件分工** 的特点，逐步增强——**不追求像素级复刻**，优先把 **路径配置、依赖可见性、错误可恢复** 做扎实。

## 生态仓库（GitHub）

| 仓库 | 说明 |
|------|------|
| [oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew) | 运行时、HTTP API、`creator-docs` 与角色契约 |
| [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) | 角色包编写器（导出与校验） |

**纽带与契约**：三应用只通过磁盘上的 **角色包** 连接；`min_runtime_version`、Remote 环境变量等语义见 oclivenewnew **[PACK_VERSIONING.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/role-pack/PACK_VERSIONING.md)** 与 **[REMOTE_PLUGIN_PROTOCOL.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**。

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

### 附带 Ollama 安装包（Windows，与启动器同发）

可将官方 **Ollama** Windows 安装包与启动器**打进同一安装包**分发：从 [ollama/ollama Releases](https://github.com/ollama/ollama/releases) 下载 **`OllamaSetup.exe`**（即官方安装程序，不是模型文件），放在本仓库根目录（与 `package.json` 同级），执行 `npm run tauri:build` 时 **`scripts/sync-ollama-installer.mjs`** 会自动将其复制到 `src-tauri/bundled/ollama/` 并打入安装包；若目标已存在且**字节大小一致**则跳过复制（加快反复构建）。也可手动放到该目录。用户在「环境」页可见 **「运行附带安装包」**。许可与再分发说明见 **`src-tauri/bundled/ollama/README.txt`**。

### 发版、Git 与大文件（`OllamaSetup.exe`）

| 做法 | 说明 |
|------|------|
| **推荐：不要提交安装包进 Git** | `OllamaSetup.exe` 体积大（约数百 MB），进主分支会让克隆与历史永久膨胀；根目录与 `src-tauri/bundled/ollama/` 下的该文件已在 **`.gitignore`** 中忽略。 |
| **分发大文件** | 使用 **GitHub Releases**（或其它发布渠道）上传「已构建的启动器安装包 / 整合 zip」；必要时单独附上 `OllamaSetup.exe` 或仅在「胖包」里内嵌，与「仅启动器」瘦包区分。 |
| **CI 构建** | 可在 workflow 中于 `tauri build` 前从官方 Release **下载**指定版本的 `OllamaSetup.exe` 到仓库根或 `bundled/ollama/`，再跑 `sync` 与打包；无需把二进制长期放在仓库里。 |
| **可选两种资产** | **瘦包**：不含 Ollama，用户自行安装；**胖包**：构建前放入 `OllamaSetup.exe`，用户可用「运行附带安装包」。 |

## 功能概览

| 区域 | 说明 |
|------|------|
| **开发者公告** | 应用配置目录 **`announcements.md`**；可选 **`developerAnnouncementsUrl`**（`launcher-config.json`）+ 「拉取最新」从 http(s) 同步并缓存到本地（非自动推送，需手动拉取） |
| **创作者公告** | 仅展示包内 **`creator_message.txt`**（可一句或多行）；作者在 **oclive-pack-editor** 导出，启动器**只读**；选角色见 **`launcherEchoRoleId`** |
| **版本与更新** | 分别为编写器、oclive 填写 GitHub `owner/repo`，检查远端最新 Release；本地版本从各项目根目录 `package.json` 读取 |
| **启动** | 每个应用可选 **开发模式**（在项目根执行 `npm run <脚本>`，默认 `tauri:dev`）或 **exe 模式**（直接运行已构建的 `.exe`）；为 **oclive** 子进程注入 **`OCLIVE_ROLES_DIR`**（若已填写）；支持从 oclivenewnew 仓库根 **一键填入 `roles/`** |
| **第一次使用** | 上手步骤与三仓库关系；鼓励玩家与创作者自由尝试 |
| **首次启动** | 自动跑一次环境检测一次（本地记忆），状态栏提示欢迎语 |
| **环境与排障** | **一键检测** Node / npm / Ollama（CLI 与 `127.0.0.1:11434` API）、编写器/oclive 项目目录、`OCLIVE_ROLES_DIR` 是否有效；Ollama 未就绪时 **横幅提示**；**Windows**：若本机有 **winget**，可 **一键安装官方 Ollama**；若打包时包含 **`bundled/ollama/OllamaSetup.exe`**，可 **运行附带安装包**（均非静默，可能弹 UAC）；**打开配置目录**；**一键重置**损坏的 `launcher-config.json`（原文件尽量备份为 `launcher-config.json.corrupt.bak`）；附 Node / Ollama 官方下载链接 |
| **外观与字号** | 日间 **暖色（象牙/卡其）** 界面；顶栏 **A− / 百分比 / A+** 调节缩放（与 **oclive-pack-editor** 档位一致，本地保存）；顶栏 **问号** 说明字号、主题与「保存配置」分工（配置落盘见下「配置存储」） |
| **推理后端（大脑）** | 可选 **本机 Ollama** 或 **云端 Remote LLM**：注入 **`OCLIVE_LLM_BACKEND`**（`ollama` / `remote`），运行时覆盖角色包内 `plugin_backends.llm`；云端需填 **JSON-RPC** 侧车 URL（非厂商原始 REST），可选 Token 与超时。协议见 oclivenewnew **`REMOTE_PLUGIN_PROTOCOL.md`**；**在本机用自带 API Key 接闭源模型**的用户向步骤见 **`SIDECAR_LLM_USER_GUIDE.md`**（[主仓库链接](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)） |
| **角色包 zip 安装** | 「从 zip 安装角色包」：解压编写器导出的包到 `roles/` 下，对话框选择 **Ollama 模型**（默认 `qwen2.5:7b`）、本机已拉取列表、或 **手动输入**；可选是否 **覆盖** `settings.json` 里已有 `model`；可一键 **`ollama pull`**（日志筛选「ollama」） |
| **运行日志** | 子进程在 Windows 上使用 **无控制台窗口** 启动，输出汇总到下方日志区，可按应用筛选（含 **ollama pull**、**winget**、**附带安装包**） |

### 仓库内的远程开发者公告示例

维护者可编辑本仓库 **[docs/announcements/developer.md](./docs/announcements/developer.md)**（索引见 **[docs/announcements/README.md](./docs/announcements/README.md)**），发布后把 **Raw** 链接填进启动器（`main` 分支示例）：

`https://raw.githubusercontent.com/linkaiheng2233-cyber/oclive-launcher/main/docs/announcements/developer.md`

## 随包寄语与职责边界（创作者公告）

| 角色 | 职责 |
|------|------|
| **文件名** | 固定为 **`creator_message.txt`**（与编写器 `src/lib/rolePackCreatorMessage.ts` 中常量、本仓库 `src/lib/rolePackCreatorMessage.ts` 的 **`ROLE_PACK_CREATOR_MESSAGE_FILENAME`**、`src-tauri/src/role_creator_message.rs` 保持同名；改名须**跨仓同步**） |
| **编写器（oclive-pack-editor）** | 在导出 / 写入文件夹时生成或编辑；可选 **整包一句**（只导出首条非空行）或 **按行多条**（每行一条，每行最多 **160** 字，Unicode 标量，以代码为准） |
| **启动器（本仓库）** | 从 **`roles/<角色id>/creator_message.txt`** 读取**全部非空行**并逐条展示；**不在此编辑**；**不**参与 oclive 对话逻辑 |
| **oclivenewnew 运行时** | **不读取** `creator_message.txt`；对话与角色加载契约以该仓库 **`creator-docs/`** 为准 |

## 与「整合包 / 一键装模型」等方向的路线（长期）

以下能力在 **oclivenewnew** 主仓库文档 **`creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md`**（体验差异化 backlog）中与愿景一并维护；若本仓库与 `oclivenewnew` **同级克隆**，本地相对路径为 `../oclivenewnew/creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md`。启动器可逐步承接其中「玩家侧上手」部分：

| 方向 | 说明 |
|------|------|
| **整合包 / 离线安装** | 将启动器、Ollama、基础模型、示例角色打成一个安装包，实现「下载即聊」——需安装包工程与许可证策略，与 CI 发版流程配合。 |
| **更智能的依赖** | 在现有检测基础上，可扩展：引导安装 Ollama、拉取推荐模型、低配机策略等（须避免静默破坏用户环境）。 |
| **包 / 插件「商店」** | 浏览与一键安装社区角色包——依赖索引服务、签名与版本契约；**发版与市场同发、启动器入口** 的落地步骤见主仓库 **[MARKET_LAUNCHER_INTEGRATION.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md)**（与 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) 第三节对照）。 |
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

## 贡献与发版

参与开发与 **发版前检查清单** 见 **[CONTRIBUTING.md](CONTRIBUTING.md)**；版本摘要见 **[CHANGELOG.md](CHANGELOG.md)**。

## 许可

MIT（与 oclive 生态其他仓库对齐时可再统一）。
