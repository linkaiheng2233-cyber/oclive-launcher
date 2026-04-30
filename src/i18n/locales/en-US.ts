export default {
  common: {
    language: "Language",
    system: "System",
    zhCN: "中文",
    enUS: "English",
  },
  helpHint: {
    ariaLabel: "View help",
  },
  launcher: {
    views: {
      start: "Getting started",
      version: "Versions & downloads",
      launchOclive: "Launch oclive",
      launchEditor: "Role Pack Editor",
      assistant: "Environment check",
      logs: "Run logs",
    },
    titlebar: {
      kicker: "oclive · Toolchain",
      toolsAria: "Appearance and language",
      scaleAria: "UI scale",
      shrink: "Shrink",
      shrinkAria: "Shrink UI",
      enlarge: "Enlarge",
      enlargeAria: "Enlarge UI",
      relativeScaleTitle: "Relative to default font size: {label}",
      themeTitle: "Theme: {label} (click to switch)",
      theme: {
        light: "Light",
        dark: "Dark",
        system: "System",
      },
      saveConfigTitle:
        "Write paths, GitHub repos, and run modes into local config. Theme and UI scale are saved locally and take effect immediately.",
      saveConfig: "Save config",
    },
    viewSub: {
      start: "Starter guide, developer announcements, and creator announcements (pack message; read-only) are all on this page.",
      version: "Compare versions published online and open download pages quickly.",
      launchOclive:
        "Configure roles directory, chat brain, installer and launch path. Outputs are summarized below and in the Logs tab.",
      launchEditor:
        "Download or point to the editor (web / source / exe) and open it with one click. Logs are also collected below and in the Logs tab.",
      assistant:
        "Check whether Node, Ollama, and folder paths are correct; follow prompts to install or fix paths.",
      other: "All background logs are shown here. When something breaks, check here first.",
    },
    startGuide: {
      title: "Just follow these steps",
      links: {
        ollamaDownload: "Ollama download",
        ollamaLibrary: "Ollama model library",
        editorReleases: "Editor releases",
        ocliveReleases: "Runtime releases",
      },
    },
    versionPage: {
      title: "Versions & downloads",
      quickLinks: "Quick links",
      buttons: {
        versionsListing: "Ecosystem site · releases overview",
        editorReleases: "Editor releases",
        ocliveReleases: "oclive runtime releases",
      },
    },
  },
  creatorAnnouncements: {
    title: "Creator announcements",
    rolePickerAria: "Choose role for announcements",
    roleLabel: "Role",
    pickRole: "Choose a role…",
    refresh: "Refresh list",
    clearTitle: "Clear current role selection (does not delete pack files)",
    clear: "Stop following",
    clearHint:
      '"Stop following" = do not lock announcements to a role. Role folder and {file} will not be deleted.',
    noFile: "This pack does not have {file}.",
    emptyPick: "Please choose a role.",
  },
  developerAnnouncements: {
    title: "Developer announcements",
    readonlyHint: "Read-only. Maintainer builds can edit and save.",
    remoteUrlLabel: "Remote body URL (optional)",
    remoteUrlPlaceholder: "https://raw.githubusercontent.com/your-user/your-repo/main/announcements.md",
    fetching: "Fetching…",
    fetchLatest: "Fetch latest",
    remoteUrlHint:
      "Do not paste a repository homepage URL. Use a Raw (or equivalent) direct link whose response body is the Markdown/plain text. After changing the URL, click “Save config” first, then fetch.",
    saveLocal: "Save to local",
    empty: "No developer announcements.",
  },
};

