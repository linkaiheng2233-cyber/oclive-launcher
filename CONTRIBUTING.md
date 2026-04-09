# 参与贡献与发版说明

感谢关注 **oclive-launcher**。本仓库为独立桌面应用（Tauri + Vue），与 **oclivenewnew**、**oclive-pack-editor** 通过本地路径与角色包协作，无源码嵌套。

## 日常开发

- 环境：Node.js LTS、Rust、Windows 上 WebView2（与编写器类似）。
- 安装依赖：`npm install`。
- 开发：`npm run tauri:dev`（Vite `127.0.0.1:5174`）。
- 自检：`npm run build`，`cargo build --manifest-path src-tauri/Cargo.toml`（或与 CI 一致）。

合并前请保证前端构建与 Rust 侧能通过；若改动 Tauri 或子进程逻辑，建议在 Windows 上实际跑一遍「启动 / 环境检测 / 日志」。

## 跨仓约定：随包寄语文件名

若需重命名包内随包寄语文件（默认 **`creator_message.txt`**），须**同步**更新：`src/lib/rolePackCreatorMessage.ts` 中的 **`ROLE_PACK_CREATOR_MESSAGE_FILENAME`**、`src-tauri/src/role_creator_message.rs` 中的路径拼接，以及 **oclive-pack-editor** 内同源常量（见该仓库 [CONTRIBUTING.md](https://github.com/linkaiheng2233-cyber/oclive-pack-editor/blob/main/CONTRIBUTING.md)）。正文长度与「首行」语义以各仓实现为准；权威职责表见根目录 [README.md](README.md) **随包寄语与职责边界**。

## 发版检查清单（维护者）

在创建 **GitHub Release** 或对外分发安装包前，建议按顺序确认：

1. **版本号**  
   - 与 `package.json`、`src-tauri/tauri.conf.json` 里 `package.version` / 展示版本一致（或说明为何仅用 tag 区分）。

2. **构建**  
   - 本地或 CI 执行：`npm run tauri:build`（或 [.github/workflows/release.yml](./.github/workflows/release.yml)：推送 **`v*`** 标签或 **手动运行** workflow，在 Windows 上构建并上传 **`bundle/`** 为 Artifact）。  
   - 产物路径以 Tauri 输出为准（如 `src-tauri/target/release/bundle/` 下各平台安装包）。

3. **CHANGELOG**  
   - 更新根目录 [CHANGELOG.md](./CHANGELOG.md) 对应版本小节（可与 tag 同步）。

4. **附带 Ollama（可选，「胖包」）**  
   - 若本次 Release **内含** 官方 `OllamaSetup.exe`：在构建前放到仓库根目录或 `src-tauri/bundled/ollama/`，再打包（见根目录 [README.md](README.md)「附带 Ollama 安装包」与「发版、Git 与大文件」）。  
   - **勿**将 `OllamaSetup.exe` 提交进 Git；大文件走 **Release 资产** 或 CI 构建时下载。

5. **Release 资产**  
   - 上传构建好的启动器安装包；若另提供「仅启动器」瘦包与「含 Ollama 安装器」说明，在 Release 文案里写清。  
   - 与生态其它仓库资产命名习惯对齐时可参考 README 中的前缀约定（`oclive-launcher-windows-v…` 等）。

6. **说明与合规**  
   - Release 附简短更新说明；若再分发 Ollama 安装包，保留上游版权与许可提示，并可链到 [ollama/ollama/releases](https://github.com/ollama/ollama/releases)。

7. **冒烟**  
   - 在干净或接近用户环境的机器上：安装/解压 → 打开启动器 → 「环境」检测 → 按需验证「运行附带安装包」或 winget / 官网路径。

## 相关文档

- [README.md](README.md)：功能、命令、配置路径、Ollama 与 Git 约定。  
- [src-tauri/bundled/ollama/README.txt](src-tauri/bundled/ollama/README.txt)：附带 `OllamaSetup.exe` 的放置与许可提示。
