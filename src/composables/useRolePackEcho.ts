import { type Ref, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { type RolePackEchoConfig, TAURI_ROLE_CREATOR_MESSAGE } from '../lib/rolePackCreatorMessage'

export function useRolePackEcho<C extends RolePackEchoConfig>(
  config: Ref<C>,
  opts: {
    setStatus: (msg: string) => void
    /** 写入完整 `launcher-config.json`（仅负责持久化，不改文案） */
    persistLauncherConfig: () => Promise<void>
  },
) {
  const echoLines = ref<string[]>([])
  const roleIds = ref<string[]>([])

  /** 当前跟随角色在磁盘上的寄语行（一句或多行）；未跟随或读失败时为空数组 */
  async function readFollowedRoleLines(): Promise<string[]> {
    const root = config.value.ocliveRolesDir?.trim()
    const rid = config.value.launcherEchoRoleId?.trim()
    if (!root || !rid) return []
    try {
      const lines = await invoke<string[]>(TAURI_ROLE_CREATOR_MESSAGE.read, {
        rolesRoot: root,
        roleId: rid,
      })
      return Array.isArray(lines) ? lines : []
    } catch {
      return []
    }
  }

  async function refreshEchoLines() {
    echoLines.value = await readFollowedRoleLines()
  }

  async function refreshEchoUi() {
    const root = config.value.ocliveRolesDir?.trim()
    if (!root) {
      roleIds.value = []
      echoLines.value = []
      return
    }
    try {
      roleIds.value = await invoke<string[]>(TAURI_ROLE_CREATOR_MESSAGE.listRoleIds, {
        rolesRoot: root,
      })
    } catch {
      roleIds.value = []
    }
    await refreshEchoLines()
  }

  async function persistFollowRole() {
    try {
      await opts.persistLauncherConfig()
      await refreshEchoLines()
    } catch (e) {
      opts.setStatus(String(e))
    }
  }

  async function clearFollowRole() {
    config.value.launcherEchoRoleId = ''
    await persistFollowRole()
  }

  return {
    echoLines,
    roleIds,
    refreshEchoUi,
    refreshEchoLines,
    persistFollowRole,
    clearFollowRole,
  }
}
