<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

const VIEW_LABELS: Record<string, string> = {
  start: '第一次使用',
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
  /** 启动 oclive 时注入 OCLIVE_ROLES_DIR */
  ocliveRolesDir: string
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
  ocliveRolesDirOk: boolean
  ocliveRolesDirHasRoleHint: boolean
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
  githubEditorOwner: 'linkaiheng2233-cyber',
  githubEditorRepo: 'oclive-pack-editor',
  githubOcliveOwner: 'linkaiheng2233-cyber',
  githubOcliveRepo: 'oclivenewnew',
  ocliveRolesDir: '',
})

const announcements = ref('')
const statusMsg = ref('')
const logFilter = ref<'all' | 'editor' | 'oclive' | 'ollama'>('all')
const logs = ref<LogLine[]>([])
const MAX_LOG = 4000

const editorLocalVer = ref<string | null>(null)
const ocliveLocalVer = ref<string | null>(null)
const editorRemote = ref<ReleaseInfo | null>(null)
const ocliveRemote = ref<ReleaseInfo | null>(null)
const checkErr = ref('')
const envDiag = ref<EnvDiagnostics | null>(null)
const envDiagErr = ref('')

/** 与运行时 / 编写器默认一致 */
const DEFAULT_OLLAMA_MODEL = 'qwen2.5:7b'
const MODEL_OPTION_CUSTOM = '__custom__'

const installModalOpen = ref(false)
const pendingZipPath = ref<string | null>(null)
const installModelSelect = ref(DEFAULT_OLLAMA_MODEL)
const installCustomModel = ref('')
const installOverwriteModel = ref(false)
const ollamaLocalModels = ref<string[]>([])
const installBusy = ref(false)
const pullBusy = ref(false)

const installModelOptions = computed(() => {
  const opts: { value: string; label: string }[] = [
    { value: DEFAULT_OLLAMA_MODEL, label: `${DEFAULT_OLLAMA_MODEL}（推荐默认）` },
  ]
  const fromHost = ollamaLocalModels.value.filter((m) => m !== DEFAULT_OLLAMA_MODEL)
  const seen = new Set(opts.map((o) => o.value))
  for (const m of fromHost) {
    if (seen.has(m)) continue
    seen.add(m)
    opts.push({ value: m, label: m })
  }
  opts.push({ value: MODEL_OPTION_CUSTOM, label: '手动输入模型名…' })
  return opts
})

const effectiveInstallModel = computed(() => {
  if (installModelSelect.value === MODEL_OPTION_CUSTOM) {
    return installCustomModel.value.trim()
  }
  return installModelSelect.value.trim()
})

/** 未检测到 Ollama CLI 或 API 不可达时，提示安装 */
const ollamaNeedsAttention = computed(() => {
  const d = envDiag.value
  if (!d) return false
  return !d.ollamaApiReachable || !d.ollamaVersion
})

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

async function pickRolesRoot() {
  const p = await invoke<string | undefined>('pick_folder')
  if (p) config.value.ocliveRolesDir = p
}

