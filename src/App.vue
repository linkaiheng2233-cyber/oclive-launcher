<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from "vue-i18n";
import { setAppLocale, type AppLocale } from "./i18n";
import HelpHint from './components/HelpHint.vue'
import CreatorAnnouncementsSection from './announcements/CreatorAnnouncementsSection.vue'
import DeveloperAnnouncementsSection from './announcements/DeveloperAnnouncementsSection.vue'
import { useDeveloperAnnouncements } from './announcements/useDeveloperAnnouncements'
import { useLauncherBootstrap } from './composables/useLauncherBootstrap'
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

const { t } = useI18n()

const uiLocale = ref<AppLocale>("system");
function onLocaleChange(v: string) {
  const next = (v as AppLocale) || "system";
  uiLocale.value = next;
  setAppLocale(next);
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
    { value: DEFAULT_OLLAMA_MODEL, label: String(t("launcher.ollama.modelOptions.recommendedDefault", { model: DEFAULT_OLLAMA_MODEL })) },
  ]
  const fromHost = ollamaLocalModels.value.filter((m) => m !== DEFAULT_OLLAMA_MODEL)
  const seen = new Set(opts.map((o) => o.value))
  for (const m of fromHost) {
    if (seen.has(m)) continue
    seen.add(m)
    opts.push({ value: m, label: m })
  }
  opts.push({ value: MODEL_OPTION_CUSTOM, label: String(t("launcher.ollama.modelOptions.custom")) })
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

const { loadAll, scheduleFirstLaunchDiagnose } = useLauncherBootstrap({
  config,
  setStatus: (msg) => {
    statusMsg.value = msg
  },
  loadConfig: () => invoke<LauncherConfig>('load_config'),
  refreshRolePackEchoUi,
  reloadDevAnnounceFromDisk,
  refreshLocalVersions,
  refreshWingetAvailability,
  refreshBundledOllamaInfo,
  runEnvironmentDiagnose,
})

async function saveConfig() {
  try {
    await invoke('save_config', { config: config.value })
    statusMsg.value = String(t("launcher.status.configSaved"))
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
    statusMsg.value = String(t("launcher.status.githubRepoUnrecognized"))
    return
  }
  config.value.githubEditorOwner = r.owner
  config.value.githubEditorRepo = r.repo
  statusMsg.value = String(
    t("launcher.status.githubEditorRepoApplied", { owner: r.owner, repo: r.repo }),
  )
  await saveConfig()
}

async function applyOcliveRepoFromPastedUrl() {
  const r = parseGithubRepoFromUrl(ocliveGhUrlPaste.value)
  if (!r) {
    statusMsg.value = String(t("launcher.status.githubRepoUnrecognized"))
    return
  }
  config.value.githubOcliveOwner = r.owner
  config.value.githubOcliveRepo = r.repo
  statusMsg.value = String(
    t("launcher.status.githubOcliveRepoApplied", { owner: r.owner, repo: r.repo }),
  )
  await saveConfig()
}

function onOcliveExeInputPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  const p = normalizeExePathPaste(text)
  if (!p) return
  e.preventDefault()
  config.value.ocliveExe = p
  config.value.ocliveMode = 'exe'
  statusMsg.value = String(t("launcher.status.ocliveExeRecognizedFromPaste"))
  void saveConfig()
}

function onEditorExeInputPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  const p = normalizeExePathPaste(text)
  if (!p) return
  e.preventDefault()
  config.value.editorExe = p
  config.value.editorMode = 'exe'
  statusMsg.value = String(t("launcher.status.editorExeRecognizedFromPaste"))
  void saveConfig()
}

async function applyOcliveExeFromField() {
  const p = normalizeExePathPaste(config.value.ocliveExe)
  if (!p) {
    statusMsg.value = String(t("launcher.status.exePathInvalid"))
    return
  }
  config.value.ocliveExe = p
  config.value.ocliveMode = 'exe'
  statusMsg.value = String(t("launcher.status.ocliveExeNormalized"))
  await saveConfig()
}

