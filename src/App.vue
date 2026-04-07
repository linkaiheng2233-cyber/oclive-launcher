<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

const VIEW_LABELS: Record<string, string> = {
  announce: '公告栏',
  version: '版本与更新',
  apps: '启动应用',
  assistant: '环境与排障',
  logs: '运行日志',
}
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

interface EnvDiagnostics {
  nodeVersion: string | null
  npmVersion: string | null
  ollamaVersion: string | null
  ollamaApiReachable: boolean
  editorProjectOk: boolean
  editorPackageJson: boolean
  ocliveProjectOk: boolean
  oclivePackageJson: boolean
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
const envDiag = ref<EnvDiagnostics | null>(null)
const envDiagErr = ref('')

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

async function runEnvironmentDiagnose() {
  envDiagErr.value = ''
  try {
    envDiag.value = await invoke<EnvDiagnostics>('diagnose_environment', {
      config: config.value,
    })
    statusMsg.value = '环境检测完成'
  } catch (e) {
    envDiagErr.value = String(e)
    envDiag.value = null
  }
}

async function resetLauncherConfig() {
  if (!confirm('将清空启动器内保存的路径与 GitHub 仓库设置，并恢复默认。是否继续？')) return
  try {
    const c = await invoke<LauncherConfig>('reset_config_to_default')
    config.value = { ...config.value, ...c }
    envDiag.value = null
    statusMsg.value = '已重置为默认配置（若原文件损坏，同目录下可能有 .corrupt.bak 备份）'
    await refreshLocalVersions()
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function openLauncherConfigFolder() {
  try {
    await invoke('open_config_directory')
    statusMsg.value = '已尝试打开配置目录（含 launcher-config.json）'
  } catch (e) {
    statusMsg.value = String(e)
  }
}

function clearLogs() {
  logs.value = []
}

const navItems = [
  { id: 'announce', label: '公告', icon: '📢' },
  { id: 'version', label: '版本', icon: '📦' },
  { id: 'apps', label: '启动', icon: '▶' },
  { id: 'assistant', label: '环境', icon: '🩺' },
  { id: 'logs', label: '日志', icon: '📋' },
] as const

const activeNav = ref<string>('announce')

const currentViewLabel = computed(() => VIEW_LABELS[activeNav.value] ?? '')

function setView(id: string) {
  activeNav.value = id
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
  <div class="fluent-root">
    <aside class="rail" aria-label="区块导航">
      <button
        v-for="item in navItems"
        :key="item.id"
        type="button"
        class="rail-btn"
        :class="{ active: activeNav === item.id }"
        @click="setView(item.id)"
      >
        <span class="rail-ico" aria-hidden="true">{{ item.icon }}</span>
        <span class="rail-lbl">{{ item.label }}</span>
      </button>
    </aside>

    <div class="main-col">
      <header class="titlebar">
        <div class="titlebar-inner">
          <div>
            <p class="kicker">oclive · 工具链</p>
            <h1>{{ currentViewLabel }}</h1>
            <p class="sub">
              <template v-if="activeNav === 'announce'">编辑面向创作者的通知；与版本、启动、日志互不干扰，按需切换左侧栏目。</template>
              <template v-else-if="activeNav === 'version'">对照 GitHub Release 与本地 package.json 版本。</template>
              <template v-else-if="activeNav === 'apps'">配置并启动角色包编写器与 oclive 运行时（无额外终端窗口）。</template>
              <template v-else-if="activeNav === 'assistant'">检测 Node / npm / Ollama 与项目路径，降低上手门槛；复杂问题仍见 README 与上游文档。</template>
              <template v-else>查看子进程输出；筛选编写器或 oclive。</template>
            </p>
          </div>
          <button type="button" class="btn primary" @click="saveConfig">保存配置</button>
        </div>
      </header>

      <nav class="mobile-nav" aria-label="栏目切换">
        <button
          v-for="item in navItems"
          :key="'m-' + item.id"
          type="button"
          class="mobile-nav-btn"
          :class="{ active: activeNav === item.id }"
          @click="setView(item.id)"
        >
          {{ item.icon }} {{ item.label }}
        </button>
      </nav>

      <p v-if="statusMsg" class="status">{{ statusMsg }}</p>

      <div class="scroll-main">
        <section v-if="activeNav === 'announce'" class="view-panel card">
        <h2>公告栏</h2>
        <p class="hint">编辑后点击保存；内容存于本机应用配置目录，便于你写面向创作者的通知。</p>
        <textarea v-model="announcements" class="announce" rows="14" spellcheck="false" />
        <button type="button" class="btn" @click="saveAnnouncements">保存公告</button>
      </section>

      <section v-else-if="activeNav === 'version'" class="view-panel card">
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

    <section v-else-if="activeNav === 'assistant'" class="view-panel card">
        <h2>环境与排障</h2>
        <p class="hint">
          启动前可点「重新检测」：确认本机已安装 <strong>Node.js</strong> / <strong>npm</strong>（开发模式必需），以及
          <strong>Ollama</strong> 是否在运行（oclive 对话默认走本地模型）。若配置文件损坏，可用「一键重置启动器配置」恢复默认，原文件会尽量备份为
          <code>launcher-config.json.corrupt.bak</code>。
        </p>
        <div class="assistant-actions">
          <button type="button" class="btn primary" @click="runEnvironmentDiagnose">重新检测</button>
          <button type="button" class="btn" @click="openLauncherConfigFolder">打开配置目录</button>
          <button type="button" class="btn danger" @click="resetLauncherConfig">一键重置启动器配置</button>
        </div>
        <p v-if="envDiagErr" class="err">{{ envDiagErr }}</p>
        <table v-if="envDiag" class="diag-table">
          <tbody>
            <tr>
              <th>Node</th>
              <td :class="{ ok: !!envDiag.nodeVersion, bad: !envDiag.nodeVersion }">
                {{ envDiag.nodeVersion ?? '未检测到（请安装 Node.js LTS 并加入 PATH）' }}
              </td>
            </tr>
            <tr>
              <th>npm</th>
              <td :class="{ ok: !!envDiag.npmVersion, bad: !envDiag.npmVersion }">
                {{ envDiag.npmVersion ?? '未检测到' }}
              </td>
            </tr>
            <tr>
              <th>Ollama CLI</th>
              <td :class="{ ok: !!envDiag.ollamaVersion, bad: !envDiag.ollamaVersion }">
                {{ envDiag.ollamaVersion ?? '未在 PATH 中找到 ollama（可仍通过服务运行）' }}
              </td>
            </tr>
            <tr>
              <th>Ollama API</th>
              <td :class="{ ok: envDiag.ollamaApiReachable, bad: !envDiag.ollamaApiReachable }">
                {{
                  envDiag.ollamaApiReachable
                    ? '127.0.0.1:11434 可访问'
                    : '不可访问（请启动 Ollama 服务）'
                }}
              </td>
            </tr>
            <tr>
              <th>编写器目录</th>
              <td :class="{ ok: envDiag.editorProjectOk && envDiag.editorPackageJson, bad: !envDiag.editorProjectOk }">
                <template v-if="!config.editorProjectRoot?.trim()">未填写（开发模式需填写项目根）</template>
                <template v-else-if="!envDiag.editorProjectOk">路径不存在或不是文件夹</template>
                <template v-else-if="!envDiag.editorPackageJson">缺少 package.json</template>
                <template v-else>正常</template>
              </td>
            </tr>
            <tr>
              <th>oclive 目录</th>
              <td :class="{ ok: envDiag.ocliveProjectOk && envDiag.oclivePackageJson, bad: !envDiag.ocliveProjectOk }">
                <template v-if="!config.ocliveProjectRoot?.trim()">未填写（开发模式需填写项目根）</template>
                <template v-else-if="!envDiag.ocliveProjectOk">路径不存在或不是文件夹</template>
                <template v-else-if="!envDiag.oclivePackageJson">缺少 package.json</template>
                <template v-else>正常</template>
              </td>
            </tr>
          </tbody>
        </table>
        <p class="hint assistant-links">
          <button type="button" class="linkish" @click="openRelease('https://nodejs.org/')">Node.js 下载</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/download')">Ollama 下载</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://github.com/ollama/ollama')">Ollama 文档</button>
        </p>
      </section>

    <div v-else-if="activeNav === 'apps'" class="view-panel grid grid-2">
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

    <section v-else-if="activeNav === 'logs'" class="view-panel card log-card">
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
    </div>
  </div>
</template>

<style scoped>
.fluent-root {
  display: flex;
  min-height: 100vh;
  font-family: var(--fluent-font);
  color: var(--fluent-text-primary);
  background: var(--fluent-bg-page);
}

.rail {
  width: 76px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.85rem 0.4rem;
  border-right: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-card);
  box-shadow: var(--fluent-shadow-soft);
}

.rail-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.2rem;
  padding: 0.45rem 0.2rem;
  border: none;
  border-radius: var(--fluent-radius-lg);
  background: transparent;
  color: var(--fluent-text-secondary);
  cursor: pointer;
  font-size: 0.65rem;
  font-weight: 500;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.rail-btn:hover {
  background: var(--fluent-bg-subtle);
  color: var(--fluent-text-primary);
}

.rail-btn.active {
  background: var(--fluent-accent-subtle);
  color: var(--fluent-accent);
}

.rail-ico {
  font-size: 1.2rem;
  line-height: 1;
}

.rail-lbl {
  line-height: 1.1;
  text-align: center;
}

.main-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.titlebar {
  flex-shrink: 0;
  padding: 1rem 1.35rem 0;
  border-bottom: 1px solid var(--fluent-border-stroke);
  background: linear-gradient(180deg, var(--fluent-bg-card) 0%, var(--fluent-bg-page) 100%);
}

.titlebar-inner {
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding-bottom: 1rem;
}

.titlebar h1 {
  margin: 0.15rem 0 0;
  font-size: 1.5rem;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.kicker {
  margin: 0;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--fluent-accent);
}

.sub {
  margin: 0.4rem 0 0;
  max-width: 48rem;
  font-size: 0.875rem;
  color: var(--fluent-text-secondary);
  line-height: 1.5;
}

.status {
  flex-shrink: 0;
  max-width: 1100px;
  width: 100%;
  margin: 0 auto;
  padding: 0.35rem 1.35rem 0;
  font-size: 0.8125rem;
  color: var(--fluent-accent);
}

.scroll-main {
  flex: 1;
  overflow-y: auto;
  padding: 1rem 1.35rem 2rem;
}

.scroll-main > .view-panel {
  max-width: 1100px;
  margin-left: auto;
  margin-right: auto;
  width: 100%;
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-bottom: 1rem;
}

.mobile-nav {
  display: none;
}

.mobile-nav-btn {
  padding: 0.35rem 0.55rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-card);
  color: var(--fluent-text-primary);
  font-size: 0.78rem;
  cursor: pointer;
}

.mobile-nav-btn.active {
  border-color: var(--fluent-accent);
  background: var(--fluent-accent-subtle);
  color: var(--fluent-accent);
}

@media (max-width: 900px) {
  .grid {
    grid-template-columns: 1fr;
  }
  .rail {
    display: none;
  }
  .mobile-nav {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    max-width: 1100px;
    width: 100%;
    margin: 0 auto;
    padding: 0 1.35rem 0.5rem;
  }
}

.grid-2 {
  grid-template-columns: 1fr 1fr;
}

@media (max-width: 900px) {
  .grid-2 {
    grid-template-columns: 1fr;
  }
}

.card {
  scroll-margin-top: 0.75rem;
  background: var(--fluent-bg-card);
  border: 1px solid var(--fluent-border-stroke);
  border-radius: var(--fluent-radius-lg);
  padding: 1rem 1.15rem;
  box-shadow: var(--fluent-shadow-card);
}

.card h2 {
  margin: 0 0 0.5rem;
  font-size: 1rem;
  font-weight: 600;
}

.hint {
  margin: 0 0 0.75rem;
  font-size: 0.8125rem;
  color: var(--fluent-text-secondary);
  line-height: 1.45;
}

.announce {
  width: 100%;
  box-sizing: border-box;
  font-family: var(--fluent-mono);
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-control);
  padding: 0.5rem 0.65rem;
  background: var(--fluent-bg-input);
  color: var(--fluent-text-primary);
}

