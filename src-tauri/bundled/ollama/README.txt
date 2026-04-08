附带 Ollama Windows 安装包（与启动器一并分发）
================================================

官方 Windows 安装包在 GitHub Releases 中通常名为 OllamaSetup.exe，就是 Ollama 本体安装程序（不是模型文件）。

推荐：把 OllamaSetup.exe 放在启动器仓库根目录（与 package.json 同级），执行 npm run tauri:dev 或 tauri:build 时会自动复制到本目录后再打包。

也可手动将 OllamaSetup.exe 放到本目录（src-tauri/bundled/ollama/）。

获取方式（请自行核对版本与校验和）：
  https://github.com/ollama/ollama/releases

Ollama 以 MIT 许可发布；再分发时请保留上游版权与许可说明，并建议随包附官方下载页链接。

若本目录仅有本说明而无 OllamaSetup.exe，开发与 CI 仍可正常构建；「运行附带安装包」按钮仅在打包进 exe 后存在该文件时可用。

请勿将 OllamaSetup.exe 提交到 Git：体积大且会拖慢仓库；发版时用 GitHub Releases 或 CI 在构建前下载/复制该文件即可。
