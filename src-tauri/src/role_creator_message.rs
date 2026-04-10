//! `roles/{id}/creator_message.txt`：与编写器导出约定一致，供启动器只读展示「创作者公告」。
//! 文件可含多行，每行一条；一句模式导出时为单行。
//!
//! 字数上限与文件名须与 **oclive-pack-editor** `rolePackCreatorMessage.ts`、启动器前端
//! `src/lib/rolePackCreatorMessage.ts` 保持同步。

use std::fs;
use std::path::{Path, PathBuf};

const ROLE_CREATOR_MESSAGE_MAX_CHARS: usize = 160;
const ROLE_CREATOR_MESSAGE_FILENAME: &str = "creator_message.txt";

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn validate_roles_subdir_name(id: &str) -> Result<(), String> {
    let t = id.trim();
    if t.is_empty() {
        return Err("角色 id 不能为空".into());
    }
    if t == "." || t == ".." {
        return Err("角色 id 非法".into());
    }
    if t.contains('/') || t.contains('\\') || t.contains('\0') {
        return Err("角色 id 含非法字符".into());
    }
    Ok(())
}

fn normalize_role_creator_message_line(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("寄语不能为空".into());
    }
    let line = text
        .lines()
        .next()
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    if line.is_empty() {
        return Err("寄语不能为空".into());
    }
    if line.chars().count() > ROLE_CREATOR_MESSAGE_MAX_CHARS {
        return Err(format!(
            "寄语请在 {} 字以内",
            ROLE_CREATOR_MESSAGE_MAX_CHARS
        ));
    }
    Ok(line)
}

/// 列出 `roles` 根含 `manifest.json` 的子目录名（角色 id）。
#[tauri::command]
pub fn list_role_ids_with_manifest(roles_root: String) -> Result<Vec<String>, String> {
    let root = PathBuf::from(roles_root.trim());
    if roles_root.trim().is_empty() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        return Err("角色包根目录不存在或不是文件夹".into());
    }
    let mut out: Vec<String> = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ft = entry.file_type().map_err(|e| e.to_string())?;
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if entry.path().join("manifest.json").is_file() {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

/// 读取 `roles/{role_id}/creator_message.txt`：每个非空行视为一条寄语（一句模式为单行；多模块拼接为多行）。
#[tauri::command]
pub fn read_role_creator_message_lines(
    roles_root: String,
    role_id: String,
) -> Result<Vec<String>, String> {
    if roles_root.trim().is_empty() {
        return Ok(Vec::new());
    }
    validate_roles_subdir_name(&role_id)?;
    let root = PathBuf::from(roles_root.trim());
    let path = root
        .join(role_id.trim())
        .join(ROLE_CREATOR_MESSAGE_FILENAME);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut out: Vec<String> = Vec::new();
    for line in s.lines() {
        let t = line.trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
    }
    Ok(out)
}

/// 写入 `roles/{role_id}/creator_message.txt`（单行）；目录须已存在。
#[tauri::command]
pub fn write_role_creator_message(
    roles_root: String,
    role_id: String,
    text: String,
) -> Result<(), String> {
    validate_roles_subdir_name(&role_id)?;
    let line = normalize_role_creator_message_line(&text)?;
    let root = PathBuf::from(roles_root.trim());
    if roles_root.trim().is_empty() {
        return Err("请先填写角色包根目录".into());
    }
    let role_dir = root.join(role_id.trim());
    if !role_dir.is_dir() {
        return Err("角色目录不存在：请先将角色包安装或导出到该 roles 根下".into());
    }
    let path = role_dir.join(ROLE_CREATOR_MESSAGE_FILENAME);
    ensure_parent_dir(&path)?;
    fs::write(&path, format!("{}\n", line)).map_err(|e| e.to_string())
}
