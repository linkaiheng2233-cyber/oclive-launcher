import { type Ref, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  type RoleBlueprintMeta,
  type RolePackEchoConfig,
  TAURI_ROLE_CREATOR_MESSAGE,
} from '../lib/rolePackCreatorMessage'

export function useRolePackEcho<C extends RolePackEchoConfig>(
  config: Ref<C>,
  opts: {
    setStatus: (msg: string) => void
    /** 写入完整 `launcher-config.json`（仅负责持久化，不改文案） */
    persistLauncherConfig: () => Promise<void>
  },
) {
  const echoLines = ref<string[]>([])
  const roleSummaries = ref<RoleBlueprintMeta[]>([])

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
      roleSummaries.value = []
      echoLines.value = []
      return
    }
    try {
      roleSummaries.value = await invoke<RoleBlueprintMeta[]>(TAURI_ROLE_CREATOR_MESSAGE.listRoleMeta, {
        rolesRoot: root,
      })
    } catch {
      roleSummaries.value = []
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
    roleSummaries,
    /** @deprecated use roleSummaries */
    roleIds: roleSummaries,
    refreshEchoUi,
    refreshEchoLines,
    persistFollowRole,
    clearFollowRole,
  }
}