async function applyEditorExeFromField() {
  const p = normalizeExePathPaste(config.value.editorExe)
  if (!p) {
    statusMsg.value = String(t("launcher.status.exePathInvalid"))
    return
  }
  config.value.editorExe = p
  config.value.editorMode = 'exe'
  statusMsg.value = String(t("launcher.status.editorExeNormalized"))
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
      ? String(t("launcher.status.ghAssetsListedOclive", { n: ocliveGhAssets.value.length }))
      : String(t("launcher.status.ghAssetsNone"))
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
      ? String(t("launcher.status.ghAssetsListedEditor", { n: editorGhAssets.value.length }))
      : String(t("launcher.status.ghAssetsNone"))
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
    statusMsg.value = String(t("launcher.status.pickAssetFirst"))
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
      statusMsg.value = String(t("launcher.status.ocliveDownloadedAndConfigured"))
    } else {
      statusMsg.value = r.hint
        ? String(t('launcher.status.savedHintAndPath', { hint: r.hint, savedPath: r.savedPath }))
        : String(t('launcher.status.savedPathOnly', { savedPath: r.savedPath }))
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
    statusMsg.value = String(t("launcher.status.pickAssetFirst"))
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
      statusMsg.value = String(t("launcher.status.editorDownloadedAndConfigured"))
    } else {
      statusMsg.value = r.hint
        ? String(t('launcher.status.savedHintAndPath', { hint: r.hint, savedPath: r.savedPath }))
        : String(t('launcher.status.savedPathOnly', { savedPath: r.savedPath }))
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
      statusMsg.value = String(t("launcher.status.rolesDirSuggestedFilled"))
    } else {
      statusMsg.value = String(t("launcher.status.rolesDirSuggestedNotFound"))
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
      statusMsg.value = String(t("launcher.status.rolesRootMissing"))
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
      statusMsg.value = String(t("launcher.status.ollamaLocalModelsListFailed"))
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
    statusMsg.value = String(t("launcher.status.ollamaModelMissing"))
    return
  }
  const zip = pendingZipPath.value
  const root = config.value.ocliveRolesDir?.trim()
  if (!zip || !root) {
    statusMsg.value = String(t("launcher.status.installMissingZipOrRoot"))
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
    statusMsg.value = String(t("launcher.status.rolePackInstalled", { roleId }))
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
      String(t("launcher.confirms.launchBundledOllamaInstaller")),
    )
  ) {
    return
  }
  try {
    await invoke('launch_bundled_ollama_installer')
    statusMsg.value = String(t("launcher.status.bundledOllamaInstallerLaunched"))
    focusLogsFilter('bundled-ollama')
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function installOllamaViaWinget() {
  if (
    !confirm(
      String(t("launcher.confirms.installOllamaViaWinget")),
    )
  ) {
    return
  }
  wingetInstallBusy.value = true
  try {
    await invoke('install_ollama_via_winget')
    statusMsg.value = String(t("launcher.status.wingetInstallStarted"))
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
    statusMsg.value = String(
      t("launcher.status.ollamaPullStarted", { model: DEFAULT_OLLAMA_MODEL }),
    )
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
    statusMsg.value = String(t("launcher.status.ollamaPullModelMissing"))
    return
  }
  pullBusy.value = true
  try {
    await invoke('ollama_pull_model', { model })
    statusMsg.value = String(t("launcher.status.ollamaPullStartedWithRefreshHint", { model }))
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
    statusMsg.value = String(t("launcher.status.ollamaLocalModelsRefreshed"))
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function spawnEditor() {
  try {
    await invoke('spawn_managed_app', { kind: 'editor', config: config.value })
    if (config.value.editorMode === 'web') {
      statusMsg.value = String(t("launcher.status.editorOpenedInBrowser"))
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
    statusMsg.value = String(t("launcher.status.remoteVersionsChecked"))
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
    if (!quiet) statusMsg.value = String(t("launcher.status.envDiagnoseDone"))
  } catch (e) {
    envDiagErr.value = String(e)
    envDiag.value = null
    if (!quiet) statusMsg.value = String(e)
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
    statusMsg.value = String(t("launcher.status.versionsListingOpened"))
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
      statusMsg.value = String(t("launcher.status.githubReposSyncedFromPaste"))
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
      String(t("launcher.confirms.resetConfigToDefault")),
    )
  )
    return
  try {
    const c = await invoke<LauncherConfig>('reset_config_to_default')
    config.value = { ...config.value, ...c }
    envDiag.value = null
    statusMsg.value = String(t("launcher.status.configResetToDefault"))
    await refreshLocalVersions()
  } catch (e) {
    statusMsg.value = String(e)
  }
}

async function openLauncherConfigFolder() {
  try {
    await invoke('open_config_directory')
    statusMsg.value = String(t("launcher.status.configDirectoryOpened"))
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
  { id: 'start', label: String(t("launcher.nav.start")), icon: '🚀' },
  { id: 'version', label: String(t("launcher.nav.version")), icon: '📦' },
  { id: 'launch-oclive', label: String(t("launcher.nav.launchOclive")), icon: '💬', accent: 'oclive' },
  { id: 'launch-editor', label: String(t("launcher.nav.launchEditor")), icon: '✏️', accent: 'editor' },
  { id: 'assistant', label: String(t("launcher.nav.assistant")), icon: '🩺' },
  { id: 'logs', label: String(t("launcher.nav.logs")), icon: '📋' },
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
      return String(t('launcher.titlebar.theme.light'))
    case 'dark':
      return String(t('launcher.titlebar.theme.dark'))
    default:
      return String(t('launcher.titlebar.theme.system'))
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

const currentViewLabel = computed(() => {
  const id = activeNav.value
  if (id === 'start') return String(t('launcher.views.start'))
  if (id === 'version') return String(t('launcher.views.version'))
  if (id === 'launch-oclive') return String(t('launcher.views.launchOclive'))
  if (id === 'launch-editor') return String(t('launcher.views.launchEditor'))
  if (id === 'assistant') return String(t('launcher.views.assistant'))
  if (id === 'logs') return String(t('launcher.views.logs'))
  return ''
})

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
  unlistenLog = await listen<{ app: string; stream: string; line: string }>(
    'launcher-log',
    (e) => {
      const p = e.payload
      pushLog(p.app, p.stream, p.line)
    },
  )
  unlistenExit = await listen<{ app: string; code: number | null }>('launcher-exit', (e) => {
    const p = e.payload
    pushLog(p.app, 'out', String(t('launcher.logs.processExited', { code: p.code ?? '?' })))
  })
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
  scheduleFirstLaunchDiagnose()
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
    <aside class="rail" :aria-label="String(t('launcher.nav.railAria'))">
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
            <p class="kicker">{{ t("launcher.titlebar.kicker") }}</p>
            <h1>{{ currentViewLabel }}</h1>
            <p class="sub">
              <template v-if="activeNav === 'start'">
                {{ t("launcher.viewSub.start") }}
              </template>
              <template v-else-if="activeNav === 'version'">
                {{ t("launcher.viewSub.version") }}
              </template>
              <template v-else-if="activeNav === 'launch-oclive'">
                {{ t("launcher.viewSub.launchOclive") }}
              </template>
              <template v-else-if="activeNav === 'launch-editor'">
                {{ t("launcher.viewSub.launchEditor") }}
              </template>
              <template v-else-if="activeNav === 'assistant'">
                {{ t("launcher.viewSub.assistant") }}
              </template>
              <template v-else>{{ t("launcher.viewSub.other") }}</template>
            </p>
          </div>
          <div class="titlebar-actions" role="toolbar" :aria-label="String(t('launcher.titlebar.toolsAria'))">
            <label class="shell-locale">
              <span class="sr-only">{{ t("common.language") }}</span>
              <select
                class="shell-locale-select"
                :value="uiLocale"
                @change="onLocaleChange(($event.target as HTMLSelectElement).value)"
              >
                <option value="system">{{ t("common.system") }}</option>
                <option value="zh-CN">{{ t("common.zhCN") }}</option>
                <option value="en-US">{{ t("common.enUS") }}</option>
              </select>
            </label>
            <div class="shell-scale" :aria-label="String(t('launcher.titlebar.scaleAria'))">
              <button type="button" class="shell-tool-btn" :title="String(t('launcher.titlebar.shrink'))" :aria-label="String(t('launcher.titlebar.shrinkAria'))" @click="bumpScale(-1)">
                A−
              </button>
              <span class="shell-scale-value" :title="String(t('launcher.titlebar.relativeScaleTitle', { label: scaleLabel }))">{{ scaleLabel }}</span>
              <button type="button" class="shell-tool-btn" :title="String(t('launcher.titlebar.enlarge'))" :aria-label="String(t('launcher.titlebar.enlargeAria'))" @click="bumpScale(1)">
                A+
              </button>
            </div>
            <button
              type="button"
              class="shell-tool-btn shell-theme-btn"
              :title="String(t('launcher.titlebar.themeTitle', { label: themeCycleLabel }))"
              @click="cycleTheme"
            >
              {{ themePreference === 'system' ? '◐' : themePreference === 'dark' ? '🌙' : '☀️' }}
              {{ themeCycleLabel }}
            </button>
            <button
              type="button"
              class="btn primary"
              :title="String(t('launcher.titlebar.saveConfigTitle'))"
              @click="saveConfig"
            >
              {{ t("launcher.titlebar.saveConfig") }}
            </button>
            <HelpHint class="titlebar-tools-help" :paragraphs="LAUNCHER_HINT_TITLEBAR_TOOLS" />
          </div>
        </div>
      </header>

      <nav class="mobile-nav" :aria-label="String(t('launcher.nav.mobileAria'))">
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
          <h2>{{ t("launcher.startGuide.title") }}</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_START_GUIDE" />
        </div>
        <p class="hint guide-lead">{{ t("launcher.startGuide.lead") }}</p>
        <p class="hint">{{ t("launcher.startGuide.desc") }}</p>
        <ol class="guide-steps">
          <li>
            <strong>{{ t("launcher.startGuide.steps.env.strong") }}</strong>{{ t("launcher.startGuide.steps.env.colon") }}
            {{ t("launcher.startGuide.steps.env.devPrefix") }} <strong>Node</strong>{{ t("launcher.startGuide.steps.env.devSuffix") }}
            {{ t("launcher.startGuide.steps.env.localPrefix") }} <strong>Ollama</strong>{{ t("launcher.startGuide.steps.env.localSuffix") }}
          </li>
          <li>
            <strong>{{ t("launcher.startGuide.steps.download.strong") }}</strong>{{ t("launcher.startGuide.steps.download.colon") }}
            {{ t("launcher.startGuide.steps.download.prefix") }}<strong>{{ t("launcher.startGuide.steps.download.versionsStrong") }}</strong>{{ t("launcher.startGuide.steps.download.suffix") }}
          </li>
          <li>
            <strong>{{ t("launcher.startGuide.steps.paths.strong") }}</strong>{{ t("launcher.startGuide.steps.paths.colon") }}
            {{ t("launcher.startGuide.steps.paths.text") }}
          </li>
          <li>
            <strong>{{ t("launcher.startGuide.steps.roles.strong") }}</strong>{{ t("launcher.startGuide.steps.roles.colon") }}
            {{ t("launcher.startGuide.steps.roles.text") }}
          </li>
          <li>
            <strong>{{ t("launcher.startGuide.steps.chat.strong") }}</strong>{{ t("launcher.startGuide.steps.chat.colon") }}
            {{ t("launcher.startGuide.steps.chat.text") }}
          </li>
        </ol>
        <p class="hint guide-links">
          <button type="button" class="linkish" @click="openRelease('https://nodejs.org/')">Node.js</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/download')">{{ t("launcher.startGuide.links.ollamaDownload") }}</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/library')">{{ t("launcher.startGuide.links.ollamaLibrary") }}</button>
          ·
          <button type="button" class="linkish" @click="openRelease(releasesEditorUrl)">{{ t("launcher.startGuide.links.editorReleases") }}</button>
          ·
          <button type="button" class="linkish" @click="openRelease(releasesOcliveUrl)">{{ t("launcher.startGuide.links.ocliveReleases") }}</button>
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
          <h2>{{ t("launcher.versionPage.title") }}</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_PAGE" />
        </div>
        <p class="hint ver-page-lead">{{ t("launcher.versionPage.lead") }}</p>
        <div class="ver-quick-dl">
          <div class="label-with-hint ver-quick-head">
            <span class="ver-subtle-label">{{ t("launcher.versionPage.quickLinks") }}</span>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_QUICK_LINKS" />
          </div>
          <div class="ver-quick-btns">
            <button type="button" class="btn" @click="openVersionsListingInBrowser">{{ t("launcher.versionPage.buttons.versionsListing") }}</button>
            <button type="button" class="btn" @click="openRelease(releasesEditorUrl)">{{ t("launcher.versionPage.buttons.editorReleases") }}</button>
            <button type="button" class="btn" @click="openRelease(releasesOcliveUrl)">{{ t("launcher.versionPage.buttons.ocliveReleases") }}</button>
          </div>
        </div>
        <p class="hint">
          {{ t("launcher.versionPage.repoHelp.prefix") }}<strong>{{ t("launcher.versionPage.repoHelp.editorStrong") }}</strong>{{ t("launcher.versionPage.repoHelp.middle") }}<strong>oclive</strong>{{ t("launcher.versionPage.repoHelp.suffix") }}
        </p>

        <div class="gh-paste-block">
          <div class="label-with-hint">
            <label>{{ t("launcher.versionPage.editorRepoPaste.label") }}</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_GH_URL_PASTE" />
          </div>
          <div class="row">
            <input
              v-model="editorGhUrlPaste"
              class="paste-url-input"
              :placeholder="String(t('launcher.versionPage.editorRepoPaste.placeholder'))"
              autocomplete="off"
              @keydown.enter.prevent="applyEditorRepoFromPastedUrl"
            />
            <button type="button" class="btn" @click="applyEditorRepoFromPastedUrl">{{ t("launcher.versionPage.editorRepoPaste.apply") }}</button>
          </div>
        </div>

        <div class="gh-row">
          <div class="label-with-hint gh-row__label">
            <label>{{ t("launcher.versionPage.editorRepoLabel") }}</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_REPO_EDITOR" />
          </div>
          <div class="gh-inputs">
            <input v-model="config.githubEditorOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubEditorRepo" placeholder="repo" />
          </div>
        </div>
        <div class="label-with-hint ver-compare-hint">
          <span class="ver-subtle-label">{{ t("launcher.versionPage.compareLabel") }}</span>
          <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_LOCAL_VS_REMOTE" />
        </div>
        <div class="ver-line">
          <span>{{ t("launcher.versionPage.localVersion") }}</span>
          <strong>{{ editorLocalVer ?? '—' }}</strong>
        </div>
        <div class="ver-line" v-if="editorRemote">
          <span>{{ t("launcher.versionPage.remoteVersion") }}</span>
          <strong>{{ editorRemote.tagName }}</strong>
          <button type="button" class="btn tiny" @click="openRelease(editorRemote.htmlUrl)">
            {{ t("launcher.versionPage.openRelease") }}
          </button>
        </div>

        <hr class="sep" />

        <div class="gh-paste-block">
          <div class="label-with-hint">
            <label>{{ t("launcher.versionPage.ocliveRepoPasteLabel") }}</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_GH_URL_PASTE" />
          </div>
          <div class="row">
            <input
              v-model="ocliveGhUrlPaste"
              class="paste-url-input"
              :placeholder="String(t('launcher.versionPage.ocliveRepoPastePlaceholder'))"
              autocomplete="off"
              @keydown.enter.prevent="applyOcliveRepoFromPastedUrl"
            />
            <button type="button" class="btn" @click="applyOcliveRepoFromPastedUrl">{{ t("launcher.versionPage.applyOwnerRepo") }}</button>
          </div>
        </div>

        <div class="gh-row">
          <div class="label-with-hint gh-row__label">
            <label>{{ t("launcher.versionPage.ocliveRepoLabel") }}</label>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_REPO_OCLIVE" />
          </div>
          <div class="gh-inputs">
            <input v-model="config.githubOcliveOwner" placeholder="owner" />
            <span>/</span>
            <input v-model="config.githubOcliveRepo" placeholder="repo" />
          </div>
        </div>
        <div class="label-with-hint ver-compare-hint">
          <span class="ver-subtle-label">{{ t("launcher.versionPage.compareLabel") }}</span>
          <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_LOCAL_VS_REMOTE" />
        </div>
        <div class="ver-line">
          <span>{{ t("launcher.versionPage.localVersion") }}</span>
          <strong>{{ ocliveLocalVer ?? '—' }}</strong>
        </div>
        <div class="ver-line" v-if="ocliveRemote">
          <span>{{ t("launcher.versionPage.remoteVersion") }}</span>
          <strong>{{ ocliveRemote.tagName }}</strong>
          <button type="button" class="btn tiny" @click="openRelease(ocliveRemote.htmlUrl)">
            {{ t("launcher.versionPage.openRelease") }}
          </button>
        </div>

        <p v-if="checkErr" class="err">{{ checkErr }}</p>
        <div class="ver-actions-wrap">
          <div class="label-with-hint ver-actions-hint">
            <span class="ver-subtle-label">{{ t("launcher.versionPage.checkUpdatesLabel") }}</span>
            <HelpHint :paragraphs="LAUNCHER_HINT_VERSION_ACTIONS" />
          </div>
          <div class="ver-actions-row">
            <button type="button" class="btn primary" @click="syncGithubUrlsAndCheckUpdates">
              {{ t("launcher.versionPage.syncAndCheck") }}
            </button>
            <button type="button" class="btn" @click="checkReleases">{{ t("launcher.versionPage.checkOnly") }}</button>
          </div>
        </div>
      </section>

    <section v-else-if="activeNav === 'assistant'" class="view-panel card">
        <div class="section-title-row">
          <h2>{{ t("launcher.assistant.title") }}</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_ASSISTANT" />
        </div>
        <div v-if="envDiag && nodeNeedsAttention" class="banner-warn banner-node" role="status">
          <strong>{{ t("launcher.assistant.banners.nodeStrong") }}</strong>{{ t("launcher.assistant.banners.nodeText") }}
          <button type="button" class="btn tiny" @click="openRelease('https://nodejs.org/')">{{ t("launcher.assistant.banners.getNode") }}</button>
        </div>
        <div v-if="envDiag && ollamaNeedsAttention" class="banner-warn" role="status">
          <strong>{{ t("launcher.assistant.banners.ollamaStrong") }}</strong>{{ t("launcher.assistant.banners.ollamaText") }}
          <button type="button" class="btn tiny" @click="openRelease('https://ollama.com/download')">{{ t("launcher.assistant.banners.getOllama") }}</button>
          <button
            v-if="wingetAvailable"
            type="button"
            class="btn tiny primary"
            :disabled="wingetInstallBusy"
            @click="installOllamaViaWinget"
          >
            {{ t("launcher.assistant.banners.wingetInstall") }}
          </button>
          <button
            v-if="bundledOllamaPath"
            type="button"
            class="btn tiny"
            :disabled="wingetInstallBusy"
            @click="launchBundledOllamaInstaller"
          >
            {{ t("launcher.assistant.banners.runBundledInstaller") }}
          </button>
        </div>
        <div v-if="config.ocliveLlmMode === 'remote' && envDiag" class="banner-hint-remote" role="note">
          {{ t("launcher.assistant.remoteHintPrefix") }}<strong>{{ t("launcher.assistant.remoteHintStrong") }}</strong>{{ t("launcher.assistant.remoteHintSuffix") }}
        </div>

        <div class="assistant-actions">
          <button type="button" class="btn primary" @click="() => runEnvironmentDiagnose()">{{ t("launcher.assistant.actions.rerunDiagnose") }}</button>
          <button type="button" class="btn" @click="openLauncherConfigFolder">{{ t("launcher.assistant.actions.openConfigDir") }}</button>
          <button type="button" class="btn danger" @click="resetLauncherConfig">{{ t("launcher.assistant.actions.resetConfig") }}</button>
        </div>
        <p v-if="envDiagErr" class="err">{{ envDiagErr }}</p>
        <p class="hint">
          {{ t("launcher.assistant.tableHintPrefix") }}<code>launcher-config.json.corrupt.bak</code>{{ t("launcher.assistant.tableHintSuffix") }}
        </p>
        <table v-if="envDiag" class="diag-table">
          <tbody>
            <tr>
              <th>Node.js</th>
              <td :class="{ ok: !!envDiag.nodeVersion, bad: !envDiag.nodeVersion }">
                {{ envDiag.nodeVersion ?? t("launcher.assistant.fallbacks.nodeMissing") }}
              </td>
            </tr>
            <tr>
              <th>npm</th>
              <td :class="{ ok: !!envDiag.npmVersion, bad: !envDiag.npmVersion }">
                {{ envDiag.npmVersion ?? t("launcher.assistant.fallbacks.npmMissing") }}
              </td>
            </tr>
            <tr>
              <th>{{ t("launcher.assistant.tableHeaders.ollamaCli") }}</th>
              <td :class="{ ok: !!envDiag.ollamaVersion, bad: !envDiag.ollamaVersion }">
                {{ envDiag.ollamaVersion ?? t("launcher.assistant.fallbacks.ollamaCliMissing") }}
              </td>
            </tr>
            <tr>
              <th>{{ t("launcher.assistant.tableHeaders.ollamaService") }}</th>
              <td :class="{ ok: envDiag.ollamaApiReachable, bad: !envDiag.ollamaApiReachable }">
                {{
                  envDiag.ollamaApiReachable
                    ? t("launcher.assistant.fallbacks.ollamaApiOk")
                    : t("launcher.assistant.fallbacks.ollamaApiBad")
                }}
              </td>
            </tr>
            <tr>
              <th>{{ t("launcher.assistant.tableHeaders.editorProject") }}</th>
              <td :class="{ ok: envDiag.editorProjectOk && envDiag.editorPackageJson, bad: !envDiag.editorProjectOk }">
                <template v-if="config.editorMode === 'web'">{{ t("launcher.assistant.projects.editor.webMode") }}</template>
                <template v-else-if="!config.editorProjectRoot?.trim()">{{ t("launcher.assistant.projects.editor.missingPath") }}</template>
                <template v-else-if="!envDiag.editorProjectOk">{{ t("launcher.assistant.projects.editor.badPath") }}</template>
                <template v-else-if="!envDiag.editorPackageJson">{{ t("launcher.assistant.projects.editor.missingPkgJson") }}</template>
                <template v-else>{{ t("launcher.assistant.projects.ok") }}</template>
              </td>
            </tr>
            <tr>
              <th>{{ t("launcher.assistant.tableHeaders.ocliveProject") }}</th>
              <td :class="{ ok: envDiag.ocliveProjectOk && envDiag.oclivePackageJson, bad: !envDiag.ocliveProjectOk }">
                <template v-if="!config.ocliveProjectRoot?.trim()">{{ t("launcher.assistant.projects.oclive.missingPath") }}</template>
                <template v-else-if="!envDiag.ocliveProjectOk">{{ t("launcher.assistant.projects.oclive.badPath") }}</template>
                <template v-else-if="!envDiag.oclivePackageJson">{{ t("launcher.assistant.projects.oclive.missingPkgJson") }}</template>
                <template v-else>{{ t("launcher.assistant.projects.ok") }}</template>
              </td>
            </tr>
            <tr>
              <th>{{ t("launcher.assistant.tableHeaders.rolesDir") }}</th>
              <td
                :class="{
                  ok: envDiag.ocliveRolesDirOk,
                  bad: !!config.ocliveRolesDir?.trim() && !envDiag.ocliveRolesDirOk,
                }"
              >
                <template v-if="!config.ocliveRolesDir?.trim()">{{ t("launcher.assistant.rolesDir.missingOk") }}</template>
                <template v-else-if="!envDiag.ocliveRolesDirOk">{{ t("launcher.assistant.rolesDir.badPath") }}</template>
                <template v-else-if="!envDiag.ocliveRolesDirHasRoleHint">{{ t("launcher.assistant.rolesDir.noRolesYet") }}</template>
                <template v-else>{{ t("launcher.assistant.rolesDir.looksOk") }}</template>
              </td>
            </tr>
          </tbody>
        </table>

        <details class="ollama-details">
          <summary>{{ t("launcher.assistant.ollamaDetails.summary") }}</summary>
          <ul class="ollama-details-list">
            <li>
              <strong>{{ t("launcher.assistant.ollamaDetails.installStrong") }}</strong>{{ t("launcher.assistant.ollamaDetails.colon") }}
              {{ t("launcher.assistant.ollamaDetails.installPrefix") }}
              <button type="button" class="linkish inline" @click="openRelease('https://ollama.com/download')">ollama.com</button>
              {{ t("launcher.assistant.ollamaDetails.installSuffix") }}
            </li>
            <li>
              <strong>{{ t("launcher.assistant.ollamaDetails.modelStrong") }}</strong>{{ t("launcher.assistant.ollamaDetails.colon") }}
              {{ t("launcher.assistant.ollamaDetails.modelPrefix") }}
              <code>ollama pull {{ t("launcher.assistant.ollamaDetails.modelNamePlaceholder") }}</code>{{ t("launcher.assistant.ollamaDetails.modelMiddle") }}
              <button type="button" class="linkish inline" @click="openRelease('https://ollama.com/library')">{{ t("launcher.assistant.ollamaDetails.modelLibrary") }}</button>{{ t("launcher.assistant.ollamaDetails.modelSuffix") }}
              <code>{{ DEFAULT_OLLAMA_MODEL }}</code>{{ t("launcher.assistant.ollamaDetails.punctuationFullStop") }}
            </li>
            <li>
              <strong>{{ t("launcher.assistant.ollamaDetails.remoteStrong") }}</strong>{{ t("launcher.assistant.ollamaDetails.colon") }}
              {{ t("launcher.assistant.ollamaDetails.remotePrefix") }}<code>ollama pull</code>{{ t("launcher.assistant.ollamaDetails.remoteMiddle") }}
              <code>REMOTE_PLUGIN_PROTOCOL.md</code>{{ t("launcher.assistant.ollamaDetails.punctuationFullStop") }}
            </li>
          </ul>
        </details>

        <div class="ollama-model-box">
          <div class="section-title-row section-title-row--inline">
            <strong>{{ t("launcher.assistant.ollamaQuick.title") }}</strong>
            <HelpHint :text="String(t('launcher.assistant.ollamaQuick.help'))" />
          </div>
          <div class="env-ollama-quick">
            <span class="env-ollama-quick-label">{{ t("launcher.assistant.ollamaQuick.tapOnce") }}</span>
            <div class="env-ollama-quick-btns">
              <button
                v-if="bundledOllamaPath"
                type="button"
                class="btn"
                :disabled="wingetInstallBusy || pullBusy"
                @click="launchBundledOllamaInstaller"
              >
                {{ t("launcher.assistant.ollamaQuick.runBundledInstaller") }}
              </button>
              <button
                v-if="wingetAvailable"
                type="button"
                class="btn"
                :disabled="wingetInstallBusy || pullBusy"
                @click="installOllamaViaWinget"
              >
                {{ t("launcher.assistant.ollamaQuick.installViaWinget") }}
              </button>
              <button
                type="button"
                class="btn"
                :disabled="pullBusy || wingetInstallBusy"
                @click="pullRecommendedOllamaModel"
              >
                {{ t("launcher.assistant.ollamaQuick.pullRecommended", { model: DEFAULT_OLLAMA_MODEL }) }}
              </button>
              <button
                type="button"
                class="btn"
                :disabled="pullBusy || wingetInstallBusy"
                @click="refreshOllamaLocalModelsList"
              >
                {{ t("launcher.assistant.ollamaQuick.refreshLocalList") }}
              </button>
            </div>
            <p v-if="ollamaLocalModels.length" class="hint tiny env-ollama-model-chips">
              {{ t("launcher.assistant.ollamaQuick.localModelsPrefix") }}<code>{{ ollamaLocalModels.join('、') }}</code>
            </p>
            <p v-else class="hint tiny env-ollama-model-chips">
              {{ t("launcher.assistant.ollamaQuick.localModelsEmpty") }}
            </p>
          </div>
        </div>

        <p class="hint assistant-links">
          <button type="button" class="linkish" @click="openRelease('https://nodejs.org/')">{{ t("launcher.assistant.links.nodeDownload") }}</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://ollama.com/download')">{{ t("launcher.assistant.links.ollamaDownload") }}</button>
          ·
          <button type="button" class="linkish" @click="openRelease('https://github.com/ollama/ollama')">{{ t("launcher.assistant.links.ollamaDocs") }}</button>
        </p>
      </section>

    <div v-else-if="activeNav === 'launch-oclive'" class="view-panel app-launch-page app-launch-page--oclive">
      <p class="apps-terminal-banner">
        {{ t("launcher.launchOclive.banner.prefix") }} <strong>oclive</strong>{{ t("launcher.launchOclive.banner.middle") }}
        <strong>{{ t("launcher.launchOclive.banner.strong") }}</strong>{{ t("launcher.launchOclive.banner.suffix") }}
      </p>
      <div class="apps-launch-stack">
        <section class="card card--primary-app card--launch-app card--hero-oclive">
          <div class="section-title-row section-title-row--launch">
            <h2>{{ t("launcher.launchOclive.title") }}</h2>
            <HelpHint
              :text="String(t('launcher.launchOclive.help'))"
            />
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">{{ t("launcher.launchOclive.sections.roles.title") }}</h3>
            <div class="roles-block roles-block--in-launch">
              <div class="label-with-hint">
                <label>{{ t("launcher.launchOclive.sections.roles.rootLabel") }}</label>
                <HelpHint
                  :text="String(t('launcher.launchOclive.sections.roles.rootHelp'))"
                />
              </div>
              <p class="hint tiny">{{ t("launcher.launchOclive.sections.roles.rootHintPrefix") }} <code>{{ t("launcher.launchOclive.sections.roles.roleId") }}</code> {{ t("launcher.launchOclive.sections.roles.rootHintSuffix") }}</p>
              <div class="row">
                <input v-model="config.ocliveRolesDir" :placeholder="String(t('launcher.launchOclive.sections.roles.rootPlaceholder'))" />
                <button type="button" class="btn" @click="pickRolesRoot">{{ t("common.browse") }}</button>
                <button type="button" class="btn" @click="fillSuggestedRolesDir">{{ t("launcher.common.suggestFromRepo") }}</button>
              </div>
              <p class="hint tiny">{{ t("launcher.launchOclive.sections.roles.installHint") }}</p>
              <div class="row role-install-row">
                <button type="button" class="btn primary" @click="beginInstallRolePack">{{ t("launcher.launchOclive.sections.roles.installZip") }}</button>
              </div>
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">{{ t("launcher.launchOclive.sections.llm.title") }}</h3>
            <div class="llm-backend-block llm-backend-block--in-launch">
              <div class="label-with-hint">
                <label>{{ t("launcher.launchOclive.sections.llm.modeLabel") }}</label>
                <HelpHint
                  :text="String(t('launcher.launchOclive.sections.llm.modeHelp'))"
                />
              </div>
              <p class="hint tiny">{{ t("launcher.launchOclive.sections.llm.modeHint") }}</p>
              <div class="mode">
                <label><input v-model="config.ocliveLlmMode" type="radio" value="ollama" /> {{ t("launcher.launchOclive.sections.llm.localOllama") }}</label>
                <label><input v-model="config.ocliveLlmMode" type="radio" value="remote" /> {{ t("launcher.launchOclive.sections.llm.remoteApi") }}</label>
              </div>
              <template v-if="config.ocliveLlmMode === 'remote'">
                <label>{{ t("launcher.launchOclive.sections.llm.remoteUrlLabel") }}</label>
                <input
                  v-model="config.ocliveRemoteLlmUrl"
                  :placeholder="String(t('launcher.common.placeholders.rpcUrl'))"
                  autocomplete="off"
                />
                <label>{{ t("launcher.launchOclive.sections.llm.remoteTokenLabel") }}</label>
                <input
                  v-model="config.ocliveRemoteLlmToken"
                  type="password"
                  autocomplete="off"
                  :placeholder="String(t('common.optional'))"
                />
                <label>{{ t("launcher.launchOclive.sections.llm.remoteTimeoutLabel") }}</label>
                <input
                  v-model="config.ocliveRemoteLlmTimeoutMs"
                  :placeholder="String(t('launcher.common.placeholders.timeoutMs'))"
                  inputmode="numeric"
                />
              </template>
              <label>{{ t("launcher.launchOclive.sections.llm.sidecarUrlLabel") }}</label>
              <input
                v-model="config.ocliveRemotePluginUrl"
                :placeholder="String(t('launcher.common.placeholders.rpcUrl'))"
                autocomplete="off"
              />
              <label>{{ t("launcher.launchOclive.sections.llm.sidecarTokenLabel") }}</label>
              <input
                v-model="config.ocliveRemotePluginToken"
                type="password"
                autocomplete="off"
                :placeholder="String(t('common.optional'))"
              />
              <label>{{ t("launcher.launchOclive.sections.llm.sidecarTimeoutLabel") }}</label>
              <input
                v-model="config.ocliveRemotePluginTimeoutMs"
                :placeholder="String(t('launcher.common.placeholders.timeoutMsShort'))"
                inputmode="numeric"
              />
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">{{ t("launcher.launchOclive.sections.download.title") }}</h3>
            <p class="hint tiny">{{ t("launcher.launchOclive.sections.download.hint") }}</p>
            <div class="gh-paste-block gh-paste-block--inline">
              <div class="label-with-hint">
                <span class="gh-paste-inline-label">{{ t("launcher.launchOclive.sections.download.pasteRepoUrl") }}</span>
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
                <button type="button" class="btn" @click="applyOcliveRepoFromPastedUrl">{{ t("launcher.versionPage.applyOwnerRepo") }}</button>
              </div>
            </div>
            <div class="gh-release-dl gh-release-dl--compact">
              <div class="label-with-hint gh-release-dl__label-row">
                <span class="gh-release-dl__label">{{ t("launcher.launchOclive.sections.download.ghReleaseLabel") }}</span>
                <HelpHint :paragraphs="LAUNCHER_HINT_OCLIVE_GH_DL" />
              </div>
              <div class="row gh-release-dl-row">
                <button
                  type="button"
                  class="btn"
                  :disabled="ocliveGhBusy"
                  @click="refreshOcliveGhAssets"
                >
                  {{ t("launcher.common.gh.listAssets") }}
                </button>
                <select v-model="ocliveGhAssetUrl" class="gh-asset-select">
                  <option value="">{{ t("launcher.common.gh.pickFile") }}</option>
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
                  {{ t("launcher.common.gh.downloadTo") }}
                </button>
              </div>
              <p class="hint tiny">
                {{ t("launcher.launchOclive.sections.download.saveAsHint") }}
              </p>
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">{{ t("launcher.launchOclive.sections.runMode.title") }}</h3>
            <div class="mode mode--wrap">
              <label><input v-model="config.ocliveMode" type="radio" value="dev" /> {{ t("launcher.launchOclive.sections.runMode.dev") }}</label>
              <label><input v-model="config.ocliveMode" type="radio" value="exe" /> {{ t("launcher.launchOclive.sections.runMode.exe") }}</label>
            </div>
            <template v-if="config.ocliveMode === 'dev'">
              <label>{{ t("launcher.launchOclive.sections.runMode.sourceRoot") }}</label>
              <div class="row">
                <input v-model="config.ocliveProjectRoot" :placeholder="String(t('launcher.launchOclive.sections.runMode.sourceRootPlaceholder'))" />
                <button type="button" class="btn" @click="pickOcliveRoot">{{ t("common.browse") }}</button>
              </div>
              <label>{{ t("launcher.launchOclive.sections.runMode.npmScriptLabel") }}</label>
              <input v-model="config.ocliveNpmScript" :placeholder="String(t('launcher.launchOclive.sections.runMode.npmScriptPlaceholder'))" />
            </template>
            <template v-else>
              <div class="label-with-hint">
                <label>{{ t("launcher.launchOclive.sections.runMode.exePathLabel") }}</label>
                <HelpHint :paragraphs="LAUNCHER_HINT_EXE_PATH_PASTE" />
              </div>
              <p class="hint tiny">{{ t("launcher.launchOclive.sections.runMode.exePathHint") }}</p>
              <div class="row">
                <input
                  v-model="config.ocliveExe"
                  :placeholder="String(t('launcher.launchOclive.sections.runMode.exePathPlaceholder'))"
                  autocomplete="off"
                  @paste="onOcliveExeInputPaste"
                />
                <button type="button" class="btn" @click="pickOcliveExe">{{ t("common.browse") }}</button>
                <button type="button" class="btn" @click="applyOcliveExeFromField">{{ t("launcher.common.recognizePath") }}</button>
              </div>
            </template>
          </div>

          <div class="app-feature-block app-feature-block--run">
            <h3 class="app-feature-block__title">{{ t("launcher.launchOclive.sections.run.title") }}</h3>
            <div class="actions actions--launch">
              <button type="button" class="btn primary btn-launch" @click="spawnOclive">{{ t("launcher.launchOclive.sections.run.start") }}</button>
              <button type="button" class="btn danger btn-launch" @click="stopOclive">{{ t("launcher.launchOclive.sections.run.stop") }}</button>
              <button type="button" class="btn btn-launch-secondary" @click="focusLogsFilter('oclive')">
                {{ t("launcher.launchOclive.sections.run.filterLogs") }}
              </button>
            </div>
          </div>
        </section>
      </div>

      <section class="card apps-log-preview-card">
        <div class="apps-log-preview-head">
          <h3 class="apps-log-preview-title">{{ t("launcher.launchOclive.preview.title") }}</h3>
          <div class="apps-log-preview-tools">
            <button type="button" class="btn" @click="focusLogsFilter('all')">{{ t("launcher.launchOclive.preview.openFullLogs") }}</button>
          </div>
        </div>
        <pre
          ref="appsLogPreviewEl"
          class="apps-log-preview"
          :class="{ 'apps-log-preview--empty': !ocliveLogPreviewText.trim() }"
        >{{ ocliveLogPreviewText || t("launcher.launchOclive.preview.empty") }}</pre>
        <p class="hint tiny apps-log-preview-foot">
          {{ t("launcher.launchOclive.preview.foot") }}
        </p>
      </section>
    </div>

    <div v-else-if="activeNav === 'launch-editor'" class="view-panel app-launch-page app-launch-page--editor">
      <p class="apps-terminal-banner apps-terminal-banner--editor">
        {{ t("launcher.launchEditor.banner.prefix") }} <strong>{{ t("launcher.launchEditor.name") }}</strong>
        {{ t("launcher.launchEditor.banner.middle") }}
        <strong>{{ t("launcher.launchEditor.banner.strong") }}</strong>{{ t("launcher.launchEditor.banner.suffix") }}
      </p>
      <div class="apps-launch-stack">
        <section class="card card--launch-app card--hero-editor">
          <div class="section-title-row section-title-row--launch">
            <h2>{{ t("launcher.launchEditor.name") }}</h2>
            <HelpHint
              :text="String(t('launcher.launchEditor.help'))"
            />
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">{{ t("launcher.launchEditor.sections.download.title") }}</h3>
            <p class="hint tiny">{{ t("launcher.launchEditor.sections.download.hint") }}</p>
            <div class="gh-paste-block gh-paste-block--inline">
              <div class="label-with-hint">
                <span class="gh-paste-inline-label">{{ t("launcher.launchEditor.sections.download.pasteRepoUrl") }}</span>
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
                <button type="button" class="btn" @click="applyEditorRepoFromPastedUrl">{{ t("launcher.versionPage.applyOwnerRepo") }}</button>
              </div>
            </div>
            <div class="gh-release-dl gh-release-dl--compact">
              <div class="label-with-hint gh-release-dl__label-row">
                <span class="gh-release-dl__label">{{ t("launcher.launchEditor.sections.download.ghReleaseLabel") }}</span>
                <HelpHint :paragraphs="LAUNCHER_HINT_EDITOR_GH_DL" />
              </div>
              <div class="row gh-release-dl-row">
                <button
                  type="button"
                  class="btn"
                  :disabled="editorGhBusy"
                  @click="refreshEditorGhAssets"
                >
                  {{ t("launcher.common.gh.listAssets") }}
                </button>
                <select v-model="editorGhAssetUrl" class="gh-asset-select">
                  <option value="">{{ t("launcher.common.gh.pickFile") }}</option>
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
                  {{ t("launcher.common.gh.downloadTo") }}
                </button>
              </div>
              <p class="hint tiny">{{ t("launcher.launchEditor.sections.download.saveAsHint") }}</p>
            </div>
          </div>

          <div class="app-feature-block">
            <h3 class="app-feature-block__title">{{ t("launcher.launchEditor.sections.open.title") }}</h3>
            <div class="mode mode--wrap">
              <label><input v-model="config.editorMode" type="radio" value="web" /> {{ t("launcher.launchEditor.sections.open.web") }}</label>
              <label><input v-model="config.editorMode" type="radio" value="dev" /> {{ t("launcher.launchEditor.sections.open.dev") }}</label>
              <label><input v-model="config.editorMode" type="radio" value="exe" /> {{ t("launcher.launchEditor.sections.open.exe") }}</label>
            </div>
            <template v-if="config.editorMode === 'web'">
              <label>{{ t("launcher.launchEditor.sections.open.webUrlLabel") }}</label>
              <input
                v-model="config.editorWebUrl"
                type="url"
                autocomplete="off"
                :placeholder="String(t('launcher.launchEditor.sections.open.webUrlPlaceholder', { url: editorPagesUrlPreview }))"
              />
              <p class="hint tiny">{{ t("launcher.launchEditor.sections.open.webHint") }}</p>
            </template>
            <template v-else-if="config.editorMode === 'dev'">
              <label>{{ t("launcher.launchEditor.sections.open.sourceRoot") }}</label>
              <div class="row">
                <input v-model="config.editorProjectRoot" :placeholder="String(t('launcher.launchEditor.sections.open.sourceRootPlaceholder'))" />
                <button type="button" class="btn" @click="pickEditorRoot">{{ t("common.browse") }}</button>
              </div>
              <label>{{ t("launcher.launchEditor.sections.open.npmScriptLabel") }}</label>
              <input v-model="config.editorNpmScript" :placeholder="String(t('launcher.launchEditor.sections.open.npmScriptPlaceholder'))" />
            </template>
            <template v-else>
              <div class="label-with-hint">
                <label>{{ t("launcher.launchEditor.sections.open.exePathLabel") }}</label>
                <HelpHint :paragraphs="LAUNCHER_HINT_EXE_PATH_PASTE" />
              </div>
              <p class="hint tiny">{{ t("launcher.launchEditor.sections.open.exePathHint") }}</p>
              <div class="row">
                <input
                  v-model="config.editorExe"
                  :placeholder="String(t('launcher.launchEditor.sections.open.exePathPlaceholder'))"
                  autocomplete="off"
                  @paste="onEditorExeInputPaste"
                />
                <button type="button" class="btn" @click="pickEditorExe">{{ t("common.browse") }}</button>
                <button type="button" class="btn" @click="applyEditorExeFromField">{{ t("launcher.common.recognizePath") }}</button>
              </div>
            </template>
          </div>

          <div class="app-feature-block app-feature-block--run">
            <h3 class="app-feature-block__title">{{ t("launcher.launchEditor.sections.run.title") }}</h3>
            <div class="actions actions--launch">
              <button type="button" class="btn primary btn-launch" @click="spawnEditor">
                {{ config.editorMode === 'web' ? t("launcher.launchEditor.sections.run.openInBrowser") : t("launcher.launchEditor.sections.run.launchEditor") }}
              </button>
              <button type="button" class="btn danger btn-launch" @click="stopEditor">{{ t("launcher.launchEditor.sections.run.stop") }}</button>
              <button type="button" class="btn btn-launch-secondary" @click="focusLogsFilter('editor')">
                {{ t("launcher.launchEditor.sections.run.filterLogs") }}
              </button>
            </div>
          </div>
        </section>
      </div>

      <section class="card apps-log-preview-card">
        <div class="apps-log-preview-head">
          <h3 class="apps-log-preview-title">{{ t("launcher.launchEditor.preview.title") }}</h3>
          <div class="apps-log-preview-tools">
            <button type="button" class="btn" @click="focusLogsFilter('all')">{{ t("launcher.launchEditor.preview.openFullLogs") }}</button>
          </div>
        </div>
        <pre
          ref="appsLogPreviewEl"
          class="apps-log-preview"
          :class="{ 'apps-log-preview--empty': !editorLogPreviewText.trim() }"
        >{{ editorLogPreviewText || t("launcher.launchEditor.preview.empty") }}</pre>
        <p class="hint tiny apps-log-preview-foot">
          {{ t("launcher.launchEditor.preview.foot") }}
        </p>
      </section>
    </div>

    <section v-else-if="activeNav === 'logs'" class="view-panel card log-card">
      <div class="log-head">
        <div class="section-title-row log-title-row">
          <h2>{{ t("launcher.logsPage.title") }}</h2>
          <HelpHint :paragraphs="LAUNCHER_HINT_LOGS" />
        </div>
        <div class="log-tools">
          <label>{{ t("launcher.logsPage.onlyShow") }}</label>
          <select v-model="logFilter">
            <option value="all">{{ t("launcher.logsPage.filters.all") }}</option>
            <option value="editor">{{ t("launcher.logsPage.filters.editor") }}</option>
            <option value="oclive">oclive</option>
            <option value="ollama">{{ t("launcher.logsPage.filters.ollama") }}</option>
            <option value="winget">{{ t("launcher.logsPage.filters.winget") }}</option>
            <option value="bundled-ollama">{{ t("launcher.logsPage.filters.bundledOllama") }}</option>
          </select>
          <button type="button" class="btn" @click="clearLogs">{{ t("launcher.logsPage.clear") }}</button>
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
          <h2 id="install-modal-title">{{ t("launcher.installModal.title") }}</h2>
          <p v-if="pendingZipPath" class="hint tiny modal-zip-path">
            {{ pendingZipPath }}
          </p>
          <label class="modal-label">{{ t("launcher.installModal.modelLabel") }}</label>
          <p class="hint tiny">
            {{ t("launcher.installModal.modelHintPrefix") }} <code>{{ DEFAULT_OLLAMA_MODEL }}</code
            >{{ t("launcher.installModal.modelHintSuffix") }}
          </p>
          <select v-model="installModelSelect" class="modal-select">
            <option v-for="opt in installModelOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
          <div v-if="installModelSelect === MODEL_OPTION_CUSTOM" class="modal-custom">
            <label>{{ t("launcher.installModal.customModelLabel") }}</label>
            <input v-model="installCustomModel" type="text" :placeholder="String(t('launcher.installModal.customModelPlaceholder'))" />
          </div>
          <label class="modal-check">
            <input v-model="installOverwriteModel" type="checkbox" />
            {{ t("launcher.installModal.overwriteModel") }}
          </label>
          <div class="modal-actions">
            <button
              type="button"
              class="btn"
              :disabled="pullBusy || wingetInstallBusy"
              @click="refreshOllamaLocalModelsList"
            >
              {{ t("launcher.installModal.refreshLocalList") }}
            </button>
            <button
              type="button"
              class="btn"
              :disabled="pullBusy || installBusy || wingetInstallBusy"
              @click="pullSelectedOllamaModel"
            >
              {{ t("launcher.installModal.pullSelected") }}
            </button>
            <button type="button" class="btn" @click="cancelInstallRolePackModal">{{ t("common.cancel") }}</button>
            <button
              type="button"
              class="btn primary"
              :disabled="installBusy || pullBusy"
              @click="confirmInstallRolePack"
            >
              {{ t("launcher.installModal.extractAndWrite") }}
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

.shell-locale-select {
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
    background var(--motion-fast) var(--ease-out),
    border-color var(--motion-fast) var(--ease-out),
    box-shadow var(--motion-fast) var(--ease-out),
    transform var(--motion-fast) var(--ease-out),
    filter var(--motion-fast) var(--ease-out);
  will-change: transform;
}

.btn:hover {
  background: var(--fluent-bg-subtle);
  border-color: var(--fluent-text-secondary);
  transform: translateY(-1px);
  box-shadow: var(--fluent-shadow-card);
}

.btn:focus-visible {
  outline: none;
  box-shadow:
    var(--fluent-shadow-soft),
    0 0 0 2px var(--fluent-bg-page),
    0 0 0 4px var(--fluent-border-focus);
}

.btn:active:not(:disabled) {
  transform: translateY(0px) scale(0.985);
  box-shadow: var(--fluent-shadow-soft);
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
