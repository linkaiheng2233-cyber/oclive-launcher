<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import HelpHint from './components/HelpHint.vue'
import CreatorAnnouncementsSection from './announcements/CreatorAnnouncementsSection.vue'
import DeveloperAnnouncementsSection from './announcements/DeveloperAnnouncementsSection.vue'
import { useDeveloperAnnouncements } from './announcements/useDeveloperAnnouncements'
import { useLauncherUiScale } from './composables/useLauncherUiScale'
import { useRolePackEcho } from './composables/useRolePackEcho'
import type { RolePackEchoConfig } from './lib/rolePackCreatorMessage'
import { normalizeExePathPaste, parseGithubRepoFromUrl } from './lib/launcherPaste'
import {
  LAUNCHER_HINT_ASSISTANT,
  LAUNCHER_HINT_EDITOR_GH_DL,
  LAUNCHER_HINT_EXE_PATH_PASTE,
  LAUNCHER_HINT_GH_URL_PASTE,
  LAUNCHER_HINT_LOGS,
  LAUNCHER_HINT_OCLIVE_GH_DL,
  LAUNCHER_HINT_START_GUIDE,
  LAUNCHER_HINT_TITLEBAR_TOOLS,
  LAUNCHER_HINT_VERSION_ACTIONS,
  LAUNCHER_HINT_VERSION_LOCAL_VS_REMOTE,
  LAUNCHER_HINT_VERSION_PAGE,
  LAUNCHER_HINT_VERSION_QUICK_LINKS,
  LAUNCHER_HINT_VERSION_REPO_EDITOR,
  LAUNCHER_HINT_VERSION_REPO_OCLIVE,
} from './lib/launcherHints'

const VIEW_LABELS: Record<string, string> = {
  start: '新手入门',
  version: '版本与下载',
  'launch-oclive': '启动 oclive',
  'launch-editor': '角色包编写器',
  assistant: '环境检查',
  logs: '运行日志',
}

/** 发给别人用的包请保持 false；本地维护者构建可设为 true 以编辑开发者公告文件 */
const announceEditEnabled = computed(() => import.meta.env.VITE_ANNOUNCE_EDITABLE === 'true')

export interface LauncherConfig extends RolePackEchoConfig {
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
  /** ollama | remote — 注入 OCLIVE_LLM_BACKEND */
  ocliveLlmMode: 'ollama' | 'remote'
  ocliveRemoteLlmUrl: string
  ocliveRemoteLlmToken: string
  ocliveRemoteLlmTimeoutMs: string
  /** 可选：注入 OCLIVE_REMOTE_PLUGIN_URL/TOKEN/TIMEOUT_MS（memory/emotion/event/prompt 共用） */
  ocliveRemotePluginUrl: string
  ocliveRemotePluginToken: string
  ocliveRemotePluginTimeoutMs: string
  /** 可选：开发者公告远程正文 URL（http/https），配合「拉取最新」 */
  developerAnnouncementsUrl: string
}

interface ReleaseInfo {
  tagName: string
  name?: string
  htmlUrl: string
  publishedAt?: string
  body?: string
}

interface GhReleaseAsset {
  name: string
  browserDownloadUrl: string
  size: number
}

interface GhDownloadResult {
  savedPath: string
  resolvedExe: string | null
  hint: string | null
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
  ocliveRemotePluginUrl: '',
  ocliveRemotePluginToken: '',
  ocliveRemotePluginTimeoutMs: '',
  launcherEchoRoleId: '',
  developerAnnouncementsUrl: '',
})

const statusMsg = ref('')

async function persistLauncherConfigToDisk() {
  await invoke('save_config', { config: config.value })
}

const {
  echoLines: rolePackEchoLines,
  roleIds: rolePackRoleIds,
  refreshEchoUi: refreshRolePackEchoUi,
  persistFollowRole: persistLauncherEchoRole,
  clearFollowRole: clearLauncherEchoRole,
} = useRolePackEcho(config, {
  setStatus: (m) => {
    statusMsg.value = m
  },
  persistLauncherConfig: persistLauncherConfigToDisk,
})

const {
  text: devAnnounceBody,
  fetchBusy: devAnnounceFetchBusy,
  reloadFromDisk: reloadDevAnnounceFromDisk,
  saveToDisk: saveDevAnnounceToDisk,
  fetchRemoteAndCache: fetchDevAnnounceFromUrl,
} = useDeveloperAnnouncements((m) => {
  statusMsg.value = m
})

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

const ocliveGhAssets = ref<GhReleaseAsset[]>([])
const editorGhAssets = ref<GhReleaseAsset[]>([])
const ocliveGhAssetUrl = ref('')
const editorGhAssetUrl = ref('')
const ocliveGhBusy = ref(false)
const editorGhBusy = ref(false)

/** 粘贴完整 GitHub 仓库网址后，点「填入」拆成 owner / repo */
const editorGhUrlPaste = ref('')
const ocliveGhUrlPaste = ref('')

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

const APPS_LOG_PREVIEW_LINES = 24
const appsLogPreviewEl = ref<HTMLElement | null>(null)

const ocliveLogPreviewText = computed(() =>
  logs.value
    .filter((l) => l.app === 'oclive')
    .slice(-APPS_LOG_PREVIEW_LINES)
    .map((l) => `[${l.stream}] ${l.line}`)
    .join('\n'),
)

const editorLogPreviewText = computed(() =>
  logs.value
    .filter((l) => l.app === 'editor')
    .slice(-APPS_LOG_PREVIEW_LINES)
    .map((l) => `[${l.stream}] ${l.line}`)
    .join('\n'),
)

watch(
  () => logs.value.length,
  async () => {
    await nextTick()
    const el = appsLogPreviewEl.value
    if (el) el.scrollTop = el.scrollHeight
  },
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
    await refreshRolePackEchoUi()
    await reloadDevAnnounceFromDisk()
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
    await refreshRolePackEchoUi()
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

async function applyEditorRepoFromPastedUrl() {
  const r = parseGithubRepoFromUrl(editorGhUrlPaste.value)
  if (!r) {
    statusMsg.value =
      '认不出 GitHub 仓库地址。请粘贴浏览器地址栏里的链接，例如 https://github.com/用户名/仓库名'
    return
  }
  config.value.githubEditorOwner = r.owner
  config.value.githubEditorRepo = r.repo
  statusMsg.value = `已填入编写器仓库：${r.owner}/${r.repo}（可到「编写器」页点「列出附件」下载）`
  await saveConfig()
}

async function applyOcliveRepoFromPastedUrl() {
  const r = parseGithubRepoFromUrl(ocliveGhUrlPaste.value)
  if (!r) {
    statusMsg.value =
      '认不出 GitHub 仓库地址。请粘贴浏览器地址栏里的链接，例如 https://github.com/用户名/仓库名'
    return
  }
  config.value.githubOcliveOwner = r.owner
  config.value.githubOcliveRepo = r.repo
  statusMsg.value = `已填入 oclive 仓库：${r.owner}/${r.repo}（可到「启动 oclive」页点「列出附件」下载）`
  await saveConfig()
}

function onOcliveExeInputPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  const p = normalizeExePathPaste(text)
  if (!p) return
  e.preventDefault()
  config.value.ocliveExe = p
  config.value.ocliveMode = 'exe'
  statusMsg.value = '已从粘贴识别 oclive.exe，已切换到「已安装的 exe」'
  void saveConfig()
}

function onEditorExeInputPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  const p = normalizeExePathPaste(text)
  if (!p) return
  e.preventDefault()
  config.value.editorExe = p
  config.value.editorMode = 'exe'
  statusMsg.value = '已从粘贴识别编写器 exe，已切换到「本地 exe」'
  void saveConfig()
}