async function fillSuggestedRolesDir() {
  try {
    const r = await invoke<string | null>('suggest_roles_dir_from_oclive_root', {
      ocliveProjectRoot: config.value.ocliveProjectRoot.trim(),
    })
    if (r) {
      config.value.ocliveRolesDir = r
      statusMsg.value = '已填入 oclive 仓库下的 roles 目录'
    } else {
      statusMsg.value =
        '未找到：请确认「oclive 项目根」正确，且其下存在 roles 文件夹（若尚未克隆仓库可先手动选择目录）'
    }
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function beginInstallRolePack() {
  try {
    const zipPath = await invoke<string | undefined>('pick_role_pack_zip')
    if (!zipPath) return
    if (!config.value.ocliveRolesDir?.trim()) {
      statusMsg.value = '请先在下方填写「角色包根目录」'
      return
    }
    pendingZipPath.value = zipPath
    installModelSelect.value = DEFAULT_OLLAMA_MODEL
    installCustomModel.value = ''
    installOverwriteModel.value = false
    try {
      ollamaLocalModels.value = await invoke<string[]>('ollama_list_local_models')
    } catch {
      ollamaLocalModels.value = []
      statusMsg.value =
        '未能列出本机已拉取的模型（Ollama 未启动？）。仍可手动输入模型名并完成安装。'
    }
    installModalOpen.value = true
  } catch (e) {
    statusMsg.value = String(e)
  }
}

function cancelInstallRolePackModal() {
  installModalOpen.value = false
  pendingZipPath.value = null
}

async function confirmInstallRolePack() {
  const model = effectiveInstallModel.value
  if (!model) {
    statusMsg.value = '请选择或输入 Ollama 模型名'
    return
  }
  const zip = pendingZipPath.value
  const root = config.value.ocliveRolesDir?.trim()
  if (!zip || !root) {
    statusMsg.value = '缺少 zip 或角色包根目录'
    return
  }
  installBusy.value = true
  try {
    const roleId = await invoke<string>('install_role_pack_zip', {
      zipPath: zip,
      rolesRoot: root,
      model,
      overwriteSettingsModel: installOverwriteModel.value,
    })
    statusMsg.value = `已安装角色「${roleId}」到角色包目录，并已按选项写入 settings.json 的 model。`
    installModalOpen.value = false
    pendingZipPath.value = null
    await runEnvironmentDiagnose({ quiet: true })
  } catch (e) {
    statusMsg.value = String(e)
  } finally {
    installBusy.value = false
  }
}

async function pullSelectedOllamaModel() {
  const model = effectiveInstallModel.value
  if (!model) {
    statusMsg.value = '请选择或输入要拉取的模型名'
    return
  }
  pullBusy.value = true
  try {
    await invoke('ollama_pull_model', { model })
    statusMsg.value = `已开始拉取「${model}」，进度见「日志」→ 筛选 ollama。完成后可点「刷新本机列表」。`
    try {
      activeNav.value = 'logs'
      logFilter.value = 'ollama'
    } catch {
      /* ignore */
    }
  } catch (e) {
    statusMsg.value = String(e)
  } finally {
    pullBusy.value = false
  }
}

async function refreshOllamaLocalModelsList() {
  try {
    ollamaLocalModels.value = await invoke<string[]>('ollama_list_local_models')
    statusMsg.value = '已刷新本机 Ollama 模型列表'
  } catch (e) {
    statusMsg.value = String(e)
  }
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

async function runEnvironmentDiagnose(opts?: { quiet?: boolean }) {
  envDiagErr.value = ''
  const quiet = opts?.quiet === true
  try {
    envDiag.value = await invoke<EnvDiagnostics>('diagnose_environment', {
      config: config.value,
    })
    if (!quiet) statusMsg.value = '环境检测完成'
  } catch (e) {
    envDiagErr.value = String(e)
    envDiag.value = null
    if (!quiet) statusMsg.value = String(e)
  }
}

const STORAGE_FIRST_AUTO_ENV = 'oclive-launcher-first-auto-env-v1'

async function maybeFirstLaunchAutoDiagnose() {
  try {
    if (localStorage.getItem(STORAGE_FIRST_AUTO_ENV)) return
    await runEnvironmentDiagnose({ quiet: true })
    localStorage.setItem(STORAGE_FIRST_AUTO_ENV, '1')
    statusMsg.value =
      '欢迎！已为你自动检测环境，详见「环境」页。需要时可再点「重新检测」。'
  } catch {
    try {
      await runEnvironmentDiagnose({ quiet: true })
    } catch {
      /* ignore */
    }
  }
}

const releasesEditorUrl = computed(() => {
  const o = config.value.githubEditorOwner.trim()
  const r = config.value.githubEditorRepo.trim()
  if (!o || !r) return 'https://github.com/linkaiheng2233-cyber/oclive-pack-editor/releases'
  return `https://github.com/${o}/${r}/releases`
})

const releasesOcliveUrl = computed(() => {
  const o = config.value.githubOcliveOwner.trim()
  const r = config.value.githubOcliveRepo.trim()
  if (!o || !r) return 'https://github.com/linkaiheng2233-cyber/oclivenewnew/releases'
  return `https://github.com/${o}/${r}/releases`
})

async function resetLauncherConfig() {
  if (
    !confirm(
      '将清空启动器内保存的路径，并恢复默认（含上游 GitHub owner/repo 占位）。是否继续？',
    )
  )
    return
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
  { id: 'start', label: '上手', icon: '🚀' },
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
  await maybeFirstLaunchAutoDiagnose()
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
              <template v-if="activeNav === 'start'">不管你是想先聊天、先做角色包，还是两样都试：下面是一条最常走的路，你也可以按自己的节奏跳步。</template>
              <template v-else-if="activeNav === 'announce'">编辑面向创作者的通知；与版本、启动、日志互不干扰，按需切换左侧栏目。</template>
              <template v-else-if="activeNav === 'version'">对照 GitHub Release 与本地 package.json 版本。</template>
              <template v-else-if="activeNav === 'apps'">配置并启动角色包编写器与 oclive 运行时（无额外终端窗口）。</template>
              <template v-else-if="activeNav === 'assistant'">检测 Node / npm / Ollama 与项目路径，降低上手门槛；复杂问题仍见 README 与上游文档。</template>
              <template v-else>查看子进程输出；可筛选编写器、oclive 或 ollama pull。</template>
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
        <section v-if="activeNav === 'start'" class="view-panel card guide-card">
        <h2>欢迎来到 oclive 工具链</h2>
        <p class="hint guide-lead">
          你完全可以只做玩家、只做创作者，或两样都试——没有固定顺序。下面用最少步骤把<strong>编写器 → 角色包 → 运行时</strong>串起来；细节随时可在「版本」「环境」里补全。
        </p>
        <p class="hint">
          三个仓库各管一摊：<strong>运行时</strong>（oclivenewnew）、<strong>角色包编写器</strong>（oclive-pack-editor）、<strong>启动器</strong>（本应用）。角色内容落在磁盘上的
          <code>roles/</code> 根（<code>OCLIVE_ROLES_DIR</code>），由启动器在启动 oclive 时可选注入。
        </p>
        <ol class="guide-steps">
          <li>
            <strong>环境</strong>：安装 <strong>Node.js LTS</strong>；若要本地对话再装 <strong>Ollama</strong>。首次打开本启动器时会<strong>自动检测一次</strong>环境，也可随时在「环境」页点「重新检测」。
          </li>
          <li>
            <strong>获取软件</strong>：「版本」页可打开<strong>编写器 / 运行时</strong>的 GitHub <strong>Releases</strong> 下载安装包；或克隆仓库本地开发（同级目录最省事）。
          </li>
          <li>
            <strong>配置启动器</strong>：在「启动」页填写编写器与 oclive 的<strong>项目根</strong>或 <strong>exe</strong>；在 oclive 卡片填写<strong>角色包根目录</strong>（可用「从 oclive 仓库填入」指向仓库内 <code>roles/</code>）。
          </li>
          <li>
            <strong>角色包</strong>：用编写器导出 zip 并解压到 <code>OCLIVE_ROLES_DIR/某角色id/</code>，或使用编写器「写入文件夹」。
          </li>
          <li>
            <strong>开聊</strong>：启动 oclive，在应用内选角并开始对话。若已装 Ollama，请记得按需 <code>ollama pull</code> 拉取模型（见「环境」页说明）。
          </li>
        </ol>
        <p class="hint guide-links">
          <button type="button" class="linkish" @click="openRelease('https://nodejs.org/')">Node.js</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/download')">Ollama 下载</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/library')">Ollama 模型库</button>
          ·
          <button type="button" class="linkish" @click="openRelease(releasesEditorUrl)">编写器 Releases</button>
          ·
          <button type="button" class="linkish" @click="openRelease(releasesOcliveUrl)">运行时 Releases</button>
        </p>
      </section>

        <section v-else-if="activeNav === 'announce'" class="view-panel card">
        <h2>公告栏</h2>
        <p class="hint">编辑后点击保存；内容存于本机应用配置目录，便于你写面向创作者的通知。</p>
        <textarea v-model="announcements" class="announce" rows="14" spellcheck="false" />
        <button type="button" class="btn" @click="saveAnnouncements">保存公告</button>
      </section>

      <section v-else-if="activeNav === 'version'" class="view-panel card">
        <h2>版本与更新</h2>
        <p class="hint">
          默认已填入<strong>上游</strong> GitHub 仓库名，可直接「检查更新」或打开 Release 下载安装包；若你使用自己的 fork，请改成你的 <code>owner/repo</code>。
        </p>
        <div class="ver-quick-dl">
          <span class="ver-quick-label">快速下载（跳转各仓库 Releases 页）</span>
          <div class="ver-quick-btns">
            <button type="button" class="btn" @click="openRelease(releasesEditorUrl)">编写器 Releases</button>
            <button type="button" class="btn" @click="openRelease(releasesOcliveUrl)">oclive 运行时 Releases</button>
          </div>
        </div>
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
        <div v-if="envDiag && ollamaNeedsAttention" class="banner-warn" role="status">
          <strong>Ollama 未就绪</strong>：未检测到 CLI 或 <code>127.0.0.1:11434</code> 不可访问。对话需要本地模型时，请先安装并启动 Ollama。
          <button type="button" class="btn tiny" @click="openRelease('https://ollama.com/download')">打开 Ollama 下载页</button>
        </div>
        <div class="ollama-model-box">
          <strong>Ollama 与模型</strong>
          <ul>
            <li>
              <strong>安装</strong>：从
              <button type="button" class="linkish inline" @click="openRelease('https://ollama.com/download')">ollama.com/download</button>
              获取安装包；安装后尽量让 Ollama 在后台运行（系统托盘）。
            </li>
            <li>
              <strong>拉取模型</strong>：安装不等于已有模型。可在「启动」页用 <strong>从 zip 安装角色包</strong> 对话框里的「拉取所选模型」，或在终端执行
              <code>ollama pull &lt;模型名&gt;</code>；库见
              <button type="button" class="linkish inline" @click="openRelease('https://ollama.com/library')">Ollama 模型库</button>。生态默认推荐 <code>qwen2.5:7b</code>。
            </li>
            <li>
              <strong>云端模型</strong>：<code>ollama pull</code> 只拉取<strong>本机 Ollama</strong> 可用的模型，不能用来「下载」OpenAI / 国内云 API。要在 oclive 里用云端大脑，请在角色包
              <code>settings.json</code> 将 <code>plugin_backends.llm</code> 设为 <code>remote</code>，并在运行 oclive 的环境中设置
              <code>OCLIVE_REMOTE_LLM_URL</code>（及可选 Token），指向实现 JSON-RPC 的侧车或网关；详见 oclivenewnew 仓库
              <code>creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md</code>。
            </li>
            <li>
              <strong>网络</strong>：下载安装包或 pull 若较慢，可稍后重试；具体以 Ollama 官方文档为准。
            </li>
          </ul>
        </div>
        <p class="hint">
          启动前可点「重新检测」：确认本机已安装 <strong>Node.js</strong> / <strong>npm</strong>（开发模式必需），以及
          <strong>Ollama</strong> 是否在运行（oclive 对话默认走本地模型）。若配置文件损坏，可用「一键重置启动器配置」恢复默认，原文件会尽量备份为
          <code>launcher-config.json.corrupt.bak</code>。
        </p>
        <div class="assistant-actions">
          <button type="button" class="btn primary" @click="() => runEnvironmentDiagnose()">重新检测</button>
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
            <tr>
              <th>角色包根目录</th>
              <td
                :class="{
                  ok: envDiag.ocliveRolesDirOk,
                  bad: !!config.ocliveRolesDir?.trim() && !envDiag.ocliveRolesDirOk,
                }"
              >
                <template v-if="!config.ocliveRolesDir?.trim()">未填写（可选；填写后启动 oclive 会注入 OCLIVE_ROLES_DIR）</template>
                <template v-else-if="!envDiag.ocliveRolesDirOk">路径不存在或不是文件夹</template>
                <template v-else-if="!envDiag.ocliveRolesDirHasRoleHint">目录有效，尚未检测到子目录下的 manifest.json（可稍后放入角色包）</template>
                <template v-else>正常（已检测到角色包目录）</template>
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
        <div class="roles-block">
          <label>角色包根目录（<code>OCLIVE_ROLES_DIR</code>）</label>
          <p class="hint tiny">
            指向「各 <code>角色id/</code> 的父目录」，内含 <code>mumu/manifest.json</code> 这类结构。启动 oclive 时由启动器注入环境变量；留空则不在此注入。
          </p>
          <div class="row">
            <input v-model="config.ocliveRolesDir" placeholder="例如 D:\oclivenewnew\roles" />
            <button type="button" class="btn" @click="pickRolesRoot">浏览…</button>
            <button type="button" class="btn" @click="fillSuggestedRolesDir">从 oclive 仓库填入</button>
          </div>
          <p class="hint tiny">从编写器导出的 <code>.zip</code> / <code>.ocpak</code> 可安装到上述目录。</p>
          <div class="row role-install-row">
            <button type="button" class="btn primary" @click="beginInstallRolePack">从 zip 安装角色包…</button>
          </div>
        </div>
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
            <option value="ollama">仅 ollama pull</option>
          </select>
          <button type="button" class="btn" @click="clearLogs">清空</button>
        </div>
      </div>
      <pre class="log">{{ filteredLogs.map((l) => `[${l.app}][${l.stream}] ${l.line}`).join('\n') }}</pre>
    </section>
      </div>

      <div
        v-if="installModalOpen"
        class="install-modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="install-modal-title"
        @click.self="cancelInstallRolePackModal"
      >
        <div class="install-modal-panel card" @click.stop>
          <h2 id="install-modal-title">安装角色包：选择大脑（Ollama 模型）</h2>
          <p v-if="pendingZipPath" class="hint tiny modal-zip-path">
            {{ pendingZipPath }}
          </p>
          <label class="modal-label">使用哪个模型？</label>
          <p class="hint tiny">
            默认推荐 <code>{{ DEFAULT_OLLAMA_MODEL }}</code>；下列为当前本机已拉取的模型（来自
            <code>ollama list</code> / API）。云端 API 不走 Ollama，见环境页说明。
          </p>
          <select v-model="installModelSelect" class="modal-select">
            <option v-for="opt in installModelOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
          <div v-if="installModelSelect === MODEL_OPTION_CUSTOM" class="modal-custom">
            <label>自定义模型名（与 <code>ollama pull</code> 一致）</label>
            <input v-model="installCustomModel" type="text" placeholder="例如 llama3.2:latest" />
          </div>
          <label class="modal-check">
            <input v-model="installOverwriteModel" type="checkbox" />
            若 <code>settings.json</code> 里已有 <code>model</code>，仍覆盖为所选模型
          </label>
          <div class="modal-actions">
            <button type="button" class="btn" :disabled="pullBusy" @click="refreshOllamaLocalModelsList">
              刷新本机列表
            </button>
            <button type="button" class="btn" :disabled="pullBusy || installBusy" @click="pullSelectedOllamaModel">
              拉取所选模型（ollama pull）
            </button>
            <button type="button" class="btn" @click="cancelInstallRolePackModal">取消</button>
            <button
              type="button"
              class="btn primary"
              :disabled="installBusy || pullBusy"
              @click="confirmInstallRolePack"
            >
              解压并写入
            </button>
          </div>
        </div>
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

.guide-card .guide-steps {
  margin: 0.75rem 0 0;
  padding-left: 1.25rem;
  line-height: 1.65;
  font-size: 0.9rem;
  color: var(--fluent-text-primary);
}

.guide-card .guide-steps li {
  margin-bottom: 0.5rem;
}

.guide-links {
  margin-top: 0.75rem;
}

.banner-warn {
  margin-bottom: 0.75rem;
  padding: 0.6rem 0.75rem;
  border-radius: var(--fluent-radius);
  border: 1px solid rgba(200, 120, 0, 0.45);
  background: rgba(255, 170, 0, 0.12);
  font-size: 0.85rem;
  line-height: 1.5;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.roles-block {
  margin-bottom: 0.85rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--fluent-border-stroke);
}

.roles-block .hint.tiny {
  margin: 0.25rem 0 0.45rem;
  font-size: 0.78rem;
}

.guide-lead {
  font-size: 0.92rem;
  color: var(--fluent-text-primary);
}

.ver-quick-dl {
  margin: 0.75rem 0;
  padding: 0.65rem 0.75rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-subtle);
}
.ver-quick-label {
  display: block;
  font-size: 0.8rem;
  color: var(--fluent-text-secondary);
  margin-bottom: 0.45rem;
}
.ver-quick-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.ollama-model-box {
  margin-bottom: 0.85rem;
  padding: 0.65rem 0.75rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  font-size: 0.82rem;
  line-height: 1.55;
  color: var(--fluent-text-primary);
}
.ollama-model-box ul {
  margin: 0.4rem 0 0;
  padding-left: 1.1rem;
}
.ollama-model-box li {
  margin-bottom: 0.35rem;
}
.linkish.inline {
  display: inline;
  font-size: inherit;
  vertical-align: baseline;
}

.role-install-row {
  margin-top: 0.45rem;
}

.install-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: rgba(0, 0, 0, 0.45);
}

