import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { TAURI_ANNOUNCEMENTS } from './tauri'

export function useDeveloperAnnouncements(setStatus: (msg: string) => void) {
  const text = ref('')
  const fetchBusy = ref(false)

  async function reloadFromDisk() {
    try {
      text.value = await invoke<string>(TAURI_ANNOUNCEMENTS.loadDeveloper)
    } catch (e) {
      setStatus(String(e))
    }
  }

  async function saveToDisk() {
    try {
      await invoke(TAURI_ANNOUNCEMENTS.saveDeveloper, { text: text.value })
      setStatus('开发者公告已保存到本地 announcements.md')
    } catch (e) {
      setStatus(String(e))
    }
  }

  async function fetchRemoteAndCache(url: string) {
    const u = url.trim()
    if (!u) {
      setStatus('请先填写开发者公告 URL')
      return
    }
    fetchBusy.value = true
    try {
      const body = await invoke<string>(TAURI_ANNOUNCEMENTS.fetchRemoteText, { url: u })
      await invoke(TAURI_ANNOUNCEMENTS.saveDeveloper, { text: body })
      text.value = body
      setStatus('已从 URL 拉取并写入本地 announcements.md')
    } catch (e) {
      setStatus(String(e))
    } finally {
      fetchBusy.value = false
    }
  }

  return {
    text,
    fetchBusy,
    reloadFromDisk,
    saveToDisk,
    fetchRemoteAndCache,
  }
}
