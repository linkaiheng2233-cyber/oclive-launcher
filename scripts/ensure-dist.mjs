import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..')
const dist = path.join(root, 'dist')
const index = path.join(dist, 'index.html')

fs.mkdirSync(dist, { recursive: true })
if (!fs.existsSync(index)) {
  fs.writeFileSync(
    index,
    '<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"/><title>placeholder</title></head><body><p>请先执行 <code>npm run build</code> 生成前端；开发请用 <code>npm run tauri:dev</code>。</p></body></html>\n',
    'utf8',
  )
}
