<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import HelpHint from './components/HelpHint.vue'

const VIEW_LABELS: Record<string, string> = {
  start: '新手入门',
  announce: '公告通知',
  version: '版本与下载',
  apps: '启动软件',
  assistant: '环境检查',
  logs: '运行日志',
}

export interface LauncherConfig {
  editorProjectRoot: string
  editorExe: string
  editorMode: 'web' | 'dev' | 'exe'
  editorNpmScript: string
  /** 留空则由后端按 GitHub owner/repo 拼 Pages 地址 */
  editorWebUrl: string
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
  /** ollama | remote — 注入 OCLIVE_LLM_BACKEND */
  ocliveLlmMode: 'ollama' | 'remote'
  ocliveRemoteLlmUrl: string
  ocliveRemoteLlmToken: string
  ocliveRemoteLlmTimeoutMs: string
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
  editorMode: 'web',
  editorNpmScript: 'tauri:dev',
  editorWebUrl: '',
  ocliveProjectRoot: '',
  ocliveExe: '',
  ocliveMode: 'dev',
  ocliveNpmScript: 'tauri:dev',
  githubEditorOwner: 'linkaiheng2233-cyber',
  githubEditorRepo: 'oclive-pack-editor',
  githubOcliveOwner: 'linkaiheng2233-cyber',
  githubOcliveRepo: 'oclivenewnew',
  ocliveRolesDir: '',
  ocliveLlmMode: 'ollama',
  ocliveRemoteLlmUrl: '',
  ocliveRemoteLlmToken: '',
  ocliveRemoteLlmTimeoutMs: '',
})

const announcements = ref('')
const statusMsg = ref('')
const logFilter = ref<
  'all' | 'editor' | 'oclive' | 'ollama' | 'winget' | 'bundled-ollama'
>('all')
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
const wingetAvailable = ref(false)
const wingetInstallBusy = ref(false)
/** 非空表示打包内含有 bundled/ollama/OllamaSetup.exe（仅 Windows） */
const bundledOllamaPath = ref<string | null>(null)

/** 编写器网页地址留空时的默认 GitHub Pages（与配置里 owner/repo 一致） */
const editorPagesUrlPreview = computed(() => {
  const o = config.value.githubEditorOwner?.trim() || 'linkaiheng2233-cyber'
  const r = config.value.githubEditorRepo?.trim() || 'oclive-pack-editor'
  return `https://${o}.github.io/${r}/`
})

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

/** 开发模式依赖 Node / npm */
const nodeNeedsAttention = computed(() => {
  const d = envDiag.value
  if (!d) return false
  return !d.nodeVersion || !d.npmVersion
})