async function applyOcliveExeFromField() {
  const p = normalizeExePathPaste(config.value.ocliveExe)
  if (!p) {
    statusMsg.value = '请填写以 .exe 结尾的完整路径（可含引号）'
    return
  }
  config.value.ocliveExe = p
  config.value.ocliveMode = 'exe'
  statusMsg.value = '已整理 oclive.exe 路径并切换到 exe 模式'
  await saveConfig()
}

async function applyEditorExeFromField() {
  const p = normalizeExePathPaste(config.value.editorExe)
  if (!p) {
    statusMsg.value = '请填写以 .exe 结尾的完整路径（可含引号）'
    return
  }
  config.value.editorExe = p
  config.value.editorMode = 'exe'
  statusMsg.value = '已整理编写器 exe 路径并切换到本地 exe 模式'
  await saveConfig()
}

function formatGhBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

async function refreshOcliveGhAssets() {
  ocliveGhBusy.value = true
  try {
    ocliveGhAssets.value = await invoke<GhReleaseAsset[]>('gh_latest_release_assets', {
      owner: config.value.githubOcliveOwner.trim(),
      repo: config.value.githubOcliveRepo.trim(),
    })
    ocliveGhAssetUrl.value = ''
    statusMsg.value = ocliveGhAssets.value.length
      ? `已列出 oclive 仓库 ${ocliveGhAssets.value.length} 个 Release 附件`
      : '该 Release 下没有附件（或仓库尚无 Release）'
  } catch (e) {
    ocliveGhAssets.value = []
    ocliveGhAssetUrl.value = ''
    statusMsg.value = String(e)
  } finally {
    ocliveGhBusy.value = false
  }
}

async function refreshEditorGhAssets() {
  editorGhBusy.value = true
  try {
    editorGhAssets.value = await invoke<GhReleaseAsset[]>('gh_latest_release_assets', {
      owner: config.value.githubEditorOwner.trim(),
      repo: config.value.githubEditorRepo.trim(),
    })
    editorGhAssetUrl.value = ''
    statusMsg.value = editorGhAssets.value.length
      ? `已列出编写器仓库 ${editorGhAssets.value.length} 个 Release 附件`
      : '该 Release 下没有附件（或仓库尚无 Release）'
  } catch (e) {
    editorGhAssets.value = []
    editorGhAssetUrl.value = ''
    statusMsg.value = String(e)
  } finally {
    editorGhBusy.value = false
  }
}

async function downloadOcliveFromGh() {
  const url = ocliveGhAssetUrl.value.trim()
  const asset = ocliveGhAssets.value.find((a) => a.browserDownloadUrl === url)
  if (!asset) {
    statusMsg.value = '请先点「列出附件」并选择一个文件'
    return
  }
  ocliveGhBusy.value = true
  try {
    const r = await invoke<GhDownloadResult>('gh_download_release_asset', {
      url: asset.browserDownloadUrl,
      suggestedFileName: asset.name,
      kind: 'oclive',
    })
    if (r.resolvedExe) {
      config.value.ocliveExe = r.resolvedExe
      config.value.ocliveMode = 'exe'
      await saveConfig()
      statusMsg.value = '已下载并填入 oclive 路径，已切换到 exe 模式并保存配置'
    } else {
      statusMsg.value = r.hint ? `${r.hint}（已保存：${r.savedPath}）` : `已保存：${r.savedPath}`
    }
  } catch (e) {
    statusMsg.value = String(e)
  } finally {
    ocliveGhBusy.value = false
  }
}

async function downloadEditorFromGh() {
  const url = editorGhAssetUrl.value.trim()
  const asset = editorGhAssets.value.find((a) => a.browserDownloadUrl === url)
  if (!asset) {
    statusMsg.value = '请先点「列出附件」并选择一个文件'
    return
  }
  editorGhBusy.value = true
  try {
    const r = await invoke<GhDownloadResult>('gh_download_release_asset', {
      url: asset.browserDownloadUrl,
      suggestedFileName: asset.name,
      kind: 'editor',
    })
    if (r.resolvedExe) {
      config.value.editorExe = r.resolvedExe
      config.value.editorMode = 'exe'
      await saveConfig()
      statusMsg.value = '已下载并填入编写器路径，已切换到 exe 模式并保存配置'
    } else {
      statusMsg.value = r.hint ? `${r.hint}（已保存：${r.savedPath}）` : `已保存：${r.savedPath}`
    }
  } catch (e) {
    statusMsg.value = String(e)
  } finally {
    editorGhBusy.value = false
  }
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
    config.value.launcherEchoRoleId = roleId
    try {
      await persistLauncherConfigToDisk()
    } catch (e) {
      statusMsg.value = String(e)
    }
    await refreshRolePackEchoUi()
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
    focusLogsFilter('bundled-ollama')
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
    focusLogsFilter('winget')
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
    focusLogsFilter('ollama')
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
    focusLogsFilter('ollama')
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

/** OCLive 生态站入口（论坛/文档/发布汇总等）；换域名或仓库名时在 .env 里设 VITE_VERSIONS_PAGE_URL */
const versionsListingPageUrl = computed(() => {
  const u = (import.meta.env.VITE_VERSIONS_PAGE_URL || '').trim()
  if (u) return u
  return 'https://linkaiheng2233-cyber.github.io/oclive-plugin-market/versions'
})

async function openVersionsListingInBrowser() {
  try {
    await invoke('open_url', { url: versionsListingPageUrl.value })
    statusMsg.value = '已在浏览器打开生态站页面'
  } catch (e) {
    statusMsg.value = String(e)
  }
}

/** 将上方两个「粘贴网址」框解析进配置（若有有效内容）并拉取 Release 对照版本 */
async function syncGithubUrlsAndCheckUpdates() {
  let changed = false
  const ed = parseGithubRepoFromUrl(editorGhUrlPaste.value)
  if (ed) {
    config.value.githubEditorOwner = ed.owner
    config.value.githubEditorRepo = ed.repo
    changed = true
  }
  const oc = parseGithubRepoFromUrl(ocliveGhUrlPaste.value)
  if (oc) {
    config.value.githubOcliveOwner = oc.owner
    config.value.githubOcliveRepo = oc.repo
    changed = true
  }
  if (changed) {
    try {
      await invoke('save_config', { config: config.value })
      statusMsg.value = '已从粘贴框同步 GitHub 仓库'
    } catch (e) {
      statusMsg.value = String(e)
      return
    }
  }
  await checkReleases()
}

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

const navItems: {
  id: string
  label: string
  icon: string
  accent?: 'oclive' | 'editor'
}[] = [
  { id: 'start', label: '新手', icon: '🚀' },
  { id: 'version', label: '版本', icon: '📦' },
  { id: 'launch-oclive', label: 'oclive', icon: '💬', accent: 'oclive' },
  { id: 'launch-editor', label: '编写器', icon: '✏️', accent: 'editor' },
  { id: 'assistant', label: '环境', icon: '🩺' },
  { id: 'logs', label: '日志', icon: '📋' },
]

const THEME_STORAGE_KEY = 'oclive-launcher-theme'
type ThemePreference = 'light' | 'dark' | 'system'
const themePreference = ref<ThemePreference>('system')

let themeMediaCleanup: (() => void) | undefined

function isDarkEffective(): boolean {
  if (themePreference.value === 'dark') return true
  if (themePreference.value === 'light') return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyRootTheme() {
  document.documentElement.setAttribute('data-theme', isDarkEffective() ? 'dark' : 'light')
}

function cycleTheme() {
  const order: ThemePreference[] = ['light', 'dark', 'system']
  const i = order.indexOf(themePreference.value)
  themePreference.value = order[(i + 1) % order.length]
  try {
    localStorage.setItem(THEME_STORAGE_KEY, themePreference.value)
  } catch {
    /* ignore */
  }
  applyRootTheme()
}

const themeCycleLabel = computed(() => {
  switch (themePreference.value) {
    case 'light':
      return '浅色'
    case 'dark':
      return '深色'
    default:
      return '跟随系统'
  }
})

const { bumpScale, scaleLabel } = useLauncherUiScale()

const activeNav = ref<string>('start')

type LogAppFilter =
  | 'all'
  | 'editor'
  | 'oclive'
  | 'ollama'
  | 'winget'
  | 'bundled-ollama'

function focusLogsFilter(filter: LogAppFilter) {
  logFilter.value = filter
  activeNav.value = 'logs'
}

const currentViewLabel = computed(() => VIEW_LABELS[activeNav.value] ?? '')

function setView(id: string) {
  activeNav.value = id
}

function onDocKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (installModalOpen.value) cancelInstallRolePackModal()
  }
}

