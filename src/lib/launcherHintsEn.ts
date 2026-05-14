import type { LauncherHelpHintSet } from "./launcherHints";

/** English help bubbles (same keys as `HINTS_ZH`). */
export const HINTS_EN: Record<LauncherHelpHintSet, readonly string[]> = {
  START_GUIDE: [
    "This app does not replace the pack editor or oclive—it wires paths, downloads, and launch: pick where things live on the left, then open with one click.",
    "You do not need to learn everything at once: if stuck, open Environment and fix red rows; buttons link to Node / Ollama installers when needed.",
    "For version checks and GitHub releases, use the Version tab on the left.",
  ],
  VERSION_PAGE: [
    "This page compares what you have locally with the latest GitHub release tag, and opens the market summary or a repo Releases page for downloads.",
    "Editor and oclive each map to a public repo. Defaults are official URLs; you may point to forks—only if you trust their binaries.",
    "Fill owner/repo first, then read local vs remote tags; bottom buttons fetch the latest tag from GitHub. Use the per-section hints for terms.",
  ],
  VERSION_QUICK_LINKS: [
    "Market versions list: opens the community summary page (URL can be overridden at build time).",
    "Editor / oclive Releases: opens GitHub Releases for the current owner/repo so you can pick installers or zips.",
    "These buttons only open web pages; downloads happen on the Launch pages via “List assets” or your browser.",
  ],
  VERSION_REPO_EDITOR: [
    "Owner is the GitHub user/org; repo is the short name after github.com/owner/.",
    "This only changes which repo the launcher queries; it does not download or overwrite installed exes.",
    "Official or third-party repos are allowed—judge trust yourself; incompatible builds are your risk.",
  ],
  VERSION_REPO_OCLIVE: [
    "Same meaning as the editor row, but for the chat app repo. Editor and oclive repos may differ as long as both are public and trusted by you.",
  ],
  VERSION_LOCAL_VS_REMOTE: [
    "Local version: read from your editor/oclive project (e.g. package.json); may show a dash if missing or wrong path.",
    "Remote latest: the newest Release tag from GitHub (requires GitHub access).",
    "A mismatch only means a newer tag exists; upgrading is still your choice on Releases or Launch pages.",
  ],
  VERSION_ACTIONS: [
    "Sync pasted GitHub URLs & check: parses the two paste boxes, writes owner/repo when recognized, saves, then fetches tags. Empty boxes keep existing owner/repo.",
    "Check updates only: re-fetches tags without touching paste fields—useful after manual owner/repo edits.",
    "Pointing at third-party repos means trusting their binaries; this tool cannot vouch for them.",
  ],
  GH_URL_PASTE: [
    "Supports common forms: https://github.com/owner/repo, releases URLs, or git@github.com:owner/repo.git.",
    "Fill-in only updates owner/repo cells; use Launch pages → “List assets” to download installers.",
  ],
  OCLIVE_GH_DL: [
    "Confirm the oclive repo row matches what you want (paste URL if needed). “List assets” reads the latest Release file list.",
    "Pick an .exe or portable zip; if an exe is detected after download, the launcher fills oclive.exe and switches to exe mode when possible.",
  ],
  EDITOR_GH_DL: [
    "Shares the editor repo row from Version. “List assets” lists GitHub Release files; if an exe is detected, the editor exe path is filled and local exe mode is selected.",
  ],
  EXE_PATH_PASTE: [
    "Copy a full .exe path from Explorer’s address bar into the field; the launcher switches to installed-exe mode when it ends with .exe.",
    "Quoted paths (e.g. from shortcut properties) work; non-exe text is treated as plain input.",
  ],
  ASSISTANT: [
    "This table checks Node/npm (needed for dev mode from source), whether Ollama is running, and whether your folder paths look valid.",
    "Fix red rows, then re-check. If you only use packaged exes without npm, a red Node row can be ignored.",
  ],
  LOGS: [
    "Output from launches, model pulls, winget, etc. is aggregated here so you can skip opening a separate console.",
    "Use the filter (e.g. oclive-only); when stuck, read the last lines for errors.",
  ],
  TITLEBAR_TOOLS: [
    "A− / percent / A+: UI scale steps shared with the pack editor; stored locally and applied immediately.",
    "Light / dark / system: window theme only—also stored locally without pressing Save config.",
    "Save config: writes paths, repos, run modes, role dirs, LLM options, etc. to the launcher config file on disk—not theme/scale, which autosave separately.",
  ],
};