/** 本机 Ollama 大脑：未选云端 Remote 且 CLI/API 异常时提示 */
const ollamaNeedsAttention = computed(() => {
  if (config.value.ocliveLlmMode === 'remote') return false
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

const logPanelText = computed(() =>
  filteredLogs.value.map((l) => `[${l.app}][${l.stream}] ${l.line}`).join('\n'),
)

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

async function refreshWingetAvailability() {
  try {
    wingetAvailable.value = await invoke<boolean>('winget_available')
  } catch {
    wingetAvailable.value = false
  }
}

async function refreshBundledOllamaInfo() {
  try {
    bundledOllamaPath.value = await invoke<string | null>('bundled_ollama_installer_path')
  } catch {
    bundledOllamaPath.value = null
  }
}

async function loadAll() {
  try {
    const c = await invoke<LauncherConfig>('load_config')
    config.value = { ...config.value, ...c }
    announcements.value = await invoke<string>('load_announcements')
    await refreshLocalVersions()
    await refreshWingetAvailability()
    await refreshBundledOllamaInfo()
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

async function launchBundledOllamaInstaller() {
  if (!bundledOllamaPath.value) return
  if (
    !confirm(
      '将启动附带的 Ollama 安装程序（Windows）。若已安装过 Ollama，向导可能提示修复或卸载。是否继续？',
    )
  ) {
    return
  }
  try {
    await invoke('launch_bundled_ollama_installer')
    statusMsg.value = '已启动附带安装程序；完成后请在「环境」页点「重新检测」。'
    focusLogs('bundled-ollama')
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function installOllamaViaWinget() {
  if (
    !confirm(
      '将通过 Windows 官方包管理器 winget 安装「Ollama.Ollama」。可能弹出 UAC 或安装向导，且需网络下载。是否继续？',
    )
  ) {
    return
  }
  wingetInstallBusy.value = true
  try {
    await invoke('install_ollama_via_winget')
    statusMsg.value =
      '已开始 winget 安装 Ollama，进度见「日志」→ 筛选 winget。完成后请点「重新检测」。'
    focusLogs('winget')
  } catch (e) {
    statusMsg.value = String(e)
  } finally {
    wingetInstallBusy.value = false
  }
}

async function pullRecommendedOllamaModel() {
  pullBusy.value = true
  try {
    await invoke('ollama_pull_model', { model: DEFAULT_OLLAMA_MODEL })
    statusMsg.value = `已开始拉取「${DEFAULT_OLLAMA_MODEL}」，进度见「日志」→ 筛选 ollama。`
    focusLogs('ollama')
  } catch (e) {
    statusMsg.value = String(e)
  } finally {
    pullBusy.value = false
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
    focusLogs('ollama')
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
    if (config.value.editorMode === 'web') {
      statusMsg.value =
        '已在系统默认浏览器中打开编写器（若没看见页面，请检查任务栏或被拦截的弹窗）'
    }
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
    await refreshWingetAvailability()
    await refreshBundledOllamaInfo()
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
  { id: 'start', label: '新手', icon: '🚀' },
  { id: 'announce', label: '公告', icon: '📢' },
  { id: 'version', label: '版本', icon: '📦' },
  { id: 'apps', label: '启动', icon: '▶' },
  { id: 'assistant', label: '环境', icon: '🩺' },
  { id: 'logs', label: '日志', icon: '📋' },
] as const

const activeNav = ref<string>('announce')

function focusLogs(filter: 'ollama' | 'winget' | 'bundled-ollama') {
  activeNav.value = 'logs'
  logFilter.value = filter
}

const currentViewLabel = computed(() => VIEW_LABELS[activeNav.value] ?? '')

function setView(id: string) {
  activeNav.value = id
}

function onDocKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && installModalOpen.value) {
    cancelInstallRolePackModal()
  }
}

onMounted(async () => {
  document.addEventListener('keydown', onDocKeydown)
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
  document.removeEventListener('keydown', onDocKeydown)
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
              <template v-if="activeNav === 'start'">
                不知道怎么下手就看这一页：按顺序做一遍，就能从「写设定」到「开聊」。
              </template>
              <template v-else-if="activeNav === 'announce'">
                写一段给创作者看的说明，只保存在你电脑上；和别的页面互不打扰。
              </template>
              <template v-else-if="activeNav === 'version'">
                对照网上发布的版本号，顺便一键打开下载页。
              </template>
              <template v-else-if="activeNav === 'apps'">
                先配好聊天软件（oclive）和写设定的工具（编写器），再点按钮打开；一般不用开黑窗口。
              </template>
              <template v-else-if="activeNav === 'assistant'">
                看看 Node、Ollama、文件夹路径对不对；不对就按提示装或改路径。
              </template>
              <template v-else>软件在后台打印的信息都在这里，出问题先来这里瞄一眼。</template>
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
        <div class="section-title-row">
          <h2>新手照着做就行</h2>
          <HelpHint
            text="启动器帮你把「写角色 → 放进文件夹 → 打开聊天软件」串起来。不必一次全懂，哪步卡了就去左边「环境」看检测结果。"
          />
        </div>
        <p class="hint guide-lead">
          你可以只聊天、只做角色，或两个都来——下面是一条<strong>最省事</strong>的路线：<strong>写设定 → 放进角色文件夹 → 开 oclive 聊天</strong>。细节以后在「版本」「环境」里慢慢补。
        </p>
        <p class="hint">
          三个东西分工不同：<strong>本程序</strong>负责一键打开；<strong>编写器</strong>用来写内容；<strong>oclive</strong>是聊天窗口。角色文件都放在磁盘上的
          <code>roles</code> 一类文件夹里（启动器里叫「角色包根目录」）。
        </p>
        <ol class="guide-steps">
          <li>
            <strong>先把环境备好</strong>：开发用要装 <strong>Node</strong>；电脑本地跑对话大脑要装 <strong>Ollama</strong>。第一次打开启动器会自动帮你测一遍，也可随时去「环境」点「重新检测」。
          </li>
          <li>
            <strong>下载或克隆软件</strong>：「版本」页能跳到 GitHub 下载安装包；会开发的同学也可以把仓库克隆到本地。
          </li>
          <li>
            <strong>在「启动」里填路径</strong>：告诉启动器编写器和 oclive 在哪（网页 / 文件夹 / exe 三选一）；再填「角色包根目录」让聊天软件找得到角色（可点「从 oclive 仓库填入」偷懒）。
          </li>
          <li>
            <strong>准备角色文件</strong>：编写器导出 zip，解压到角色目录里对应角色文件夹；或用编写器自带的「写入文件夹」。
          </li>
          <li>
            <strong>开聊</strong>：从启动器启动 oclive，在软件里选角色对话。用 Ollama 的话记得先拉模型（「环境」页有快捷按钮）。
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
        <div class="section-title-row">
          <h2>给创作者看的公告</h2>
          <HelpHint text="写在这儿的字只存在你电脑里，用来提醒自己或同伴；不会自动发到网上。" />
        </div>
        <p class="hint">改完记得点「保存公告」。和版本号、启动按钮、日志不是一回事。</p>
        <textarea v-model="announcements" class="announce" rows="14" spellcheck="false" />
        <button type="button" class="btn" @click="saveAnnouncements">保存公告</button>
      </section>

      <section v-else-if="activeNav === 'version'" class="view-panel card">
        <div class="section-title-row">
          <h2>看版本、去下载</h2>
          <HelpHint
            text="左边是网上最新发布号，右边是你本机仓库里 package.json 的版本。用自己 fork 的话把 owner/repo 改成你的。"
          />
        </div>
        <p class="hint">
          默认已经填了官方仓库名；点下面按钮会直接跳到 GitHub 发布页下载安装包。
        </p>
        <div class="ver-quick-dl">
          <span class="ver-quick-label">一键打开下载页</span>
          <div class="ver-quick-btns">
            <button type="button" class="btn" @click="openRelease(releasesEditorUrl)">编写器 Releases</button>
            <button type="button" class="btn" @click="openRelease(releasesOcliveUrl)">oclive 运行时 Releases</button>
          </div>
        </div>
        <p class="hint">填好仓库名后点「检查更新」，会把网上最新 Tag 和本机版本并排给你看（仓库要能在 GitHub 上访问）。</p>

        <div class="gh-row">
          <label>编写器在哪个仓库</label>
          <div class="gh-inputs">
            <input v-model="config.githubEditorOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubEditorRepo" placeholder="repo" />
          </div>
        </div>
        <div class="ver-line">
          <span>本机版本</span>
          <strong>{{ editorLocalVer ?? '—' }}</strong>
        </div>
        <div class="ver-line" v-if="editorRemote">
          <span>网上最新</span>
          <strong>{{ editorRemote.tagName }}</strong>
          <button type="button" class="btn tiny" @click="openRelease(editorRemote.htmlUrl)">
            打开发布页
          </button>
        </div>

        <hr class="sep" />

        <div class="gh-row">
          <label>oclive 聊天软件在哪个仓库</label>
          <div class="gh-inputs">
            <input v-model="config.githubOcliveOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubOcliveRepo" placeholder="repo" />
          </div>
        </div>
        <div class="ver-line">
          <span>本机版本</span>
          <strong>{{ ocliveLocalVer ?? '—' }}</strong>
        </div>
        <div class="ver-line" v-if="ocliveRemote">
          <span>网上最新</span>
          <strong>{{ ocliveRemote.tagName }}</strong>
          <button type="button" class="btn tiny" @click="openRelease(ocliveRemote.htmlUrl)">
            打开发布页
          </button>
        </div>

        <p v-if="checkErr" class="err">{{ checkErr }}</p>
        <button type="button" class="btn primary" @click="checkReleases">检查更新</button>
      </section>

    <section v-else-if="activeNav === 'assistant'" class="view-panel card">
        <div class="section-title-row">
          <h2>本机环境一眼看完</h2>
          <HelpHint
            text="下面表格逐项打勾：绿的算过关，红的就是还要装软件或改路径。看不懂的名词可以点「展开」看白话说明。"
          />
        </div>
        <div v-if="envDiag && nodeNeedsAttention" class="banner-warn banner-node" role="status">
          <strong>没检测到 Node / npm</strong>：只有当你要用「开发模式」跑源码时才必须装；若只用安装包 exe，可以忽略。
          <button type="button" class="btn tiny" @click="openRelease('https://nodejs.org/')">去下 Node</button>
        </div>
        <div v-if="envDiag && ollamaNeedsAttention" class="banner-warn" role="status">
          <strong>Ollama 没连上</strong>：打算让对话走本机模型时，需要先装好并打开 Ollama（托盘里常驻）。
          <button type="button" class="btn tiny" @click="openRelease('https://ollama.com/download')">去下 Ollama</button>
          <button
            v-if="wingetAvailable"
            type="button"
            class="btn tiny primary"
            :disabled="wingetInstallBusy"
            @click="installOllamaViaWinget"
          >
            winget 一键装
          </button>
          <button
            v-if="bundledOllamaPath"
            type="button"
            class="btn tiny"
            :disabled="wingetInstallBusy"
            @click="launchBundledOllamaInstaller"
          >
            跑附带安装包
          </button>
        </div>
        <div v-if="config.ocliveLlmMode === 'remote' && envDiag" class="banner-hint-remote" role="note">
          你在「启动」里选了<strong>云端大脑</strong>，聊天可以不靠本机 Ollama；下面装 zip、选模型时若仍要用本机模型，下面的按钮照样有用。
        </div>

        <div class="assistant-actions">
          <button type="button" class="btn primary" @click="() => runEnvironmentDiagnose()">重新检测一遍</button>
          <button type="button" class="btn" @click="openLauncherConfigFolder">打开配置文件夹</button>
          <button type="button" class="btn danger" @click="resetLauncherConfig">恢复默认配置</button>
        </div>
        <p v-if="envDiagErr" class="err">{{ envDiagErr }}</p>
        <p class="hint">
          点「重新检测」刷新表格。配置文件坏了可用「恢复默认」，旧文件会尽量改名成
          <code>launcher-config.json.corrupt.bak</code> 留着。
        </p>
        <table v-if="envDiag" class="diag-table">
          <tbody>
            <tr>
              <th>Node.js</th>
              <td :class="{ ok: !!envDiag.nodeVersion, bad: !envDiag.nodeVersion }">
                {{ envDiag.nodeVersion ?? '没装或没进 PATH（开发模式才需要）' }}
              </td>
            </tr>
            <tr>
              <th>npm</th>
              <td :class="{ ok: !!envDiag.npmVersion, bad: !envDiag.npmVersion }">
                {{ envDiag.npmVersion ?? '未检测到' }}
              </td>
            </tr>
            <tr>
              <th>Ollama 命令行</th>
              <td :class="{ ok: !!envDiag.ollamaVersion, bad: !envDiag.ollamaVersion }">
                {{ envDiag.ollamaVersion ?? '没找到命令（有时服务仍在跑，看下一行）' }}
              </td>
            </tr>
            <tr>
              <th>Ollama 服务</th>
              <td :class="{ ok: envDiag.ollamaApiReachable, bad: !envDiag.ollamaApiReachable }">
                {{
                  envDiag.ollamaApiReachable
                    ? '本机 11434 端口通着'
                    : '连不上（先打开 Ollama 软件）'
                }}
              </td>
            </tr>
            <tr>
              <th>编写器项目</th>
              <td :class="{ ok: envDiag.editorProjectOk && envDiag.editorPackageJson, bad: !envDiag.editorProjectOk }">
                <template v-if="config.editorMode === 'web'">用浏览器，不用本地文件夹</template>
                <template v-else-if="!config.editorProjectRoot?.trim()">没填（开发模式才要填）</template>
                <template v-else-if="!envDiag.editorProjectOk">路径不存在或不是文件夹</template>
                <template v-else-if="!envDiag.editorPackageJson">缺少 package.json</template>
                <template v-else>正常</template>
              </td>
            </tr>
            <tr>
              <th>oclive 项目</th>
              <td :class="{ ok: envDiag.ocliveProjectOk && envDiag.oclivePackageJson, bad: !envDiag.ocliveProjectOk }">
                <template v-if="!config.ocliveProjectRoot?.trim()">没填（开发模式才要填）</template>
                <template v-else-if="!envDiag.ocliveProjectOk">路径不存在或不是文件夹</template>
                <template v-else-if="!envDiag.oclivePackageJson">缺少 package.json</template>
                <template v-else>正常</template>
              </td>
            </tr>
            <tr>
              <th>角色文件夹</th>
              <td
                :class="{
                  ok: envDiag.ocliveRolesDirOk,
                  bad: !!config.ocliveRolesDir?.trim() && !envDiag.ocliveRolesDirOk,
                }"
              >
                <template v-if="!config.ocliveRolesDir?.trim()">没填也行；填了启动 oclive 会自动指过去</template>
                <template v-else-if="!envDiag.ocliveRolesDirOk">路径不对</template>
                <template v-else-if="!envDiag.ocliveRolesDirHasRoleHint">文件夹在，还没看到角色文件（可先启动再装）</template>
                <template v-else>看起来已有角色数据</template>
              </td>
            </tr>
          </tbody>
        </table>

        <details class="ollama-details">
          <summary>展开：Ollama 是啥、模型怎么下、和「云端大脑」啥关系</summary>
          <ul class="ollama-details-list">
            <li>
              <strong>安装</strong>：到
              <button type="button" class="linkish inline" @click="openRelease('https://ollama.com/download')">ollama.com</button>
              下安装包，装完让它在后台跑。Windows 可用 winget 或启动器打包里附带的安装包（可能弹 UAC）。
            </li>
            <li>
              <strong>模型</strong>：装完软件还要单独拉模型。可在「启动」里装 zip 角色包时顺手拉，或终端执行
              <code>ollama pull 模型名</code>；列表见
              <button type="button" class="linkish inline" @click="openRelease('https://ollama.com/library')">模型库</button>。常用推荐
              <code>qwen2.5:7b</code>。
            </li>
            <li>
              <strong>云端</strong>：<code>ollama pull</code> 只管本机；要用 OpenAI 类云端 API，要在角色设置里改「大脑」为 remote，并配环境变量，详见主仓库文档
              <code>REMOTE_PLUGIN_PROTOCOL.md</code>。
            </li>
          </ul>
        </details>

        <div class="ollama-model-box">
          <div class="section-title-row section-title-row--inline">
            <strong>Ollama 常用按钮</strong>
            <HelpHint text="一键安装、拉模型、看本机已有模型；进度在「日志」里筛 ollama 看。" />
          </div>
          <div class="env-ollama-quick">
            <span class="env-ollama-quick-label">点一下</span>
            <div class="env-ollama-quick-btns">
              <button
                v-if="bundledOllamaPath"
                type="button"
                class="btn"
                :disabled="wingetInstallBusy || pullBusy"
                @click="launchBundledOllamaInstaller"
              >
                运行附带 Ollama 安装包
              </button>
              <button
                v-if="wingetAvailable"
                type="button"
                class="btn"
                :disabled="wingetInstallBusy || pullBusy"
                @click="installOllamaViaWinget"
              >
                一键安装 Ollama（winget）
              </button>
              <button
                type="button"
                class="btn"
                :disabled="pullBusy || wingetInstallBusy"
                @click="pullRecommendedOllamaModel"
              >
                一键拉取推荐模型（{{ DEFAULT_OLLAMA_MODEL }}）
              </button>
              <button
                type="button"
                class="btn"
                :disabled="pullBusy || wingetInstallBusy"
                @click="refreshOllamaLocalModelsList"
              >
                刷新本机已拉取列表
              </button>
            </div>
            <p v-if="ollamaLocalModels.length" class="hint tiny env-ollama-model-chips">
              本机已有：<code>{{ ollamaLocalModels.join('、') }}</code>
            </p>
            <p v-else class="hint tiny env-ollama-model-chips">
              要先开 Ollama 再点「刷新」，才会列出模型。
            </p>
          </div>
        </div>

        <p class="hint assistant-links">
          <button type="button" class="linkish" @click="openRelease('https://nodejs.org/')">Node.js 下载</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/download')">Ollama 下载</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://github.com/ollama/ollama')">Ollama 文档</button>
        </p>
      </section>

    <div v-else-if="activeNav === 'apps'" class="view-panel grid grid-2 apps-grid">
      <section class="card card--primary-app">
        <div class="section-title-row">
          <h2>oclive（聊天窗口）</h2>
          <HelpHint
            text="这就是真正「聊天」的软件。上面填角色文件夹、选大脑，下面告诉启动器 exe 或源码在哪。"
          />
        </div>
        <div class="roles-block">
          <div class="label-with-hint">
            <label>角色都存在哪个文件夹</label>
            <HelpHint
              text="填「很多个角色文件夹」的共同上一级。里面通常是「角色名/」下面再有一堆配置。不填也能启动，但不会自动帮你指到磁盘上的角色。"
            />
          </div>
          <p class="hint tiny">就是一堆 <code>角色id</code> 子文件夹的<strong>父目录</strong>；启动器会告诉 oclive 去那儿找。</p>
          <div class="row">
            <input v-model="config.ocliveRolesDir" placeholder="例如 D:\oclivenewnew\roles" />
            <button type="button" class="btn" @click="pickRolesRoot">浏览…</button>
            <button type="button" class="btn" @click="fillSuggestedRolesDir">从仓库猜一个</button>
          </div>
          <p class="hint tiny">编写器导出的 zip / ocpak 可以装到这个目录里。</p>
          <div class="row role-install-row">
            <button type="button" class="btn primary" @click="beginInstallRolePack">用 zip 装一个角色包…</button>
          </div>
        </div>
        <div class="llm-backend-block">
          <div class="label-with-hint">
            <label>对话用谁想（大脑）</label>
            <HelpHint
              text="本机：走 Ollama 里的模型。云端：填你自己搭好的接口地址（JSON-RPC），适合不用本机显卡的情况。"
            />
          </div>
          <p class="hint tiny">选「本机」就是 Ollama；选「云端」要在下面填网址，细节见主仓库说明文档。</p>
          <div class="mode">
            <label><input v-model="config.ocliveLlmMode" type="radio" value="ollama" /> 本机 Ollama</label>
            <label><input v-model="config.ocliveLlmMode" type="radio" value="remote" /> 云端接口</label>
          </div>
          <template v-if="config.ocliveLlmMode === 'remote'">
            <label>云端地址（JSON-RPC）</label>
            <input
              v-model="config.ocliveRemoteLlmUrl"
              placeholder="例如 http://127.0.0.1:8765/rpc"
              autocomplete="off"
            />
            <label>令牌 Token（没有就空着）</label>
            <input
              v-model="config.ocliveRemoteLlmToken"
              type="password"
              autocomplete="off"
              placeholder="可选"
            />
            <label>超时多少毫秒（可选）</label>
            <input
              v-model="config.ocliveRemoteLlmTimeoutMs"
              placeholder="例如 120000"
              inputmode="numeric"
            />
          </template>
        </div>
        <div class="mode mode--wrap">
          <label><input v-model="config.ocliveMode" type="radio" value="dev" /> 本地跑源码（npm）</label>
          <label><input v-model="config.ocliveMode" type="radio" value="exe" /> 用安装好的 exe</label>
        </div>
        <template v-if="config.ocliveMode === 'dev'">
          <label>oclive 源码文件夹</label>
          <div class="row">
            <input v-model="config.ocliveProjectRoot" placeholder="例如 D:\oclivenewnew" />
            <button type="button" class="btn" @click="pickOcliveRoot">浏览…</button>
          </div>
          <label>npm 里要跑哪条命令</label>
          <input v-model="config.ocliveNpmScript" placeholder="一般写 tauri:dev" />
        </template>
        <template v-else>
          <label>oclive 的 exe 在哪</label>
          <div class="row">
            <input v-model="config.ocliveExe" placeholder="选你的 oclive.exe" />
            <button type="button" class="btn" @click="pickOcliveExe">浏览…</button>
          </div>
        </template>
        <div class="actions">
          <button type="button" class="btn primary" @click="spawnOclive">启动 oclive</button>
          <button type="button" class="btn danger" @click="stopOclive">关掉</button>
        </div>
      </section>

      <section class="card">
        <div class="section-title-row">
          <h2>角色包编写器</h2>
          <HelpHint
            text="用来写人设、世界观、导出 zip。一般用浏览器打开官网就够；只有要改源码时才选下面两种。"
          />
        </div>
        <div class="mode mode--wrap">
          <label><input v-model="config.editorMode" type="radio" value="web" /> 用网页（省事）</label>
          <label><input v-model="config.editorMode" type="radio" value="dev" /> 本地源码（npm）</label>
          <label><input v-model="config.editorMode" type="radio" value="exe" /> 安装版 exe</label>
        </div>
        <template v-if="config.editorMode === 'web'">
          <label>网页地址（可空）</label>
          <input
            v-model="config.editorWebUrl"
            type="url"
            autocomplete="off"
            :placeholder="`不填就用：${editorPagesUrlPreview}`"
          />
          <p class="hint tiny">空着时按「版本」页的仓库名打开线上 Pages；本地调试可填 <code>http://127.0.0.1:…</code>。</p>
        </template>
        <template v-else-if="config.editorMode === 'dev'">
          <label>编写器源码根目录</label>
          <div class="row">
            <input v-model="config.editorProjectRoot" placeholder="例如 D:\oclive-pack-editor" />
            <button type="button" class="btn" @click="pickEditorRoot">浏览…</button>
          </div>
          <label>npm 脚本名</label>
          <input v-model="config.editorNpmScript" placeholder="tauri:dev" />
        </template>
        <template v-else>
          <label>编写器 exe</label>
          <div class="row">
            <input v-model="config.editorExe" placeholder=".exe 路径" />
            <button type="button" class="btn" @click="pickEditorExe">浏览…</button>
          </div>
        </template>
        <div class="actions">
          <button type="button" class="btn primary" @click="spawnEditor">
            {{ config.editorMode === 'web' ? '在浏览器打开' : '启动编写器' }}
          </button>
          <button type="button" class="btn danger" @click="stopEditor">关掉</button>
        </div>
      </section>
    </div>

    <section v-else-if="activeNav === 'logs'" class="view-panel card log-card">
      <div class="log-head">
        <div class="section-title-row log-title-row">
          <h2>后台日志</h2>
          <HelpHint
            text="编写器、oclive、装模型时打印的黑字都在这。卡住了先筛到对应一项看最后几行报错。"
          />
        </div>
        <div class="log-tools">
          <label>只看</label>
          <select v-model="logFilter">
            <option value="all">全部</option>
            <option value="editor">编写器</option>
            <option value="oclive">oclive</option>
            <option value="ollama">拉模型</option>
            <option value="winget">winget 安装</option>
            <option value="bundled-ollama">附带安装包</option>
          </select>
          <button type="button" class="btn" @click="clearLogs">清空</button>
        </div>
      </div>
      <pre class="log">{{ logPanelText }}</pre>
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
            <button
              type="button"
              class="btn"
              :disabled="pullBusy || wingetInstallBusy"
              @click="refreshOllamaLocalModelsList"
            >
              刷新本机列表
            </button>
            <button
              type="button"
              class="btn"
              :disabled="pullBusy || installBusy || wingetInstallBusy"
              @click="pullSelectedOllamaModel"
            >
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

.section-title-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.15rem;
  margin-bottom: 0.5rem;
}

.section-title-row h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.section-title-row--inline {
  margin-bottom: 0.35rem;
}

.label-with-hint {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.2rem;
  margin-top: 0.5rem;
  margin-bottom: 0.15rem;
}

.label-with-hint label {
  margin: 0;
  display: inline;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--fluent-text-secondary);
}

.ollama-details {
  margin: 0.75rem 0;
  padding: 0.5rem 0.65rem;
  border: 1px dashed var(--fluent-border-stroke);
  border-radius: var(--fluent-radius);
  background: var(--fluent-bg-subtle);
}

.ollama-details summary {
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--fluent-text-secondary);
  user-select: none;
}

.ollama-details-list {
  margin: 0.5rem 0 0;
  padding-left: 1.1rem;
  font-size: 0.8125rem;
  color: var(--fluent-text-secondary);
  line-height: 1.45;
}

.card--primary-app {
  border-color: color-mix(in srgb, var(--fluent-accent) 25%, var(--fluent-border-stroke));
}

.log-title-row {
  margin-bottom: 0;
  flex: 1;
  min-width: 0;
}

.log-title-row h2 {
  margin: 0;
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

.mode--wrap {
  flex-wrap: wrap;
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

.banner-node {
  border-color: rgba(0, 99, 177, 0.4);
  background: rgba(0, 120, 212, 0.1);
}

.banner-hint-remote {
  margin-bottom: 0.75rem;
  padding: 0.55rem 0.7rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-subtle);
  font-size: 0.82rem;
  line-height: 1.5;
  color: var(--fluent-text-primary);
}

.env-ollama-quick {
  margin-top: 0.75rem;
  padding-top: 0.65rem;
  border-top: 1px dashed var(--fluent-border-stroke);
}

.env-ollama-quick-label {
  display: block;
  font-size: 0.78rem;
  font-weight: 600;
  color: var(--fluent-text-secondary);
  margin-bottom: 0.4rem;
}

.env-ollama-quick-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.env-ollama-model-chips {
  margin: 0.45rem 0 0;
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

.llm-backend-block {
  margin-bottom: 0.85rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--fluent-border-stroke);
}

.llm-backend-block label {
  display: block;
  margin-top: 0.45rem;
  font-size: 0.82rem;
  font-weight: 600;
}

.llm-backend-block input[type='text'],
.llm-backend-block input[type='password'] {
  width: 100%;
  margin-top: 0.2rem;
  padding: 0.4rem 0.5rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  font-size: 0.88rem;
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