onMounted(async () => {
  document.addEventListener('keydown', onDocKeydown)
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY)
    if (raw === 'light' || raw === 'dark' || raw === 'system') {
      themePreference.value = raw
    }
  } catch {
    /* ignore */
  }
  applyRootTheme()
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  const onScheme = () => {
    if (themePreference.value === 'system') applyRootTheme()
  }
  mq.addEventListener('change', onScheme)
  themeMediaCleanup = () => mq.removeEventListener('change', onScheme)

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
  themeMediaCleanup?.()
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
        :class="[
          'rail-btn',
          { active: activeNav === item.id },
          item.accent ? `rail-btn--accent-${item.accent}` : '',
        ]"
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
                新手指引、开发者公告与创作者公告（包内寄语，只读）都在本页。
              </template>
              <template v-else-if="activeNav === 'version'">
                对照网上发布的版本号，顺便一键打开下载页。
              </template>
              <template v-else-if="activeNav === 'launch-oclive'">
                配角色目录、对话大脑、安装包与启动路径；输出在下方摘要与「日志」里查看。
              </template>
              <template v-else-if="activeNav === 'launch-editor'">
                下载或指定编写器（网页 / 源码 / exe），再一键打开；日志同样汇总在下方与「日志」页。
              </template>
              <template v-else-if="activeNav === 'assistant'">
                看看 Node、Ollama、文件夹路径对不对；不对就按提示装或改路径。
              </template>
              <template v-else>软件在后台打印的信息都在这里，出问题先来这里瞄一眼。</template>
            </p>
          </div>
          <div class="titlebar-actions" role="toolbar" aria-label="外观与字号">
            <div class="shell-scale" aria-label="界面大小">
              <button type="button" class="shell-tool-btn" title="缩小" aria-label="缩小界面" @click="bumpScale(-1)">
                A−
              </button>
              <span class="shell-scale-value" :title="'相对默认字号：' + scaleLabel">{{ scaleLabel }}</span>
              <button type="button" class="shell-tool-btn" title="放大" aria-label="放大界面" @click="bumpScale(1)">
                A+
              </button>
            </div>
            <button
              type="button"
              class="shell-tool-btn shell-theme-btn"
              :title="`主题：${themeCycleLabel}（点击切换）`"
              @click="cycleTheme"
            >
              {{ themeCycleLabel === '跟随系统' ? '◐' : themeCycleLabel === '深色' ? '🌙' : '☀️' }}
              {{ themeCycleLabel }}
            </button>
            <button
              type="button"
              class="btn primary"
              title="把左侧各页的路径、GitHub 仓库、运行模式等写入本机配置文件；主题与界面字号保存在浏览器本地，会即时生效。"
              @click="saveConfig"
            >
              保存配置
            </button>
            <HelpHint class="titlebar-tools-help" :paragraphs="LAUNCHER_HINT_TITLEBAR_TOOLS" />
          </div>
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
        <div v-if="activeNav === 'start'" class="view-panel view-start-stack">
        <section class="card guide-card">
        <div class="section-title-row">
          <h2>新手照着做就行</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_START_GUIDE" />
        </div>
        <p class="hint guide-lead">
          你可以只聊天、只做角色，或两个都来——下面是一条<strong>最省事</strong>的路线：<strong>写设定 → 放进角色文件夹 → 开 oclive 聊天</strong>。看版本号、对 GitHub 发版请点左侧「版本」。
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
            <strong>下载或克隆软件</strong>：打开 OCLive 生态站的<strong>发布汇总</strong>页可一次看到各软件发布列表；也可点左侧「版本」进本页，用快捷按钮或粘贴 GitHub 网址检查更新。会开发的同学也可以把仓库克隆到本地。
          </li>
          <li>
            <strong>在左侧「oclive」「编写器」里填路径</strong>：告诉启动器两个软件在哪（网页 / 文件夹 / exe）；在 oclive 页填「角色包根目录」让聊天软件找得到角色（可点「从仓库猜」偷懒）。
          </li>
          <li>
            <strong>准备角色文件</strong>：编写器导出 zip，解压到角色目录里对应角色文件夹；或用编写器自带的「写入文件夹」。
          </li>
          <li>
            <strong>开聊</strong>：在左侧「oclive」页启动聊天软件，在软件里选角色对话。用 Ollama 的话记得先拉模型（「环境」页有快捷按钮）。
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

      <CreatorAnnouncementsSection
        v-model:launcher-echo-role-id="config.launcherEchoRoleId"
        :role-ids="rolePackRoleIds"
        :echo-lines="rolePackEchoLines"
        :oclive-roles-dir="config.ocliveRolesDir"
        @persist-follow="persistLauncherEchoRole"
        @refresh-roles="refreshRolePackEchoUi"
        @clear-follow="clearLauncherEchoRole"
      />

      <DeveloperAnnouncementsSection
        v-model:url="config.developerAnnouncementsUrl"
        v-model:body="devAnnounceBody"
        :announce-edit-enabled="announceEditEnabled"
        :fetch-busy="devAnnounceFetchBusy"
        @save="saveDevAnnounceToDisk"
        @fetch="fetchDevAnnounceFromUrl(config.developerAnnouncementsUrl)"
      />
        </div>

      <section v-else-if="activeNav === 'version'" class="view-panel card ver-page">
        <div class="section-title-row">
          <h2>看版本、去下载</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_PAGE" />
        </div>
        <p class="hint ver-page-lead">
          大白话：这一页就是帮你对照「电脑上装的版本」和「GitHub 上最新发的一不一样」，再顺手打开市场汇总页或某个仓库的下载页。具体名词点各小节旁的<strong>小问号</strong>看详细说明。
        </p>
        <div class="ver-quick-dl">
          <div class="label-with-hint ver-quick-head">
            <span class="ver-subtle-label">快捷入口</span>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_QUICK_LINKS" />
          </div>
          <div class="ver-quick-btns">
            <button type="button" class="btn" @click="openVersionsListingInBrowser">生态站 · 发布汇总页</button>
            <button type="button" class="btn" @click="openRelease(releasesEditorUrl)">编写器 Releases</button>
            <button type="button" class="btn" @click="openRelease(releasesOcliveUrl)">oclive 运行时 Releases</button>
          </div>
        </div>
        <p class="hint">
          下面分两块：<strong>编写器</strong>一块、<strong>oclive</strong>一块。每块都可以先粘贴整段仓库网址再点「填入」，或手改 owner / repo。最底下两个按钮用来联网查 GitHub；区别见「检查更新」旁的问号。
        </p>

        <div class="gh-paste-block">
          <div class="label-with-hint">
            <label>粘贴编写器仓库网址（可选）</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_GH_URL_PASTE" />
          </div>
          <div class="row">
            <input
              v-model="editorGhUrlPaste"
              class="paste-url-input"
              placeholder="例如 https://github.com/你的用户名/oclive-pack-editor"
              autocomplete="off"
              @keydown.enter.prevent="applyEditorRepoFromPastedUrl"
            />
            <button type="button" class="btn" @click="applyEditorRepoFromPastedUrl">填入 owner / repo</button>
          </div>
        </div>

        <div class="gh-row">
          <div class="label-with-hint gh-row__label">
            <label>编写器在哪个仓库</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_REPO_EDITOR" />
          </div>
          <div class="gh-inputs">
            <input v-model="config.githubEditorOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubEditorRepo" placeholder="repo" />
          </div>
        </div>
        <div class="label-with-hint ver-compare-hint">
          <span class="ver-subtle-label">本机版本 vs 网上最新</span>
          <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_LOCAL_VS_REMOTE" />
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

        <div class="gh-paste-block">
          <div class="label-with-hint">
            <label>粘贴 oclive 仓库网址（可选）</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_GH_URL_PASTE" />
          </div>
          <div class="row">
            <input
              v-model="ocliveGhUrlPaste"
              class="paste-url-input"
              placeholder="例如 https://github.com/你的用户名/oclivenewnew"
              autocomplete="off"
              @keydown.enter.prevent="applyOcliveRepoFromPastedUrl"
            />
            <button type="button" class="btn" @click="applyOcliveRepoFromPastedUrl">填入 owner / repo</button>
          </div>
        </div>

        <div class="gh-row">
          <div class="label-with-hint gh-row__label">
            <label>oclive 聊天软件在哪个仓库</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_REPO_OCLIVE" />
          </div>
          <div class="gh-inputs">
            <input v-model="config.githubOcliveOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubOcliveRepo" placeholder="repo" />
          </div>
        </div>
        <div class="label-with-hint ver-compare-hint">
          <span class="ver-subtle-label">本机版本 vs 网上最新</span>
          <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_LOCAL_VS_REMOTE" />
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
        <div class="ver-actions-wrap">
          <div class="label-with-hint ver-actions-hint">
            <span class="ver-subtle-label">检查更新</span>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_ACTIONS" />
          </div>
          <div class="ver-actions-row">
            <button type="button" class="btn primary" @click="syncGithubUrlsAndCheckUpdates">
              同步粘贴的 GitHub 网址并检查更新
            </button>
            <button type="button" class="btn" @click="checkReleases">仅检查更新（当前 owner/repo）</button>
          </div>
        </div>
      </section>

    <section v-else-if="activeNav === 'assistant'" class="view-panel card">
        <div class="section-title-row">
          <h2>本机环境一眼看完</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_ASSISTANT" />
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
          你在 oclive 页选了<strong>云端大脑</strong>，聊天可以不靠本机 Ollama；下面装 zip、选模型时若仍要用本机模型，下面的按钮照样有用。
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
              <strong>模型</strong>：装完软件还要单独拉模型。可在 oclive 页装 zip 角色包时顺手拉，或终端执行
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

    <div v-else-if="activeNav === 'launch-oclive'" class="view-panel app-launch-page app-launch-page--oclive">
      <p class="apps-terminal-banner">
        在本页启动 <strong>oclive</strong>（含 npm 开发模式）时，<strong>标准输出会进下方摘要与「日志」页</strong>；启动器已尽量隐藏命令行黑窗（Windows）。安装向导与软件自身窗口仍会弹出。
      </p>
      <div class="apps-launch-stack">
        <section class="card card--primary-app card--launch-app card--hero-oclive">
          <div class="section-title-row section-title-row--launch">
            <h2>oclive（聊天窗口）</h2>
            <HelpHint
              text="这就是真正「聊天」的软件。上面填角色文件夹、选大脑，下面告诉启动器 exe 或源码在哪。"
            />
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">① 角色与资源</h3>
            <div class="roles-block roles-block--in-launch">
              <div class="label-with-hint">
                <label>角色包根目录</label>
                <HelpHint
                  text="填「很多个角色文件夹」的共同上一级。里面通常是「角色名/」下面再有一堆配置。不填也能启动，但不会自动帮你指到磁盘上的角色。"
                />
              </div>
              <p class="hint tiny">一堆 <code>角色id</code> 子文件夹的<strong>父目录</strong>；启动器会注入给 oclive。</p>
              <div class="row">
                <input v-model="config.ocliveRolesDir" placeholder="例如 D:\oclivenewnew\roles" />
                <button type="button" class="btn" @click="pickRolesRoot">浏览…</button>
                <button type="button" class="btn" @click="fillSuggestedRolesDir">从仓库猜</button>
              </div>
              <p class="hint tiny">编写器导出的 zip / ocpak 可装到此目录。</p>
              <div class="row role-install-row">
                <button type="button" class="btn primary" @click="beginInstallRolePack">用 zip 装角色包…</button>
              </div>
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">② 对话大脑（LLM）</h3>
            <div class="llm-backend-block llm-backend-block--in-launch">
              <div class="label-with-hint">
                <label>用本机还是云端</label>
                <HelpHint
                  text="本机：走 Ollama 里的模型。云端：填你自己搭好的接口地址（JSON-RPC），适合不用本机显卡的情况。"
                />
              </div>
              <p class="hint tiny">选「本机」即 Ollama；「云端」须填下方地址。</p>
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
                <label>令牌 Token（可选）</label>
                <input
                  v-model="config.ocliveRemoteLlmToken"
                  type="password"
                  autocomplete="off"
                  placeholder="可选"
                />
                <label>超时毫秒（可选）</label>
                <input
                  v-model="config.ocliveRemoteLlmTimeoutMs"
                  placeholder="例如 120000"
                  inputmode="numeric"
                />
              </template>
              <label>模块侧车地址（可选，memory/emotion/event/prompt 共用）</label>
              <input
                v-model="config.ocliveRemotePluginUrl"
                placeholder="例如 http://127.0.0.1:8765/rpc"
                autocomplete="off"
              />
              <label>模块侧车 Token（可选）</label>
              <input
                v-model="config.ocliveRemotePluginToken"
                type="password"
                autocomplete="off"
                placeholder="可选"
              />
              <label>模块侧车超时毫秒（可选）</label>
              <input
                v-model="config.ocliveRemotePluginTimeoutMs"
                placeholder="例如 8000"
                inputmode="numeric"
              />
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">③ 获取 oclive 安装包</h3>
            <p class="hint tiny">不知道 owner / repo 怎么填？去 GitHub 打开仓库首页，复制地址栏，在下面粘贴后点「填入」，与「版本与下载」页共用同一配置。</p>
            <div class="gh-paste-block gh-paste-block--inline">
              <div class="label-with-hint">
                <span class="gh-paste-inline-label">粘贴仓库网址</span>
                <HelpHint :paragraphs="LAUNCHER_HINT_GH_URL_PASTE" />
              </div>
              <div class="row">
                <input
                  v-model="ocliveGhUrlPaste"
                  class="paste-url-input"
                  placeholder="https://github.com/…/…"
                  autocomplete="off"
                  @keydown.enter.prevent="applyOcliveRepoFromPastedUrl"
                />
                <button type="button" class="btn" @click="applyOcliveRepoFromPastedUrl">填入</button>
              </div>
            </div>
            <div class="gh-release-dl gh-release-dl--compact">
              <div class="label-with-hint gh-release-dl__label-row">
                <span class="gh-release-dl__label">GitHub Release（与「版本」里 oclive 仓库一致）</span>
                <HelpHint :paragraphs="LAUNCHER_HINT_OCLIVE_GH_DL" />
              </div>
              <div class="row gh-release-dl-row">
                <button
                  type="button"
                  class="btn"
                  :disabled="ocliveGhBusy"
                  @click="refreshOcliveGhAssets"
                >
                  列出附件
                </button>
                <select v-model="ocliveGhAssetUrl" class="gh-asset-select">
                  <option value="">选择文件…</option>
                  <option
                    v-for="a in ocliveGhAssets"
                    :key="a.browserDownloadUrl"
                    :value="a.browserDownloadUrl"
                  >
                    {{ a.name }} — {{ formatGhBytes(a.size) }}
                  </option>
                </select>
                <button
                  type="button"
                  class="btn primary"
                  :disabled="ocliveGhBusy || !ocliveGhAssetUrl"
                  @click="downloadOcliveFromGh"
                >
                  下载到…
                </button>
              </div>
              <p class="hint tiny">
                另存为选路径；zip 会解压到「文件名_extracted」并尝试填入 exe。
              </p>
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">④ 启动方式与路径</h3>
            <div class="mode mode--wrap">
              <label><input v-model="config.ocliveMode" type="radio" value="dev" /> 本地源码（npm）</label>
              <label><input v-model="config.ocliveMode" type="radio" value="exe" /> 已安装的 exe</label>
            </div>
            <template v-if="config.ocliveMode === 'dev'">
              <label>源码根目录</label>
              <div class="row">
                <input v-model="config.ocliveProjectRoot" placeholder="例如 D:\oclivenewnew" />
                <button type="button" class="btn" @click="pickOcliveRoot">浏览…</button>
              </div>
              <label>npm 脚本名</label>
              <input v-model="config.ocliveNpmScript" placeholder="一般写 tauri:dev" />
            </template>
            <template v-else>
              <div class="label-with-hint">
                <label>oclive.exe 路径</label>
                <HelpHint :paragraphs="LAUNCHER_HINT_EXE_PATH_PASTE" />
              </div>
              <p class="hint tiny">可直接把资源管理器地址栏或快捷方式里的<strong>完整路径</strong>粘贴进框内；也可点「识别框内路径」。</p>
              <div class="row">
                <input
                  v-model="config.ocliveExe"
                  placeholder="例如 C:\...\oclive.exe（可粘贴）"
                  autocomplete="off"
                  @paste="onOcliveExeInputPaste"
                />
                <button type="button" class="btn" @click="pickOcliveExe">浏览…</button>
                <button type="button" class="btn" @click="applyOcliveExeFromField">识别框内路径</button>
              </div>
            </template>
          </div>

          <div class="app-feature-block app-feature-block--run">
            <h3 class="app-feature-block__title">⑤ 运行</h3>
            <div class="actions actions--launch">
              <button type="button" class="btn primary btn-launch" @click="spawnOclive">启动 oclive</button>
              <button type="button" class="btn danger btn-launch" @click="stopOclive">结束进程</button>
              <button type="button" class="btn btn-launch-secondary" @click="focusLogsFilter('oclive')">
                在「日志」里只看 oclive
              </button>
            </div>
          </div>
        </section>
      </div>

      <section class="card apps-log-preview-card">
        <div class="apps-log-preview-head">
          <h3 class="apps-log-preview-title">oclive 输出摘要</h3>
          <div class="apps-log-preview-tools">
            <button type="button" class="btn" @click="focusLogsFilter('all')">打开完整日志页</button>
          </div>
        </div>
        <pre
          ref="appsLogPreviewEl"
          class="apps-log-preview"
          :class="{ 'apps-log-preview--empty': !ocliveLogPreviewText.trim() }"
        >{{ ocliveLogPreviewText || '尚无输出；启动 oclive 后这里会滚动显示最近几行。' }}</pre>
        <p class="hint tiny apps-log-preview-foot">
          完整历史与筛选（ollama、winget 等）在左侧「日志」。
        </p>
      </section>
    </div>

    <div v-else-if="activeNav === 'launch-editor'" class="view-panel app-launch-page app-launch-page--editor">
      <p class="apps-terminal-banner apps-terminal-banner--editor">
        在本页打开 <strong>角色包编写器</strong> 时，npm / exe 的<strong>输出同样汇总在下方与「日志」页</strong>；用「网页」模式则会在系统浏览器中打开。
      </p>
      <div class="apps-launch-stack">
        <section class="card card--launch-app card--hero-editor">
          <div class="section-title-row section-title-row--launch">
            <h2>角色包编写器</h2>
            <HelpHint
              text="用来写包内核心性格档案与世界观、导出 zip；运行时的可变性格档案由 oclive 内模型维护，此处不可编辑。一般用浏览器打开官网就够；要改源码再选下面两种。"
            />
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">① 获取编写器</h3>
            <p class="hint tiny">复制 GitHub 上<strong>仓库首页</strong>的网址，在下面粘贴并点「填入」，即可同步 owner / repo（与「版本与下载」一致）。</p>
            <div class="gh-paste-block gh-paste-block--inline">
              <div class="label-with-hint">
                <span class="gh-paste-inline-label">粘贴仓库网址</span>
                <HelpHint :paragraphs="LAUNCHER_HINT_GH_URL_PASTE" />
              </div>
              <div class="row">
                <input
                  v-model="editorGhUrlPaste"
                  class="paste-url-input"
                  placeholder="https://github.com/…/…"
                  autocomplete="off"
                  @keydown.enter.prevent="applyEditorRepoFromPastedUrl"
                />
                <button type="button" class="btn" @click="applyEditorRepoFromPastedUrl">填入</button>
              </div>
            </div>
            <div class="gh-release-dl gh-release-dl--compact">
              <div class="label-with-hint gh-release-dl__label-row">
                <span class="gh-release-dl__label">GitHub Release（与「版本」里编写器仓库一致）</span>
                <HelpHint :paragraphs="LAUNCHER_HINT_EDITOR_GH_DL" />
              </div>
              <div class="row gh-release-dl-row">
                <button
                  type="button"
                  class="btn"
                  :disabled="editorGhBusy"
                  @click="refreshEditorGhAssets"
                >
                  列出附件
                </button>
                <select v-model="editorGhAssetUrl" class="gh-asset-select">
                  <option value="">选择文件…</option>
                  <option
                    v-for="a in editorGhAssets"
                    :key="a.browserDownloadUrl"
                    :value="a.browserDownloadUrl"
                  >
                    {{ a.name }} — {{ formatGhBytes(a.size) }}
                  </option>
                </select>
                <button
                  type="button"
                  class="btn primary"
                  :disabled="editorGhBusy || !editorGhAssetUrl"
                  @click="downloadEditorFromGh"
                >
                  下载到…
                </button>
              </div>
              <p class="hint tiny">另存为选路径；便携 zip 会解压并尝试填入 exe。</p>
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">② 打开方式</h3>
            <div class="mode mode--wrap">
              <label><input v-model="config.editorMode" type="radio" value="web" /> 网页</label>
              <label><input v-model="config.editorMode" type="radio" value="dev" /> 本地源码（npm）</label>
              <label><input v-model="config.editorMode" type="radio" value="exe" /> 本地 exe</label>
            </div>
            <template v-if="config.editorMode === 'web'">
              <label>网页地址（可空）</label>
              <input
                v-model="config.editorWebUrl"
                type="url"
                autocomplete="off"
                :placeholder="`不填：${editorPagesUrlPreview}`"
              />
              <p class="hint tiny">空则用线上 Pages；本地调试可填 <code>http://127.0.0.1:…</code>。</p>
            </template>
            <template v-else-if="config.editorMode === 'dev'">
              <label>源码根目录</label>
              <div class="row">
                <input v-model="config.editorProjectRoot" placeholder="例如 D:\oclive-pack-editor" />
                <button type="button" class="btn" @click="pickEditorRoot">浏览…</button>
              </div>
              <label>npm 脚本名</label>
              <input v-model="config.editorNpmScript" placeholder="tauri:dev" />
            </template>
            <template v-else>
              <div class="label-with-hint">
                <label>编写器 exe</label>
                <HelpHint :paragraphs="LAUNCHER_HINT_EXE_PATH_PASTE" />
              </div>
              <p class="hint tiny">粘贴以 .exe 结尾的完整路径后会自动切到「本地 exe」；也可点「识别框内路径」。</p>
              <div class="row">
                <input
                  v-model="config.editorExe"
                  placeholder="例如 …\oclive-pack-editor.exe（可粘贴）"
                  autocomplete="off"
                  @paste="onEditorExeInputPaste"
                />
                <button type="button" class="btn" @click="pickEditorExe">浏览…</button>
                <button type="button" class="btn" @click="applyEditorExeFromField">识别框内路径</button>
              </div>
            </template>
          </div>

          <div class="app-feature-block app-feature-block--run">
            <h3 class="app-feature-block__title">③ 运行</h3>
            <div class="actions actions--launch">
              <button type="button" class="btn primary btn-launch" @click="spawnEditor">
                {{ config.editorMode === 'web' ? '在浏览器打开' : '启动编写器' }}
              </button>
              <button type="button" class="btn danger btn-launch" @click="stopEditor">结束进程</button>
              <button type="button" class="btn btn-launch-secondary" @click="focusLogsFilter('editor')">
                在「日志」里只看编写器
              </button>
            </div>
          </div>
        </section>
      </div>

      <section class="card apps-log-preview-card">
        <div class="apps-log-preview-head">
          <h3 class="apps-log-preview-title">编写器输出摘要</h3>
          <div class="apps-log-preview-tools">
            <button type="button" class="btn" @click="focusLogsFilter('all')">打开完整日志页</button>
          </div>
        </div>
        <pre
          ref="appsLogPreviewEl"
          class="apps-log-preview"
          :class="{ 'apps-log-preview--empty': !editorLogPreviewText.trim() }"
        >{{ editorLogPreviewText || '尚无输出；启动编写器后这里会滚动显示最近几行。' }}</pre>
        <p class="hint tiny apps-log-preview-foot">
          完整历史与筛选在左侧「日志」。
        </p>
      </section>
    </div>

    <section v-else-if="activeNav === 'logs'" class="view-panel card log-card">
      <div class="log-head">
        <div class="section-title-row log-title-row">
          <h2>后台日志</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_LOGS" />
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
  min-height: 0;
  height: 100%;
  max-height: 100%;
  overflow: hidden;
  font-family: var(--fluent-font);
  color: var(--fluent-text-primary);
  /* 与全局 html 暖色底一致，避免主栏仍偏冷灰 */
  background:
    radial-gradient(120% 72% at 100% -5%, color-mix(in srgb, #b89a6e 14%, transparent), transparent 54%),
    radial-gradient(95% 58% at -8% 105%, color-mix(in srgb, #7d9c91 11%, transparent), transparent 50%),
    var(--fluent-bg-page);
  transition:
    background 0.22s ease,
    color 0.18s ease;
}

.rail {
  /* 宽度与图标方块（2.75rem）+ 左右对称内边距对齐 */
  width: calc(2.75rem + 1rem);
  box-sizing: border-box;
  flex-shrink: 0;
  align-self: stretch;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.45rem;
  padding: 1rem 0.5rem;
  border-right: 1px solid var(--fluent-border-stroke);
  background: color-mix(in srgb, var(--fluent-bg-card) 78%, transparent);
  backdrop-filter: blur(10px) saturate(110%);
  -webkit-backdrop-filter: blur(10px) saturate(110%);
  box-shadow:
    var(--fluent-shadow-soft),
    inset -1px 0 0 color-mix(in srgb, var(--fluent-border-stroke) 60%, transparent);
  /* 仅主栏滚动时，侧栏随视口高度始终铺满 */
  position: sticky;
  top: 0;
  max-height: 100%;
  overflow-y: auto;
}

.rail-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  max-width: 2.75rem;
  gap: 0.3rem;
  padding: 0.1rem 0;
  border: none;
  border-radius: 0;
  background: transparent;
  color: var(--fluent-text-secondary);
  cursor: pointer;
  font-size: 0.65rem;
  font-weight: 500;
  transition:
    color 0.15s ease,
    transform 0.12s ease;
}

.rail-btn:focus-visible {
  outline: none;
}

.rail-btn:focus-visible .rail-ico {
  box-shadow: 0 0 0 2px var(--fluent-border-focus);
}

.rail-ico {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  flex-shrink: 0;
  border-radius: var(--fluent-radius);
  font-size: 1.25rem;
  line-height: 1;
  transition:
    background 0.15s ease,
    color 0.15s ease,
    box-shadow 0.2s ease;
}

.rail-btn:hover .rail-ico {
  background: var(--fluent-bg-subtle);
  color: var(--fluent-text-primary);
}

.rail-btn.active .rail-ico {
  background: color-mix(in srgb, var(--fluent-accent-subtle) 70%, rgba(255, 255, 255, 0.12));
  color: var(--fluent-accent);
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--fluent-accent) 26%, transparent),
    0 0 10px color-mix(in srgb, var(--fluent-accent) 20%, transparent);
}

