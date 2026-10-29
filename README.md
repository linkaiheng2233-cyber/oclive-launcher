# OCLive 旧版桌面启动器

**本仓已退役，保留历史源码供查阅。新用户可直接使用 A.I.Live 运行时与角色包编写器，无需安装本启动器。**

A historical OCLive desktop launcher. This repository is not the current end-user entry point.

## 现在从哪里开始

| 目标 | 公开入口 |
|---|---|
| 导入角色包、运行角色和聊天 | [A.I.Live / Chat Pro](https://github.com/linkaiheng2233-cyber/oclivenewnew) · [用户手册](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/getting-started/USER_MANUAL.md) |
| 创作、转换和导出角色包 | [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) |
| 了解创作与运行的衔接 | [创作者流程](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/getting-started/CREATOR_WORKFLOW.md) |

早期的工作室整合不构成一个已经公开提供的替代安装包。公开使用路径以上述仓库的实际发布说明为准。

## 历史范围

旧启动器曾集中处理应用路径、环境检查、子进程日志与发布页入口。它不包含模型或完整对话引擎；旧配置、Ollama 同捆和多应用启动说明属于当时的实现快照。

源码版本见 [package.json](package.json)。完整历史使用说明保留在[原始 README 快照](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/adea5c5e5fc499c3e4659fa4291e37419d04483e/README.md)，其中旧路径、依赖和发布计划不代表当前产品承诺。

2026-10-29 的 [CI 构建](https://github.com/linkaiheng2233-cyber/oclive-launcher/actions/workflows/ci.yml)仍因 `src/i18n/locales/shared/common.en`、`common.zh` 模块缺失而未通过。历史源码不作为本轮可构建或可发布的软件交付。

## 反馈

当前运行时问题请到[主仓 Issue](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues)反馈；本历史仓库的问题可在[本仓 Issue](https://github.com/linkaiheng2233-cyber/oclive-launcher/issues)记录。请不要将仓库中的版本检查入口理解为已经发布了对应安装包。


<!-- Preserve previously published section bookmarks. -->
<a id="oclive-启动器"></a>
<a id="生态仓库github"></a>
<a id="新用户请改用编写器--运行时"></a>
<a id="你可以提前准备的材料可选"></a>
<a id="githubownerrepo是什么不是法律协议是使用约定"></a>
<a id="ollama程序下载-vs-模型下载"></a>
<a id="仅玩家整合包与-release规划向"></a>
<a id="附带-ollama-安装包windows与启动器同发"></a>
<a id="发版git-与大文件ollamasetupexe"></a>
<a id="功能概览"></a>
<a id="仓库内的远程开发者公告示例"></a>
<a id="随包寄语与职责边界创作者公告"></a>
<a id="与整合包--一键装模型等方向的路线长期"></a>
<a id="环境"></a>
<a id="命令"></a>
<a id="配置存储"></a>
<a id="与双开浏览器的关系"></a>
<a id="贡献与发版"></a>
<a id="许可"></a>

旧版章节书签保留在此。[整理前的首页](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/adea5c5e5fc499c3e4659fa4291e37419d04483e/README.md)可用于查阅历史说明；现行入口以本页与所链文档为准。
