/**
 * 从用户粘贴内容解析 GitHub 仓库或 Windows exe 路径，降低手填 owner/repo 与路径的心智负担。
 */

export function parseGithubRepoFromUrl(raw: string): { owner: string; repo: string } | null {
  let t = raw.trim()
  if (!t) return null

  if (t.startsWith('git@github.com:')) {
    t = `https://github.com/${t.slice('git@github.com:'.length)}`
  }
  if (!/^https?:\/\//i.test(t)) {
    t = `https://${t.replace(/^\/+/, '')}`
  }

  let url: URL
  try {
    url = new URL(t)
  } catch {
    return null
  }

  const host = url.hostname.toLowerCase()
  if (host !== 'github.com' && host !== 'www.github.com') {
    return null
  }

  const parts = url.pathname.split('/').filter(Boolean)
  if (parts.length < 2) return null

  const owner = parts[0]
  let repo = parts[1]
  if (repo.endsWith('.git')) repo = repo.slice(0, -4)
  if (!owner || !repo) return null

  return { owner, repo }
}

/** 识别形如 `C:\foo\bar.exe` 或带引号的路径；不是 exe 则返回 null */
export function normalizeExePathPaste(raw: string): string | null {
  let s = raw.trim()
  if (!s) return null

  if (
    (s.startsWith('"') && s.endsWith('"')) ||
    (s.startsWith("'") && s.endsWith("'"))
  ) {
    s = s.slice(1, -1).trim()
  }

  if (!/\.exe$/i.test(s)) return null
  return s
}
