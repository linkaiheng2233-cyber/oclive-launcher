/**
 * 启动器界面：大白话说明（供 HelpHint 问号气泡分段展示）。
 */

export const LAUNCHER_HINT_START_GUIDE = [
  '这个程序不代替「编写器」和「oclive」，它负责把路径、下载和启动串起来：你在左边选好软件在哪，点一下就能开。',
  '不必一次弄懂全部：卡住了就去「环境检查」看哪一项是红的；需要装 Node / Ollama 时页面里也有按钮跳到下载。',
  '看版本号、打开 GitHub 发版页，用左侧「版本」最省事。',
] as const

/** 版本页标题旁：整页是干什么的 */
export const LAUNCHER_HINT_VERSION_PAGE = [
  '这一页只做两件事：一是看你电脑上装的版本，和 GitHub 上最新发的一不一样；二是顺手打开市场汇总页或某个仓库的「发版」页面，方便你去下载。',
  '「编写器」和「oclive」各对应一个公开仓库。默认填的是常用官方地址；你也可以改成别人 fork 的、社区维护的仓库——只要仓库是公开的、你愿意信任对方发的安装包。',
  '下面分两块：先写「仓库是谁家的」，再看「本机版本」和「网上最新」两行数字；最底下两个按钮用来联网拉 GitHub 上的最新 Tag。具体名词点各小节旁的问号。',
] as const

/** 快捷入口三个按钮 */
export const LAUNCHER_HINT_VERSION_QUICK_LINKS = [
  '「市场站 · 各软件版本列表」：打开社区网站上的汇总页，一次浏览多个相关软件和说明（地址可在构建时用环境变量改）。',
  '「编写器 / oclive Releases」：用当前填好的 owner/repo，在浏览器里打开对应仓库的 GitHub Releases 页面，方便你挑安装包或 zip。还没改过仓库名时，会用默认官方仓库链接。',
  '这三个都是「打开网页」，不会在后台替你下载；真正下载安装包在「启动 oclive / 编写器」页用「列出附件」或浏览器里自行下载。',
] as const

/** 编写器 owner / repo 两行输入 */
export const LAUNCHER_HINT_VERSION_REPO_EDITOR = [
  '左边填 GitHub 用户名或组织名（owner），右边填仓库短名（repo）。合起来就是浏览器地址 github.com 后面那两段。',
  '这里改的是「启动器以后查版本、打开哪个仓库」；不会自动下载软件，也不会改你磁盘上已装好的 exe。',
  '可以填官方仓库，也可以填别人 fork 的仓库；请自行判断来源是否可信，安装包与官方不兼容的风险由使用者承担。',
] as const

/** oclive owner / repo */
export const LAUNCHER_HINT_VERSION_REPO_OCLIVE = [
  '含义与上面编写器相同，只是针对「聊天软件」仓库。编写器和 oclive 可以指向不同作者，只要两个仓库都是公开的、且你愿意用他们打的包。',
] as const

/** 「本机版本」「网上最新」两行 */
export const LAUNCHER_HINT_VERSION_LOCAL_VS_REMOTE = [
  '「本机版本」：从你现在磁盘上的编写器 / oclive 项目里读到的版本号（例如 package.json）；若还没装或路径不对，可能显示成横线。',
  '「网上最新」：启动器问 GitHub 该仓库**最新一条 Release** 的 Tag 名（需要能访问 GitHub）。',
  '两边不一致，只说明「有新版本发过」，不等于自动提示你升级；是否下载、怎么装仍由你在 Release 页或「启动」页里操作。',
] as const

/** 底部两个检查按钮 */
export const LAUNCHER_HINT_VERSION_ACTIONS = [
  '「同步粘贴的 GitHub 网址并检查更新」：先看两个「粘贴网址」框里有没有能认出来的仓库链接，有就写入 owner/repo 并保存配置，再向 GitHub 拉最新 Tag。框里留空就只用已经填好的两行 owner/repo。',
  '「仅检查更新」：不动粘贴框，也不改你已保存的仓库名，只再查一遍远端 Tag，适合你已经手改好了 owner/repo 的情况。',
  '若你把仓库改成了第三方，相当于信任对方发布的二进制；请自行甄别来源，本工具无法替第三方担保安全性。',
] as const

export const LAUNCHER_HINT_GH_URL_PASTE = [
  '支持常见形式：浏览器地址栏的 https://github.com/用户名/仓库名、带 releases 的路径、或 git@github.com:用户名/仓库.git。',
  '点「填入」后只会改上面的 owner 和 repo 两格，不会自动下载；下载安装包请在「启动 oclive / 编写器」页点「列出附件」。',
] as const

export const LAUNCHER_HINT_OCLIVE_GH_DL = [
  '先确认上面「版本与下载」里的 oclive 仓库是你自己的（可用粘贴网址填入）。点「列出附件」会读该仓库最新 Release 里的文件列表。',
  '选一个带 .exe 或便携包（zip 等）的附件再下载；若解压后能找到 exe，启动器会尽量把路径填进「oclive.exe」并切到 exe 模式。',
] as const

export const LAUNCHER_HINT_EDITOR_GH_DL = [
  '仓库名与「版本与下载」里编写器那一段共用。点「列出附件」拉取 GitHub Release；下载便携包时若识别到 exe，会自动写入「编写器 exe」并切到本地 exe 模式。',
] as const

export const LAUNCHER_HINT_EXE_PATH_PASTE = [
  '若你已经用资源管理器找到 oclive.exe 或编写器 exe，可在资源管理器地址栏复制完整路径，粘贴到输入框：启动器会识别以 .exe 结尾的路径并切到「已安装 exe」模式。',
  '也可以带引号粘贴（例如从快捷方式属性里复制）。若粘贴的不是 exe，输入框仍按普通文字处理。',
] as const

export const LAUNCHER_HINT_ASSISTANT = [
  '这张表回答三件事：Node/npm 有没有（只有「开发模式」跑源码才硬性需要）、Ollama 在不在跑、以及你填的文件夹路径是否指对了。',
  '红的项按提示装软件或改路径后再点「重新检测」。只用安装包 exe、不跑 npm 的话，Node 一行红可以忽略。',
] as const

export const LAUNCHER_HINT_LOGS = [
  '从本程序里启动 oclive、编写器、或点「拉模型」「winget」时，子进程的输出会汇总到这里，方便你不用另开黑色命令行窗口。',
  '下拉选「只看 oclive」等可过滤；卡住了就翻到最下面看最后几行报错。',
] as const

/** 顶栏：字号、主题、保存配置各自管什么 */
export const LAUNCHER_HINT_TITLEBAR_TOOLS = [
  '「A− / 中间百分比 / A+」：调节整页界面相对默认字号的缩放，档位与「角色包编写器」一致；存在本机浏览器里，改完立刻生效。',
  '「浅色 / 深色 / 跟随系统」：只影响本窗口配色，同样记在本地，不必按「保存配置」。',
  '「保存配置」：把你在各页填的**路径、GitHub 仓库、运行模式、角色目录、对话相关选项等**写入启动器配置文件（磁盘）；**不包含**仅界面的主题与字号——那两项已自动保存。',
] as const