.rail-btn--accent-oclive.active .rail-ico {
  background: var(--rail-accent-oclive-bg);
  color: var(--rail-accent-oclive);
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--rail-accent-oclive) 26%, transparent),
    0 0 10px color-mix(in srgb, var(--rail-accent-oclive) 20%, transparent);
}

.rail-btn--accent-editor.active .rail-ico {
  background: var(--rail-accent-editor-bg);
  color: var(--rail-accent-editor);
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--rail-accent-editor) 28%, transparent),
    0 0 10px color-mix(in srgb, var(--rail-accent-editor) 18%, transparent);
}

.rail-btn.active {
  color: var(--fluent-text-primary);
}

.rail-lbl {
  line-height: 1.15;
  text-align: center;
  max-width: 2.75rem;
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
  background: color-mix(in srgb, var(--fluent-bg-card) 76%, transparent);
  backdrop-filter: blur(12px) saturate(105%);
  -webkit-backdrop-filter: blur(12px) saturate(105%);
  box-shadow: 0 1px 0 color-mix(in srgb, var(--fluent-border-stroke) 65%, transparent);
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

.titlebar-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 0.65rem;
}

.titlebar-tools-help {
  margin-left: 0.15rem;
  align-self: center;
}

/* 与编写器顶栏一致：A− / 百分比 / A+，再主题，再保存 */
.shell-scale {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.15rem 0.35rem;
  border-radius: var(--fluent-radius-lg);
  border: 1px solid var(--fluent-border-stroke);
  background: color-mix(in srgb, var(--fluent-bg-card) 72%, transparent);
  box-shadow: var(--fluent-shadow-soft);
}

