/** 与 `src-tauri/src/announcements.rs` 中 `#[tauri::command]` 名称一致 */
export const TAURI_ANNOUNCEMENTS = {
  loadDeveloper: 'load_maintainer_announcements',
  saveDeveloper: 'save_maintainer_announcements',
  fetchRemoteText: 'fetch_remote_announcement_text',
} as const
