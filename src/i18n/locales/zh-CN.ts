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