.shell-scale-value {
  min-width: 2.75rem;
  text-align: center;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--fluent-text-secondary);
  font-variant-numeric: tabular-nums;
}

.shell-tool-btn {
  padding: 0.35rem 0.55rem;
  min-height: 30px;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: color-mix(in srgb, var(--fluent-bg-card) 82%, transparent);
  color: var(--fluent-text-primary);
  cursor: pointer;
  font-size: 0.78rem;
  font-weight: 500;
  font-family: var(--fluent-font);
  box-shadow: var(--fluent-shadow-soft);
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    transform 0.1s ease;
}

.shell-tool-btn:hover {
  background: var(--fluent-bg-subtle);
  border-color: var(--fluent-text-secondary);
}

.shell-tool-btn:focus-visible {
  outline: none;
  box-shadow:
    var(--fluent-shadow-soft),
    0 0 0 2px var(--fluent-bg-page),
    0 0 0 4px var(--fluent-border-focus);
}

.shell-tool-btn:active {
  transform: scale(0.98);
}

.shell-theme-btn {
  padding: 0.35rem 0.65rem;
}

.theme-toggle-btn {
  font-size: 0.8rem;
  padding: 0.4rem 0.65rem;
  min-height: 32px;
  border-color: var(--fluent-border-stroke);
  background: var(--fluent-bg-subtle);
}

