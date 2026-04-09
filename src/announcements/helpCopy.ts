/** 新手页「创作者公告」问号文案（分段展示） */
export const CREATOR_ANNOUNCE_HINT_PARAGRAPHS = [
  '内容来自角色包里的 creator_message.txt，作者在 oclive-pack-editor 导出时写入。',
  '可「整包一句」或「按行多条」：多模块各留一句时，启动器会逐条列出。',
  '此处只读。用下方选择角色；对应配置 launcherEchoRoleId（launcher-config.json）。路径：roles/<角色id>/creator_message.txt。',
  '「取消跟随」只会清空当前选中的角色（配置里不再跟随），不会删除磁盘上的角色包或 creator_message.txt。',
  'oclive 对话不会读取该文件。',
] as const

/** 新手页「开发者公告」问号文案 */
export const DEVELOPER_ANNOUNCE_HINT_PARAGRAPHS = [
  '给使用本启动器与工具链的人看的说明：更新提示、兼容与排障等。',
  '默认读取应用配置目录下的 announcements.md。',
  '远程地址必须是「点开就能下载到纯文本正文」的链接，不能填 GitHub 仓库主页。',
  '常见做法：在 GitHub 打开仓库里的 .md → Raw → 复制地址栏（多为 raw.githubusercontent.com/…）。也可用 Pages 等直链。',
  '填好 URL 后先点右上角「保存配置」，再点「拉取最新」。不是自动推送，需手动拉取或定期打开再点。',
] as const
