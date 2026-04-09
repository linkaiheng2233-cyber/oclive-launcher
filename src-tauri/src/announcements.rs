//! 启动器内「公告」相关：开发者 `announcements.md`、远程拉取，以及遗留的 `creator-announcements.md` API。
//! 与主流程（配置、子进程）解耦，便于单独维护。

use std::path::{Path, PathBuf};
use std::time::Duration;

fn app_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path_resolver()
        .app_config_dir()
        .ok_or_else(|| "无法解析应用配置目录".to_string())
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn developer_announcements_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join("announcements.md"))
}

fn legacy_creator_sticky_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join("creator-announcements.md"))
}

#[tauri::command]
pub fn load_maintainer_announcements(app: tauri::AppHandle) -> Result<String, String> {
    let path = developer_announcements_path(&app)?;
    if !path.exists() {
        return Ok("# 开发者公告\n\n（本地 `announcements.md` 尚无内容。可在配置目录创建该文件，或填写「开发者公告 URL」后点「拉取最新」。）\n".into());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_maintainer_announcements(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let path = developer_announcements_path(&app)?;
    ensure_parent_dir(&path)?;
    std::fs::write(&path, text.as_bytes()).map_err(|e| e.to_string())
}

/// 从公开 URL 拉取正文（UTF-8）；成功后由前端再调用 `save_maintainer_announcements` 写入本地缓存。
#[tauri::command]
pub fn fetch_remote_announcement_text(url: String) -> Result<String, String> {
    const MAX_BYTES: usize = 512 * 1024;
    let u = url.trim();
    if u.is_empty() {
        return Err("URL 为空".into());
    }
    let lower = u.to_ascii_lowercase();
    if !lower.starts_with("https://") && !lower.starts_with("http://") {
        return Err("仅支持以 http:// 或 https:// 开头的地址".into());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(u).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err(format!("正文超过 {} 字节", MAX_BYTES));
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| "响应体不是合法 UTF-8".into())
}

#[tauri::command]
pub fn load_creator_announcements(app: tauri::AppHandle) -> Result<String, String> {
    let path = legacy_creator_sticky_path(&app)?;
    if !path.exists() {
        return Ok("# 创作者公告\n\n在这里写创作者想对用户说的话（支持 Markdown 纯文本展示）。\n".into());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_creator_announcements(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let path = legacy_creator_sticky_path(&app)?;
    ensure_parent_dir(&path)?;
    std::fs::write(&path, text.as_bytes()).map_err(|e| e.to_string())
}