.view-start-stack {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.titlebar h1 {
  margin: 0.15rem 0 0;
  font-size: 1.55rem;
  font-weight: 650;
  letter-spacing: -0.025em;
  line-height: 1.2;
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
  box-sizing: border-box;
  max-width: min(1100px, calc(100% - 2.7rem));
  width: 100%;
  margin: 0.4rem auto 0;
  padding: 0.5rem 1rem 0.55rem;
  font-size: 0.8125rem;
  line-height: 1.45;
  color: var(--fluent-text-primary);
  text-align: center;
  border-radius: var(--fluent-radius-lg);
  border: 1px solid color-mix(in srgb, var(--fluent-border-stroke) 75%, rgba(255, 255, 255, 0.25));
  background: color-mix(in srgb, var(--fluent-bg-card) 78%, transparent);
  backdrop-filter: blur(9px) saturate(105%);
  -webkit-backdrop-filter: blur(9px) saturate(105%);
  box-shadow:
    var(--fluent-shadow-soft),
    inset 0 1px 0 color-mix(in srgb, #fff 12%, transparent);
  animation: status-in 0.28s ease-out;
}

@keyframes status-in {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.scroll-main {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 1.15rem 1.35rem 2.25rem;
  scroll-padding-top: 0.75rem;
}

/* 新手页：自上而下提高叠放顺序，避免下方区块盖住上方问号 */
.view-start-stack .guide-card {
  position: relative;
  z-index: 3;
}

.view-start-stack .announce-board--creator {
  position: relative;
  z-index: 2;
}

.view-start-stack .announce-board--developer {
  position: relative;
  z-index: 1;
}

.scroll-main > .view-panel {
  max-width: 1100px;
  margin-left: auto;
  margin-right: auto;
  width: 100%;
}

.scroll-main > .view-panel.app-launch-page {
  max-width: 1220px;
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
  padding: 0.4rem 0.65rem;
  border-radius: var(--fluent-radius-lg);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-card);
  color: var(--fluent-text-primary);
  font-size: 0.78rem;
  font-weight: 500;
  cursor: pointer;
  box-shadow: var(--fluent-shadow-soft);
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    color 0.15s ease;
}

.mobile-nav-btn:hover {
  background: var(--fluent-bg-subtle);
  border-color: var(--fluent-border-control);
}

.mobile-nav-btn.active {
  border-color: color-mix(in srgb, var(--fluent-accent) 45%, var(--fluent-border-stroke));
  background: var(--fluent-accent-subtle);
  color: var(--fluent-accent);
  font-weight: 600;
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--fluent-accent) 28%, transparent),
    0 0 10px color-mix(in srgb, var(--fluent-accent) 16%, transparent);
}

