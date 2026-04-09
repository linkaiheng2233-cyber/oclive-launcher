# Changelog

本文件随 **Git 标签 / Release** 更新；维护者发版步骤见 [CONTRIBUTING.md](./CONTRIBUTING.md)。

## [Unreleased]

- **界面**：日间 **暖色主题**（象牙/卡其）；顶栏 **界面缩放**（A− / 百分比 / A+，与编写器同档位）；**版本与下载** 等页 **HelpHint** 大白话说明；顶栏 **问号** 解释字号、主题与「保存配置」；若干 **z-index** 修正避免问号被遮挡。
- **文档**：README 功能表补充外观、侧车用户向文档链接（`SIDECAR_LLM_USER_GUIDE.md`）。
- **随包寄语**：包内 `creator_message.txt` 首行由启动器读取；UI 与 Rust 侧统一非空首行解析；README **随包寄语与职责边界** 与公告文件（`announcements.md` / `creator-announcements.md`）区分说明；CONTRIBUTING **跨仓约定**。

## 0.1.1

- **发版与文档**：GitHub Actions `release.yml`（`v*` / 手动触发）；根目录 CHANGELOG / CONTRIBUTING；README 与「附带 Ollama」说明；`scripts/sync-ollama-installer.mjs` 与 `src-tauri/bundled/ollama/README.txt`。

## 0.1.0

### Windows 与 Ollama

- **一键安装**：检测到 `winget` 时可安装 **Ollama.Ollama**（日志频道 `winget`）。
- **附带安装包**：构建前将官方 **`OllamaSetup.exe`** 置于仓库根目录（或 `src-tauri/bundled/ollama/`），`scripts/sync-ollama-installer.mjs` 在 `tauri:dev` / `tauri:build` 时同步；同体积时跳过复制以加快迭代。
- **运行附带安装包**：界面与日志频道 `bundled-ollama`。

### 环境与体验

- **Node / npm** 未检测到时横幅提示；选择 **云端 Remote LLM** 时弱化本机 Ollama 未就绪提示，并说明 zip 安装仍可用本机模型。
- **快捷操作**：一键拉取推荐模型、刷新本机模型列表；日志筛选含 **ollama** / **winget** / **附带安装包**。
- **跨平台**：非 Windows 下开发模式使用 `npm run` 而非 `cmd /C`（与 `bundle.targets: all` 一致）。

### 文档

- [README.md](./README.md) 功能表与 **附带 Ollama**、**发版与 Git** 说明；[CONTRIBUTING.md](./CONTRIBUTING.md) 发版检查清单。
