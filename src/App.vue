<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface LauncherConfig {
  editorProjectRoot: string
  editorExe: string
  editorMode: 'dev' | 'exe'
  editorNpmScript: string
  ocliveProjectRoot: string
  ocliveExe: string
  ocliveMode: 'dev' | 'exe'
  ocliveNpmScript: string
  githubEditorOwner: string
  githubEditorRepo: string
  githubOcliveOwner: string
  githubOcliveRepo: string
}

interface ReleaseInfo {
  tagName: string
  name?: string
  htmlUrl: string
  publishedAt?: string
  body?: string
}

interface LogLine {
  app: string
  stream: string
  line: string
  ts: number
}

const config = ref<LauncherConfig>({
  editorProjectRoot: '',
  editorExe: '',
  editorMode: 'dev',
  editorNpmScript: 'tauri:dev',
  ocliveProjectRoot: '',
  ocliveExe: '',
  ocliveMode: 'dev',
  ocliveNpmScript: 'tauri:dev',
  githubEditorOwner: '',
  githubEditorRepo: '',
  githubOcliveOwner: '',
  githubOcliveRepo: '',
})

const announcements = ref('')
const statusMsg = ref('')
const logFilter = ref<'all' | 'editor' | 'oclive'>('all')
const logs = ref<LogLine[]>([])
const MAX_LOG = 4000

const editorLocalVer = ref<string | null>(null)
const ocliveLocalVer = ref<string | null>(null)
const editorRemote = ref<ReleaseInfo | null>(null)
const ocliveRemote = ref<ReleaseInfo | null>(null)
const checkErr = ref('')

let unlistenLog: UnlistenFn | undefined
let unlistenExit: UnlistenFn | undefined

const filteredLogs = computed(() => {
  const f = logFilter.value
  if (f === 'all') return logs.value
  return logs.value.filter((l) => l.app === f)
})

function pushLog(app: string, stream: string, line: string) {
  logs.value.push({ app, stream, line, ts: Date.now() })
  if (logs.value.length > MAX_LOG) {
    logs.value.splice(0, logs.value.length - MAX_LOG)
  }
}

async function refreshLocalVersions() {
  try {
    if (config.value.editorProjectRoot.trim()) {
      editorLocalVer.value = await invoke<string | null>('read_package_version', {
        projectRoot: config.value.editorProjectRoot.trim(),
      })
    } else editorLocalVer.value = null
    if (config.value.ocliveProjectRoot.trim()) {
      ocliveLocalVer.value = await invoke<string | null>('read_package_version', {
        projectRoot: config.value.ocliveProjectRoot.trim(),
      })
    } else ocliveLocalVer.value = null
  } catch {
    editorLocalVer.value = null
    ocliveLocalVer.value = null
  }
}