@media (max-width: 900px) {
  .grid {
    grid-template-columns: 1fr;
  }
  .rail {
    display: none;
  }
  .titlebar-actions {
    width: 100%;
    justify-content: flex-start;
  }
  .mobile-nav {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    max-width: 1100px;
    width: 100%;
    margin: 0 auto;
    padding: 0.35rem 1.35rem 0.65rem;
    border-bottom: 1px solid var(--fluent-border-stroke);
    background: color-mix(in srgb, var(--fluent-bg-card) 82%, transparent);
    backdrop-filter: blur(8px) saturate(104%);
    -webkit-backdrop-filter: blur(8px) saturate(104%);
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
  background: color-mix(in srgb, var(--fluent-bg-card) 82%, transparent);
  backdrop-filter: blur(9px) saturate(106%);
  -webkit-backdrop-filter: blur(9px) saturate(106%);
  border: 1px solid var(--fluent-border-stroke);
  border-radius: var(--fluent-radius-lg);
  padding: 1rem 1.15rem;
  box-shadow: var(--fluent-shadow-card);
  transition:
    box-shadow 0.22s ease,
    border-color 0.22s ease;
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

.card--hero-oclive {
  border-left: 3px solid var(--rail-accent-oclive);
  box-shadow:
    var(--fluent-shadow-card),
    inset 0 1px 0 color-mix(in srgb, var(--rail-accent-oclive) 25%, transparent);
}

.card--hero-editor {
  border-left: 3px solid var(--rail-accent-editor);
  box-shadow:
    var(--fluent-shadow-card),
    inset 0 1px 0 color-mix(in srgb, var(--rail-accent-editor) 22%, transparent);
}

.app-launch-page--oclive .app-feature-block__title {
  color: var(--rail-accent-oclive);
}

.app-launch-page--editor .app-feature-block__title {
  color: var(--rail-accent-editor);
}

.apps-terminal-banner--editor {
  border-color: color-mix(in srgb, var(--rail-accent-editor) 38%, var(--fluent-border-stroke));
  background: color-mix(in srgb, var(--rail-accent-editor) 9%, var(--fluent-bg-card));
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

.gh-release-dl {
  margin: 0.65rem 0 0.75rem;
  padding: 0.55rem 0.65rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-subtle);
}

.gh-release-dl > label {
  display: block;
  font-size: 0.82rem;
  font-weight: 600;
  margin-bottom: 0.4rem;
  color: var(--fluent-text-primary);
}

.gh-release-dl__label-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-bottom: 0.4rem;
}

.gh-release-dl__label-row .gh-release-dl__label {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--fluent-text-primary);
}

/* 形成独立叠层，避免子元素 HelpHint(z-index:900) 与上方 .ver-quick-dl 等整块交错盖住 */
.gh-paste-block {
  margin: 0.5rem 0 0.75rem;
  position: relative;
  z-index: 0;
  isolation: isolate;
}

.gh-paste-block--inline {
  margin: 0.35rem 0 0.55rem;
}

.gh-paste-inline-label {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--fluent-text-secondary);
}

.paste-url-input {
  font-size: 0.8125rem;
}

.gh-release-dl-row {
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: 0.25rem;
}

.gh-asset-select {
  flex: 1;
  min-width: 160px;
  min-height: 32px;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-control);
  background: var(--fluent-bg-card);
  color: var(--fluent-text-primary);
  font-size: 0.82rem;
  padding: 0.25rem 0.35rem;
}

.gh-release-dl--compact {
  margin: 0;
  padding: 0.5rem 0.55rem;
}

.app-launch-page .apps-terminal-banner {
  margin: 0 0 1rem;
  padding: 0.65rem 0.85rem;
  border-radius: var(--fluent-radius-lg);
  border: 1px solid color-mix(in srgb, var(--fluent-accent) 35%, var(--fluent-border-stroke));
  background: color-mix(in srgb, var(--fluent-accent) 8%, var(--fluent-bg-card));
  font-size: 0.88rem;
  line-height: 1.55;
  color: var(--fluent-text-primary);
}