.announce:focus {
  outline: none;
  border-color: var(--fluent-border-focus);
  box-shadow: 0 0 0 1px var(--fluent-border-focus);
}

label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  margin-top: 0.5rem;
  margin-bottom: 0.2rem;
  color: var(--fluent-text-secondary);
}

input[type='text'] {
  width: 100%;
  box-sizing: border-box;
  min-height: 32px;
  padding: 0.35rem 0.5rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-control);
  background: var(--fluent-bg-input);
  color: var(--fluent-text-primary);
  font-size: 0.875rem;
}

input[type='text']:focus {
  outline: none;
  border-color: var(--fluent-border-focus);
  box-shadow: 0 0 0 1px var(--fluent-border-focus);
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
  color: var(--fluent-text-secondary);
}

.sep {
  border: none;
  border-top: 1px solid var(--fluent-border-stroke);
  margin: 0.75rem 0;
}

.mode {
  display: flex;
  gap: 1rem;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.mode label {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  margin: 0;
  font-weight: 500;
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
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.btn {
  padding: 0.45rem 0.9rem;
  min-height: 32px;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-control);
  background: var(--fluent-bg-card);
  color: var(--fluent-text-primary);
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  font-family: var(--fluent-font);
  box-shadow: var(--fluent-shadow-soft);
  transition:
    background 0.15s ease,
    border-color 0.15s ease;
}

.btn:hover {
  background: var(--fluent-bg-subtle);
  border-color: var(--fluent-text-secondary);
}

.btn.primary {
  background: var(--fluent-accent);
  color: #fff;
  border-color: transparent;
  box-shadow: var(--fluent-shadow-soft);
}

.btn.primary:hover {
  background: var(--fluent-accent-hover);
}

.btn.danger {
  background: var(--fluent-danger-bg);
  color: var(--fluent-danger-text);
  border-color: var(--fluent-danger-border);
}

.btn.danger:hover {
  filter: brightness(1.03);
}

.btn.tiny {
  padding: 0.2rem 0.5rem;
  min-height: 0;
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

.log-tools label {
  display: inline;
  margin: 0;
  font-weight: 500;
}

.log-tools select {
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-control);
  padding: 0.3rem 0.45rem;
  background: var(--fluent-bg-input);
  color: var(--fluent-text-primary);
  font-size: 0.875rem;
}

.log {
  margin: 0;
  max-height: 340px;
  overflow: auto;
  padding: 0.65rem 0.85rem;
  background: #1b1b1b;
  color: #e8e8e8;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  font-family: var(--fluent-mono);
  font-size: 0.72rem;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
}

@media (prefers-color-scheme: dark) {
  .log {
    background: #0c0c0c;
    color: #d4d4d4;
  }
}

.err {
  color: var(--fluent-danger-text);
  font-size: 0.85rem;
}

.assistant-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.diag-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
  margin-top: 0.5rem;
}

.diag-table th {
  text-align: left;
  vertical-align: top;
  padding: 0.4rem 0.75rem 0.4rem 0;
  color: var(--fluent-text-secondary);
  font-weight: 600;
  width: 7.5rem;
}

.diag-table td {
  padding: 0.4rem 0;
  line-height: 1.4;
}

.diag-table td.ok {
  color: #107c10;
}

.diag-table td.bad {
  color: var(--fluent-danger-text);
}

.assistant-links {
  margin-top: 0.75rem;
}

.linkish {
  background: none;
  border: none;
  padding: 0;
  color: var(--fluent-accent);
  cursor: pointer;
  font-size: inherit;
  text-decoration: underline;
  font-family: inherit;
}

.linkish:hover {
  color: var(--fluent-accent-hover);
}
</style>