.install-modal-panel {
  width: min(440px, 100%);
  max-height: min(90vh, 640px);
  overflow: auto;
  padding: 1.1rem 1.25rem;
  box-shadow: var(--fluent-shadow-soft);
}

.install-modal-panel h2 {
  margin: 0 0 0.5rem;
  font-size: 1.05rem;
}

.modal-zip-path {
  word-break: break-all;
  font-size: 0.78rem;
  color: var(--fluent-text-secondary);
}

.modal-label {
  display: block;
  font-weight: 600;
  margin: 0.65rem 0 0.25rem;
  font-size: 0.88rem;
}

.modal-select {
  width: 100%;
  margin-top: 0.35rem;
  padding: 0.4rem 0.5rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  font-size: 0.88rem;
  background: var(--fluent-bg-card);
  color: var(--fluent-text-primary);
}

.modal-custom {
  margin-top: 0.65rem;
}

.modal-custom label {
  display: block;
  font-size: 0.82rem;
  margin-bottom: 0.25rem;
}

.modal-custom input {
  width: 100%;
  padding: 0.4rem 0.5rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  font-size: 0.88rem;
}

.modal-check {
  display: flex;
  align-items: flex-start;
  gap: 0.45rem;
  margin-top: 0.75rem;
  font-size: 0.82rem;
  line-height: 1.45;
  cursor: pointer;
}

.modal-check input {
  margin-top: 0.15rem;
}

.modal-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  margin-top: 1rem;
  justify-content: flex-end;
}
</style>
