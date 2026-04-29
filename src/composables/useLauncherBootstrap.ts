import { type Ref } from 'vue'

const STORAGE_FIRST_AUTO_ENV = 'oclive-launcher-first-auto-env-v1'

type BootstrapDeps<C> = {
  config: Ref<C>
  setStatus: (msg: string) => void
  loadConfig: () => Promise<C>
  refreshRolePackEchoUi: () => Promise<void>
  reloadDevAnnounceFromDisk: () => Promise<void>
  refreshLocalVersions: () => Promise<void>
  refreshWingetAvailability: () => Promise<void>
  refreshBundledOllamaInfo: () => Promise<void>
  runEnvironmentDiagnose: (opts?: { quiet?: boolean }) => Promise<void>
}

export function useLauncherBootstrap<C>(deps: BootstrapDeps<C>) {
  async function loadAll() {
    try {
      deps.setStatus('正在加载启动器配置…')
      const loaded = await deps.loadConfig()
      deps.config.value = { ...deps.config.value, ...loaded }
      await Promise.all([
        deps.refreshRolePackEchoUi(),
        deps.reloadDevAnnounceFromDisk(),
        deps.refreshLocalVersions(),
        deps.refreshWingetAvailability(),
        deps.refreshBundledOllamaInfo(),
      ])
      deps.setStatus('已加载配置')
    } catch (e) {
      deps.setStatus(String(e))
    }
  }

  async function maybeFirstLaunchAutoDiagnose() {
    try {
      if (localStorage.getItem(STORAGE_FIRST_AUTO_ENV)) return
      await deps.runEnvironmentDiagnose({ quiet: true })
      localStorage.setItem(STORAGE_FIRST_AUTO_ENV, '1')
      deps.setStatus('欢迎！已为你自动检测环境，详见「环境」页。需要时可再点「重新检测」。')
    } catch {
      /* ignore */
    }
  }

  function scheduleFirstLaunchDiagnose() {
    window.setTimeout(() => {
      void maybeFirstLaunchAutoDiagnose()
    }, 0)
  }

  return {
    loadAll,
    maybeFirstLaunchAutoDiagnose,
    scheduleFirstLaunchDiagnose,
  }
}
