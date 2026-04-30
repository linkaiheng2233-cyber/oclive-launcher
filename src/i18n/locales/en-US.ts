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
    nav: {
      mobileAria: "Switch sections",
    },
    startGuide: {
      title: "Just follow these steps",
      lead:
        "You can just chat, just create roles, or do both. Here’s the easiest route: write settings → put them into the role folder → open oclive and chat. For versions and GitHub releases, go to “Versions” on the left.",
      desc:
        "These three tools have different jobs: this launcher opens things with one click; the editor writes content; oclive is the chat window. Role files live in a disk folder like roles (called “Role packs root directory” in the launcher).",
      links: {
        ollamaDownload: "Ollama download",
        ollamaLibrary: "Ollama model library",
        editorReleases: "Editor releases",
        ocliveReleases: "Runtime releases",
      },
    },
    versionPage: {
      title: "Versions & downloads",
      lead:
        "Plain English: this page compares the version you have installed with the latest release on GitHub, then helps you quickly open overview pages or specific release downloads. Click the small “?” next to each section for details.",
      quickLinks: "Quick links",
      buttons: {
        versionsListing: "Ecosystem site · releases overview",
        editorReleases: "Editor releases",
        ocliveReleases: "oclive runtime releases",
      },
      editorRepoPaste: {
        label: "Paste editor repository URL (optional)",
        placeholder: "e.g. https://github.com/your-name/oclive-pack-editor",
        apply: "Fill owner / repo",
      },
      remoteVersion: "Latest online",
      openRelease: "Open release page",
    },
    status: {
      configSaved: "Config saved.",
      githubRepoUnrecognized:
        "Could not recognize the GitHub repository URL. Please paste the link from your browser address bar, e.g. https://github.com/user/repo",
      githubEditorRepoApplied:
        "Editor repo set to {owner}/{repo} (you can download assets on the “Role Pack Editor” page).",
      githubOcliveRepoApplied:
        "oclive repo set to {owner}/{repo} (you can download assets on the “Launch oclive” page).",
      ocliveExeRecognizedFromPaste: "Detected oclive.exe from paste and switched to “installed exe”.",
      editorExeRecognizedFromPaste: "Detected editor exe from paste and switched to “local exe”.",
      exePathInvalid: "Please provide a full path ending with .exe (quotes are allowed).",
      ocliveExeNormalized: "Normalized oclive.exe path and switched to exe mode.",
      editorExeNormalized: "Normalized editor exe path and switched to local exe mode.",
      ghAssetsListedOclive: "Listed {n} release assets for the oclive repo.",
      ghAssetsListedEditor: "Listed {n} release assets for the editor repo.",
      ghAssetsNone: "No assets in this release (or the repo has no releases yet).",
      pickAssetFirst: "Click “List assets” first, then pick a file.",
      ocliveDownloadedAndConfigured:
        "Downloaded and filled the oclive path, switched to exe mode, and saved config.",
      editorDownloadedAndConfigured:
        "Downloaded and filled the editor path, switched to exe mode, and saved config.",
      rolesDirSuggestedFilled: "Filled the roles directory under the oclive repo.",
      rolesDirSuggestedNotFound:
        "Not found. Please confirm “oclive project root” is correct and contains a roles folder (if you haven’t cloned the repo yet, pick a directory manually).",
      rolesRootMissing: "Please fill “Role packs root directory” below first.",
      ollamaLocalModelsListFailed:
        "Could not list local models (is Ollama running?). You can still type a model name manually and continue installation.",
      ollamaModelMissing: "Please choose or enter an Ollama model name.",
      installMissingZipOrRoot: "Missing zip file or role packs root directory.",
      rolePackInstalled:
        "Installed role “{roleId}” into the role packs directory and wrote the selected model into settings.json.",
      bundledOllamaInstallerLaunched:
        "Launched the bundled installer. After it completes, click “Re-check” on the Environment page.",
      wingetInstallStarted:
        "Started winget installation for Ollama. Check progress in Logs (filter: winget). When done, click “Re-check”.",
    },
    confirms: {
      launchBundledOllamaInstaller:
        "This will start the bundled Ollama installer (Windows). If Ollama is already installed, the wizard may offer repair/uninstall. Continue?",
      installOllamaViaWinget:
        "This will install “Ollama.Ollama” via Windows package manager winget. It may trigger UAC/installer prompts and requires network download. Continue?",
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

