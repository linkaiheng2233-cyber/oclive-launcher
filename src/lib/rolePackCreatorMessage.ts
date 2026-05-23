/**
 * 随包寄语（`creator_message.txt`）前端常量。
 * 修改字数或文件名时请同步：
 * - **编写器**：`oclive-pack-editor/src/lib/rolePackCreatorMessage.ts`
 * - **本仓库 Rust**：`src-tauri/src/role_creator_message.rs`
 */
export const ROLE_PACK_CREATOR_MESSAGE_MAX_CHARS = 160

export const ROLE_PACK_CREATOR_MESSAGE_FILENAME = 'creator_message.txt'

/** 启动器「随包寄语」所需配置字段（`LauncherConfig` 应包含此结构） */
export type RolePackEchoConfig = {
  ocliveRolesDir: string
  launcherEchoRoleId: string
}

export type RoleBlueprintMeta = {
  id: string
  name?: string | null
  version?: string | null
  author?: string | null
}

export function formatRoleBlueprintLabel(meta: RoleBlueprintMeta): string {
  const name = meta.name?.trim()
  const version = meta.version?.trim()
  const bits: string[] = []
  if (name && name !== meta.id) bits.push(name)
  bits.push(meta.id)
  if (version) bits.push(`v${version}`)
  return bits.join(' · ')
}

export const TAURI_ROLE_CREATOR_MESSAGE = {
  listRoleIds: 'list_role_ids_with_manifest',
  listRoleMeta: 'list_role_blueprint_meta',
  readBlueprintMeta: 'read_role_blueprint_meta',
  read: 'read_role_creator_message_lines',
  write: 'write_role_creator_message',
} as const