.apps-launch-stack {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  margin-bottom: 1.25rem;
}

.card--launch-app {
  padding: 1.2rem 1.35rem 1.35rem;
}

.section-title-row--launch h2 {
  font-size: 1.35rem;
  font-weight: 650;
  letter-spacing: -0.02em;
}

.app-feature-block {
  margin-bottom: 1.05rem;
  padding-bottom: 1.05rem;
  border-bottom: 1px solid var(--fluent-border-stroke);
}

.app-feature-block--run {
  border-bottom: none;
  margin-bottom: 0;
  padding-bottom: 0;
}

.app-feature-block__title {
  margin: 0 0 0.55rem;
  font-size: 0.78rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--fluent-accent);
}

.app-launch-page .roles-block--in-launch,
.app-launch-page .llm-backend-block--in-launch {
  border-bottom: none;
  margin-bottom: 0;
  padding-bottom: 0;
}

.btn-launch {
  min-height: 46px;
  padding: 0.55rem 1.35rem;
  font-size: 1.02rem;
  font-weight: 600;
}

.btn-launch-secondary {
  min-height: 46px;
  padding: 0.5rem 1rem;
  font-size: 0.9rem;
}

.actions--launch {
  margin-top: 0.35rem;
  gap: 0.55rem;
  align-items: center;
}

.apps-log-preview-card {
  margin-top: 0;
}

.apps-log-preview-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
  margin-bottom: 0.5rem;
}

.apps-log-preview-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.apps-log-preview {
  margin: 0;
  max-height: 180px;
  overflow: auto;
  padding: 0.55rem 0.65rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-subtle);
  font-family: ui-monospace, 'Cascadia Code', monospace;
  font-size: 0.72rem;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--fluent-text-primary);
}

.apps-log-preview--empty {
  color: var(--fluent-text-secondary);
  font-style: italic;
}

.apps-log-preview-foot {
  margin: 0.45rem 0 0;
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
    border-color 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.1s ease;
}

.btn:hover {
  background: var(--fluent-bg-subtle);
  border-color: var(--fluent-text-secondary);
}

.btn:focus-visible {
  outline: none;
  box-shadow:
    var(--fluent-shadow-soft),
    0 0 0 2px var(--fluent-bg-page),
    0 0 0 4px var(--fluent-border-focus);
}

.btn:active:not(:disabled) {
  transform: scale(0.985);
}

.btn.primary {
  background: var(--fluent-accent);
  color: #fff;
  border-color: transparent;
  box-shadow:
    var(--fluent-shadow-soft),
    0 1px 0 color-mix(in srgb, #fff 18%, transparent);
}

.btn.primary:hover {
  background: var(--fluent-accent-hover);
}

.btn.primary:focus-visible {
  box-shadow:
    var(--fluent-shadow-soft),
    0 0 0 2px var(--fluent-bg-page),
    0 0 0 4px var(--fluent-border-focus);
}

.btn:disabled {
  opacity: 0.52;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
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
  background: #1a1c22;
  color: #e4e6eb;
  border-radius: var(--fluent-radius-lg);
  border: 1px solid color-mix(in srgb, var(--fluent-border-stroke) 40%, #2a2d36);
  font-family: var(--fluent-mono);
  font-size: 0.72rem;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  box-shadow: inset 0 1px 0 color-mix(in srgb, #fff 6%, transparent);
}

:global(html[data-theme='dark']) .log {
  background: #0d0f14;
  color: #c8ccd4;
  border-color: #2a303c;
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

.diag-table tbody tr {
  transition: background 0.12s ease;
}

.diag-table tbody tr:hover {
  background: color-mix(in srgb, var(--fluent-bg-subtle) 72%, transparent);
}

.diag-table td {
  padding: 0.45rem 0;
  line-height: 1.4;
}

.diag-table td.ok {
  color: var(--fluent-success-text);
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

.guide-card {
  border-left: 3px solid var(--fluent-accent);
  padding: 1.15rem 1.25rem 1.2rem;
  box-shadow:
    var(--fluent-shadow-card),
    inset 0 1px 0 color-mix(in srgb, var(--fluent-accent) 14%, transparent);
}

.guide-card .guide-steps {
  margin: 1rem 0 0;
  padding-left: 1.15rem;
  list-style: disc;
  list-style-position: outside;
  line-height: 1.65;
  font-size: 0.9rem;
  color: var(--fluent-text-primary);
}

.guide-card .guide-steps li {
  margin-bottom: 0.7rem;
  padding-left: 0.2rem;
  text-wrap: pretty;
}

.guide-card .guide-steps li:last-child {
  margin-bottom: 0;
}

.guide-card .guide-steps li::marker {
  color: color-mix(in srgb, var(--fluent-text-secondary) 70%, var(--fluent-text-primary));
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
  font-size: 0.94rem;
  color: var(--fluent-text-primary);
  padding: 0.55rem 0.7rem;
  margin-bottom: 0.65rem;
  border-radius: var(--fluent-radius);
  border: 1px solid color-mix(in srgb, var(--fluent-border-stroke) 78%, rgba(255, 255, 255, 0.2));
  background: color-mix(in srgb, var(--fluent-bg-subtle) 78%, transparent);
  line-height: 1.55;
  text-wrap: pretty;
}

.ver-page .ver-page-lead {
  line-height: 1.55;
}

/* 去掉毛玻璃；整块高于 .gh-paste-block(0)，避免下方粘贴区问号与快捷入口叠层打架 */
.ver-quick-dl {
  margin: 0.85rem 0;
  padding: 0.85rem 1rem;
  border-radius: var(--fluent-radius-lg);
  border: 1px solid var(--fluent-border-stroke);
  border-left: 3px solid color-mix(in srgb, var(--fluent-text-secondary) 55%, var(--fluent-border-stroke));
  background: color-mix(in srgb, var(--fluent-bg-subtle) 88%, transparent);
  box-shadow: var(--fluent-shadow-soft);
  overflow: visible;
  position: relative;
  z-index: 2;
  isolation: isolate;
}

.ver-quick-head {
  margin-top: 0;
  margin-bottom: 0.35rem;
  position: relative;
  z-index: 2;
}

.ver-quick-head .ver-subtle-label {
  font-size: 0.78rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--fluent-text-secondary);
}

.ver-subtle-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--fluent-text-secondary);
}

.gh-row__label {
  margin-top: 0;
  margin-bottom: 0.35rem;
}

.gh-row__label label {
  font-size: 0.85rem;
}

.ver-compare-hint {
  margin-top: 0.15rem;
  margin-bottom: 0.35rem;
}

.ver-actions-wrap {
  margin-top: 0.35rem;
}

.ver-actions-hint {
  margin-top: 0.35rem;
  margin-bottom: 0.35rem;
}

.ver-quick-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
  position: relative;
  z-index: 0;
}

.ver-actions-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.55rem;
  margin-top: 0.85rem;
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
  background: color-mix(in srgb, var(--fluent-text-primary) 38%, transparent);
  backdrop-filter: blur(5px);
  -webkit-backdrop-filter: blur(5px);
}

.install-modal-panel {
  width: min(440px, 100%);
  max-height: min(90vh, 640px);
  overflow: auto;
  padding: 1.15rem 1.3rem;
  border-radius: var(--fluent-radius-xl);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-card);
  box-shadow:
    var(--fluent-shadow-card),
    0 24px 48px color-mix(in srgb, var(--fluent-text-primary) 12%, transparent);
  animation: modal-in 0.22s ease-out;
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
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