async function loadAll() {
  try {
    const c = await invoke<LauncherConfig>('load_config')
    config.value = { ...config.value, ...c }
    announcements.value = await invoke<string>('load_announcements')
    await refreshLocalVersions()
    statusMsg.value = '已加载配置'
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function saveConfig() {
  try {
    await invoke('save_config', { config: config.value })
    statusMsg.value = '配置已保存'
    await refreshLocalVersions()
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function saveAnnouncements() {
  try {
    await invoke('save_announcements', { text: announcements.value })
    statusMsg.value = '公告已保存'
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function pickEditorRoot() {
  const p = await invoke<string | undefined>('pick_folder')
  if (p) config.value.editorProjectRoot = p
}
async function pickOcliveRoot() {
  const p = await invoke<string | undefined>('pick_folder')
  if (p) config.value.ocliveProjectRoot = p
}
async function pickEditorExe() {
  const p = await invoke<string | undefined>('pick_exe')
  if (p) config.value.editorExe = p
}
async function pickOcliveExe() {
  const p = await invoke<string | undefined>('pick_exe')
  if (p) config.value.ocliveExe = p
}

async function spawnEditor() {
  try {
    await invoke('spawn_managed_app', { kind: 'editor', config: config.value })
  } catch (e) {
    statusMsg.value = String(e)
  }
}
async function spawnOclive() {
  try {
    await invoke('spawn_managed_app', { kind: 'oclive', config: config.value })
  } catch (e) {
    statusMsg.value = String(e)
  }
}
async function stopEditor() {
  try {
    await invoke('stop_managed_app', { kind: 'editor' })
  } catch (e) {
    statusMsg.value = String(e)
  }
}
async function stopOclive() {
  try {
    await invoke('stop_managed_app', { kind: 'oclive' })
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function checkReleases() {
  checkErr.value = ''
  editorRemote.value = null
  ocliveRemote.value = null
  try {
    if (config.value.githubEditorOwner && config.value.githubEditorRepo) {
      editorRemote.value = await invoke<ReleaseInfo>('fetch_github_release', {
        owner: config.value.githubEditorOwner,
        repo: config.value.githubEditorRepo,
      })
    }
    if (config.value.githubOcliveOwner && config.value.githubOcliveRepo) {
      ocliveRemote.value = await invoke<ReleaseInfo>('fetch_github_release', {
        owner: config.value.githubOcliveOwner,
        repo: config.value.githubOcliveRepo,
      })
    }
    await refreshLocalVersions()
    statusMsg.value = '已检查远端版本'
  } catch (e) {
    checkErr.value = String(e)
  }
}

async function openRelease(url: string) {
  try {
    await invoke('open_url', { url })
  } catch (e) {
    statusMsg.value = String(e)
  }
}

function clearLogs() {
  logs.value = []
}

onMounted(async () => {
  await loadAll()
  unlistenLog = await listen<{ app: string; stream: string; line: string }>(
    'launcher-log',
    (e) => {
      const p = e.payload
      pushLog(p.app, p.stream, p.line)
    },
  )
  unlistenExit = await listen<{ app: string; code: number | null }>('launcher-exit', (e) => {
    const p = e.payload
    pushLog(p.app, 'out', `--- 进程已结束 (exit code: ${p.code ?? '?'}) ---`)
  })
})

onUnmounted(() => {
  unlistenLog?.()
  unlistenExit?.()
})
</script>

<template>
  <div class="shell">
    <header class="top">
      <div>
        <h1>oclive 启动器</h1>
        <p class="sub">
          下载与版本信息、公告、启动编写器与 oclive；子进程日志集中在下方，不再弹出多个终端窗口。
        </p>
      </div>
      <div class="top-actions">
        <button type="button" class="btn primary" @click="saveConfig">保存配置</button>
      </div>
    </header>

    <p v-if="statusMsg" class="status">{{ statusMsg }}</p>

    <div class="grid">
      <section class="card">
        <h2>公告栏</h2>
        <p class="hint">编辑后点击保存；内容存于本机应用配置目录，便于你写面向创作者的通知。</p>
        <textarea v-model="announcements" class="announce" rows="12" spellcheck="false" />
        <button type="button" class="btn" @click="saveAnnouncements">保存公告</button>
      </section>

      <section class="card">
        <h2>版本与更新</h2>
        <p class="hint">
          填写 GitHub <code>owner</code> / <code>repo</code> 后点「检查更新」，拉取最新 Release
         （需仓库有公开 Release 或你有权访问）。
        </p>

        <div class="gh-row">
          <label>编写器仓库</label>
          <div class="gh-inputs">
            <input v-model="config.githubEditorOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubEditorRepo" placeholder="repo" />
          </div>
        </div>
        <div class="ver-line">
          <span>本地（package.json）</span>
          <strong>{{ editorLocalVer ?? '—' }}</strong>
        </div>
        <div class="ver-line" v-if="editorRemote">
          <span>远端最新</span>
          <strong>{{ editorRemote.tagName }}</strong>
          <button type="button" class="btn tiny" @click="openRelease(editorRemote.htmlUrl)">
            打开发布页
          </button>
        </div>

        <hr class="sep" />

        <div class="gh-row">
          <label>oclive 仓库</label>
          <div class="gh-inputs">
            <input v-model="config.githubOcliveOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubOcliveRepo" placeholder="repo" />
          </div>
        </div>
        <div class="ver-line">
          <span>本地（package.json）</span>
          <strong>{{ ocliveLocalVer ?? '—' }}</strong>
        </div>
        <div class="ver-line" v-if="ocliveRemote">
          <span>远端最新</span>
          <strong>{{ ocliveRemote.tagName }}</strong>
          <button type="button" class="btn tiny" @click="openRelease(ocliveRemote.htmlUrl)">
            打开发布页
          </button>
        </div>

        <p v-if="checkErr" class="err">{{ checkErr }}</p>
        <button type="button" class="btn primary" @click="checkReleases">检查更新</button>
      </section>
    </div>

    <div class="grid grid-2">
      <section class="card">
        <h2>角色包编写器</h2>
        <div class="mode">
          <label><input v-model="config.editorMode" type="radio" value="dev" /> 开发（npm）</label>
          <label><input v-model="config.editorMode" type="radio" value="exe" /> 已安装 exe</label>
        </div>
        <template v-if="config.editorMode === 'dev'">
          <label>项目根目录</label>
          <div class="row">
            <input v-model="config.editorProjectRoot" placeholder="例如 D:\oclive-pack-editor" />
            <button type="button" class="btn" @click="pickEditorRoot">浏览…</button>
          </div>
          <label>npm 脚本名</label>
          <input v-model="config.editorNpmScript" placeholder="tauri:dev" />
        </template>
        <template v-else>
          <label>可执行文件</label>
          <div class="row">
            <input v-model="config.editorExe" placeholder=".exe 路径" />
            <button type="button" class="btn" @click="pickEditorExe">浏览…</button>
          </div>
        </template>
        <div class="actions">
          <button type="button" class="btn primary" @click="spawnEditor">启动</button>
          <button type="button" class="btn danger" @click="stopEditor">停止</button>
        </div>
      </section>

      <section class="card">
        <h2>oclive 运行时</h2>
        <div class="mode">
          <label><input v-model="config.ocliveMode" type="radio" value="dev" /> 开发（npm）</label>
          <label><input v-model="config.ocliveMode" type="radio" value="exe" /> 已安装 exe</label>
        </div>
        <template v-if="config.ocliveMode === 'dev'">
          <label>项目根目录</label>
          <div class="row">
            <input v-model="config.ocliveProjectRoot" placeholder="例如 D:\oclivenewnew" />
            <button type="button" class="btn" @click="pickOcliveRoot">浏览…</button>
          </div>
          <label>npm 脚本名</label>
          <input v-model="config.ocliveNpmScript" placeholder="tauri:dev" />
        </template>
        <template v-else>
          <label>可执行文件</label>
          <div class="row">
            <input v-model="config.ocliveExe" placeholder=".exe 路径" />
            <button type="button" class="btn" @click="pickOcliveExe">浏览…</button>
          </div>
        </template>
        <div class="actions">
          <button type="button" class="btn primary" @click="spawnOclive">启动</button>
          <button type="button" class="btn danger" @click="stopOclive">停止</button>
        </div>
      </section>
    </div>

    <section class="card log-card">
      <div class="log-head">
        <h2>运行日志（子进程 stdout / stderr）</h2>
        <div class="log-tools">
          <label>筛选</label>
          <select v-model="logFilter">
            <option value="all">全部</option>
            <option value="editor">仅编写器</option>
            <option value="oclive">仅 oclive</option>
          </select>
          <button type="button" class="btn" @click="clearLogs">清空</button>
        </div>
      </div>
      <pre class="log">{{ filteredLogs.map((l) => `[${l.app}][${l.stream}] ${l.line}`).join('\n') }}</pre>
    </section>
  </div>
</template>

<style scoped>
.shell {
  max-width: 1100px;
  margin: 0 auto;
  padding: 1rem 1.25rem 2rem;
  font-family:
    'Segoe UI',
    system-ui,
    sans-serif;
  color: var(--text);
  background: var(--bg);
  min-height: 100vh;
}

.top {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.5rem;
}

.top h1 {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 600;
}

.sub {
  margin: 0.35rem 0 0;
  font-size: 0.88rem;
  color: var(--muted);
  max-width: 52rem;
}

.status {
  margin: 0 0 0.75rem;
  font-size: 0.85rem;
  color: var(--accent);
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-bottom: 1rem;
}

@media (max-width: 900px) {
  .grid {
    grid-template-columns: 1fr;
  }
}

.grid-2 {
  grid-template-columns: 1fr 1fr;
}

.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem 1.1rem;
  box-shadow: var(--shadow);
}

.card h2 {
  margin: 0 0 0.5rem;
  font-size: 1rem;
}

.hint {
  margin: 0 0 0.75rem;
  font-size: 0.8rem;
  color: var(--muted);
  line-height: 1.45;
}

.announce {
  width: 100%;
  box-sizing: border-box;
  font-family: ui-monospace, monospace;
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  padding: 0.5rem 0.6rem;
  background: var(--input-bg);
  color: var(--text);
}

label {
  display: block;
  font-size: 0.8rem;
  margin-top: 0.5rem;
  margin-bottom: 0.2rem;
  color: var(--muted);
}

input[type='text'] {
  width: 100%;
  box-sizing: border-box;
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--input-bg);
  color: var(--text);
}

.gh-row {
  margin-bottom: 0.5rem;
}

.gh-inputs {
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

.gh-inputs input {
  flex: 1;
}

.ver-line {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  margin: 0.25rem 0;
}

.ver-line span {
  color: var(--muted);
}

.sep {
  border: none;
  border-top: 1px solid var(--border);
  margin: 0.75rem 0;
}

.mode {
  display: flex;
  gap: 1rem;
  margin-bottom: 0.5rem;
  font-size: 0.88rem;
}

.mode label {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  margin: 0;
}

.row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.35rem;
}

.row input {
  flex: 1;
}

.actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.btn {
  padding: 0.35rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--btn-bg);
  color: var(--text);
  cursor: pointer;
  font-size: 0.88rem;
}

.btn:hover {
  filter: brightness(1.05);
}

.btn.primary {
  background: var(--accent);
  color: #fff;
  border-color: transparent;
}

.btn.danger {
  background: var(--danger-bg);
  color: var(--danger-text);
  border-color: transparent;
}

.btn.tiny {
  padding: 0.15rem 0.45rem;
  font-size: 0.78rem;
}

.log-card {
  margin-top: 0.25rem;
}

.log-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.log-tools {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
}

.log-tools select {
  border-radius: 6px;
  border: 1px solid var(--border);
  padding: 0.25rem 0.4rem;
  background: var(--input-bg);
  color: var(--text);
}

.log {
  margin: 0;
  max-height: 320px;
  overflow: auto;
  padding: 0.6rem 0.75rem;
  background: #0f1419;
  color: #d8dee9;
  border-radius: 6px;
  font-size: 0.72rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
}

.err {
  color: var(--danger-text);
  font-size: 0.85rem;
}
</style>
