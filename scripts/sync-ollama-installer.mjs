/**
 * 若仓库根目录（与 package.json 同级）存在官方 OllamaSetup.exe，
 * 复制到 src-tauri/bundled/ollama/，供 Tauri bundle.resources 打进安装包。
 * 根目录无该文件时静默退出（不报错）。
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.join(__dirname, '..')
const src = path.join(root, 'OllamaSetup.exe')
const destDir = path.join(root, 'src-tauri', 'bundled', 'ollama')
const dest = path.join(destDir, 'OllamaSetup.exe')

if (!fs.existsSync(src)) {
  process.exit(0)
}
fs.mkdirSync(destDir, { recursive: true })
if (fs.existsSync(dest)) {
  const ss = fs.statSync(src)
  const ds = fs.statSync(dest)
  if (ss.size === ds.size) {
    process.exit(0)
  }
}
try {
  fs.copyFileSync(src, dest)
} catch (e) {
  console.error('[oclive-launcher] 复制 OllamaSetup.exe 失败:', e instanceof Error ? e.message : e)
  process.exit(1)
}
console.log('[oclive-launcher] 已同步 OllamaSetup.exe → src-tauri/bundled/ollama/')
