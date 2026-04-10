//! 从编写器导出的 zip（`角色id/manifest.json` 结构）解压到 `roles` 根目录。

use std::fs::{self, File};
use std::io;
use std::path::Path;

use serde_json::{json, Value};
use zip::read::ZipArchive;

fn normalize_zip_path(name: &str) -> String {
    name.replace('\\', "/").trim_start_matches('/').to_string()
}

fn detect_role_id_from_names(names: &[String]) -> Result<String, String> {
    for name in names {
        let n = normalize_zip_path(name);
        if n.ends_with('/') {
            continue;
        }
        if let Some((a, b)) = n.rsplit_once('/') {
            if b == "manifest.json" && !a.contains('/') && !a.is_empty() {
                return Ok(a.to_string());
            }
        }
    }
    Err("未在 zip 内找到 角色id/manifest.json（请使用编写器导出的角色包）".into())
}

/// 解压到 `roles_root/角色id/`，返回角色 id。
pub fn extract_role_pack_zip(zip_path: &Path, roles_root: &Path) -> Result<String, String> {
    let zf = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(zf).map_err(|e| e.to_string())?;
    let mut names = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let f = archive.by_index(i).map_err(|e| e.to_string())?;
        names.push(f.name().to_string());
    }
    let role_id = detect_role_id_from_names(&names)?;
    drop(archive);

    let zf = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(zf).map_err(|e| e.to_string())?;
    let dest_base = roles_root.join(&role_id);
    fs::create_dir_all(&dest_base).map_err(|e| e.to_string())?;
    let dest_can = fs::canonicalize(&dest_base).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let raw_name = normalize_zip_path(file.name());
        if raw_name.ends_with('/') {
            continue;
        }
        let prefix = format!("{}/", role_id);
        if !raw_name.starts_with(&prefix) {
            return Err(format!("zip 结构异常：{} 不在 {}/ 下", raw_name, role_id));
        }
        let rel = &raw_name[prefix.len()..];
        if rel.contains("..") {
            return Err("zip 含非法路径".into());
        }
        let out_path = dest_base.join(rel);
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let out_can = fs::canonicalize(&out_path).unwrap_or_else(|_| out_path.clone());
        if !out_can.starts_with(&dest_can) {
            return Err("zip 路径越界".into());
        }
        let mut out_f = fs::File::create(&out_path).map_err(|e| e.to_string())?;
        io::copy(&mut file, &mut out_f).map_err(|e| e.to_string())?;
    }

    Ok(role_id)
}

pub fn patch_settings_model(path: &Path, model: &str, overwrite: bool) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }
    let s = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut v: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
    let obj = v
        .as_object_mut()
        .ok_or_else(|| "settings.json 根须为 JSON 对象".to_string())?;
    let has = obj
        .get("model")
        .and_then(|x| x.as_str())
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false)
        || obj
            .get("ollama_model")
            .and_then(|x| x.as_str())
            .map(|t| !t.trim().is_empty())
            .unwrap_or(false);
    if has && !overwrite {
        return Ok(());
    }
    obj.insert("model".into(), json!(model));
    let out = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(())
}
