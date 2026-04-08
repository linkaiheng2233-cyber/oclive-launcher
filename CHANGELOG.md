# Changelog

本文件随 **Git 标签 / Release** 更新；维护者发版步骤见 [CONTRIBUTING.md](./CONTRIBUTING.md)。

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
