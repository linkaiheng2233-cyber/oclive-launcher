export default {
  common: {
    language: "语言",
    system: "跟随系统",
    zhCN: "中文",
    enUS: "English",
  },
  helpHint: {
    ariaLabel: "查看说明",
  },
  launcher: {
    views: {
      start: "新手入门",
      version: "版本与下载",
      launchOclive: "启动 oclive",
      launchEditor: "角色包编写器",
      assistant: "环境检查",
      logs: "运行日志",
    },
    titlebar: {
      kicker: "oclive · 工具链",
      toolsAria: "外观与语言",
      scaleAria: "界面大小",
      shrink: "缩小",
      shrinkAria: "缩小界面",
      enlarge: "放大",
      enlargeAria: "放大界面",
      relativeScaleTitle: "相对默认字号：{label}",
      themeTitle: "主题：{label}（点击切换）",
      theme: {
        light: "浅色",
        dark: "深色",
        system: "跟随系统",
      },
      saveConfigTitle:
        "把左侧各页的路径、GitHub 仓库、运行模式等写入本机配置文件；主题与界面字号保存在浏览器本地，会即时生效。",
      saveConfig: "保存配置",
    },
    viewSub: {
      start: "新手指引、开发者公告与创作者公告（包内寄语，只读）都在本页。",
      version: "对照网上发布的版本号，顺便一键打开下载页。",
      launchOclive:
        "配角色目录、对话大脑、安装包与启动路径；输出在下方摘要与「日志」里查看。",
      launchEditor:
        "下载或指定编写器（网页 / 源码 / exe），再一键打开；日志同样汇总在下方与「日志」页。",
      assistant: "看看 Node、Ollama、文件夹路径对不对；不对就按提示装或改路径。",
      other: "软件在后台打印的信息都在这里，出问题先来这里瞄一眼。",
    },
    nav: {
      mobileAria: "栏目切换",
    },
    startGuide: {
      title: "新手照着做就行",
      lead:
        "你可以只聊天、只做角色，或两个都来——下面是一条最省事的路线：写设定 → 放进角色文件夹 → 开 oclive 聊天。看版本号、对 GitHub 发版请点左侧「版本」。",
      desc:
        "三个东西分工不同：本程序负责一键打开；编写器用来写内容；oclive 是聊天窗口。角色文件都放在磁盘上的 roles 一类文件夹里（启动器里叫「角色包根目录」）。",
      links: {
        ollamaDownload: "Ollama 下载",
        ollamaLibrary: "Ollama 模型库",
        editorReleases: "编写器 Releases",
        ocliveReleases: "运行时 Releases",
      },
    },
    versionPage: {
      title: "看版本、去下载",
      lead:
        "大白话：这一页就是帮你对照「电脑上装的版本」和「GitHub 上最新发的一不一样」，再顺手打开市场汇总页或某个仓库的下载页。具体名词点各小节旁的小问号看详细说明。",
      quickLinks: "快捷入口",
      buttons: {
        versionsListing: "生态站 · 发布汇总页",
        editorReleases: "编写器 Releases",
        ocliveReleases: "oclive 运行时 Releases",
      },
      editorRepoPaste: {
        label: "粘贴编写器仓库网址（可选）",
        placeholder: "例如 https://github.com/你的用户名/oclive-pack-editor",
        apply: "填入 owner / repo",
      },
      remoteVersion: "网上最新",
      openRelease: "打开发布页",
    },
    status: {
      configSaved: "配置已保存",
      githubRepoUnrecognized:
        "认不出 GitHub 仓库地址。请粘贴浏览器地址栏里的链接，例如 https://github.com/用户名/仓库名",
      githubEditorRepoApplied: "已填入编写器仓库：{owner}/{repo}（可到「编写器」页点「列出附件」下载）",
      githubOcliveRepoApplied: "已填入 oclive 仓库：{owner}/{repo}（可到「启动 oclive」页点「列出附件」下载）",
      ocliveExeRecognizedFromPaste: "已从粘贴识别 oclive.exe，已切换到「已安装的 exe」",
      editorExeRecognizedFromPaste: "已从粘贴识别编写器 exe，已切换到「本地 exe」",
      exePathInvalid: "请填写以 .exe 结尾的完整路径（可含引号）",
      ocliveExeNormalized: "已整理 oclive.exe 路径并切换到 exe 模式",
      editorExeNormalized: "已整理编写器 exe 路径并切换到本地 exe 模式",
      ghAssetsListedOclive: "已列出 oclive 仓库 {n} 个 Release 附件",
      ghAssetsListedEditor: "已列出编写器仓库 {n} 个 Release 附件",
      ghAssetsNone: "该 Release 下没有附件（或仓库尚无 Release）",
      pickAssetFirst: "请先点「列出附件」并选择一个文件",
      ocliveDownloadedAndConfigured: "已下载并填入 oclive 路径，已切换到 exe 模式并保存配置",
      editorDownloadedAndConfigured: "已下载并填入编写器路径，已切换到 exe 模式并保存配置",
      rolesDirSuggestedFilled: "已填入 oclive 仓库下的 roles 目录",
      rolesDirSuggestedNotFound:
        "未找到：请确认「oclive 项目根」正确，且其下存在 roles 文件夹（若尚未克隆仓库可先手动选择目录）",
      rolesRootMissing: "请先在下方填写「角色包根目录」",
      ollamaLocalModelsListFailed:
        "未能列出本机已拉取的模型（Ollama 未启动？）。仍可手动输入模型名并完成安装。",
      ollamaModelMissing: "请选择或输入 Ollama 模型名",
      installMissingZipOrRoot: "缺少 zip 或角色包根目录",
      rolePackInstalled: "已安装角色「{roleId}」到角色包目录，并已按选项写入 settings.json 的 model。",
      bundledOllamaInstallerLaunched: "已启动附带安装程序；完成后请在「环境」页点「重新检测」。",
      wingetInstallStarted:
        "已开始 winget 安装 Ollama，进度见「日志」→ 筛选 winget。完成后请点「重新检测」。",
    },
    confirms: {
      launchBundledOllamaInstaller:
        "将启动附带的 Ollama 安装程序（Windows）。若已安装过 Ollama，向导可能提示修复或卸载。是否继续？",
      installOllamaViaWinget:
        "将通过 Windows 官方包管理器 winget 安装「Ollama.Ollama」。可能弹出 UAC 或安装向导，且需网络下载。是否继续？",
    },
  },
  creatorAnnouncements: {
    title: "创作者公告",
    rolePickerAria: "选择要展示公告的角色",
    roleLabel: "角色",
    pickRole: "选择角色…",
    refresh: "刷新列表",
    clearTitle: "清空当前选中的角色（不删包内文件）",
    clear: "取消跟随",
    clearHint: "「取消跟随」= 不再锁定要展示公告的角色；角色文件夹与 {file} 不会被删除。",
    noFile: "该包内尚无 {file}。",
    emptyPick: "请选择角色。",
  },
  developerAnnouncements: {
    title: "开发者公告",
    readonlyHint: "只读；维护者构建可编辑保存。",
    remoteUrlLabel: "远程正文地址（可选）",
    remoteUrlPlaceholder: "https://raw.githubusercontent.com/你的用户/你的仓库/main/公告.md",
    fetching: "拉取中…",
    fetchLatest: "拉取最新",
    remoteUrlHint:
      "不是复制仓库主页链接：需要 Raw 或等价直链（响应体就是 Markdown/纯文本）。改 URL 后请先「保存配置」，再拉取。",
    saveLocal: "保存到本地",
    empty: "暂无开发者公告。",
  },
};

