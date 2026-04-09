//! 从 GitHub Release 列出附件、另存为下载、zip 便携包解压并猜测主程序 exe。

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::Manager;
use zip::read::ZipArchive;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GhReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GhDownloadResult {
    pub saved_path: String,
    pub resolved_exe: Option<String>,
    pub hint: Option<String>,
}

fn github_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(concat!("oclive-launcher/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())
}

fn emit_dl_log(app: &tauri::AppHandle, app_id: &str, line: &str) {
    let line = if line.len() > 16_000 {
        format!("{}…", &line[..16_000])
    } else {
        line.to_string()
    };
    let _ = app.emit_all(
        "launcher-log",
        serde_json::json!({
            "app": app_id,
            "stream": "out",
            "line": line,
        }),
    );
}

fn normalize_zip_entry(name: &str) -> String {
    name.replace('\\', "/").trim_start_matches('/').to_string()
}

fn safe_join_under(base: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = normalize_zip_entry(rel);
    if rel.is_empty() || rel.ends_with('/') {
        return Err("zip 条目无效".into());
    }
    if rel.contains("..") {
        return Err("zip 含非法路径".into());
    }
    let mut out = base.to_path_buf();
    for seg in rel.split('/') {
        if seg.is_empty() {
            continue;
        }
        if seg == ".." {
            return Err("zip 含非法路径".into());
        }
        out.push(seg);
    }
    Ok(out)
}

/// 将便携 zip 解压到 `out_dir`（须已存在），带路径穿越检查。
pub fn extract_portable_zip(zip_path: &Path, out_dir: &Path) -> Result<(), String> {
    let base_can = fs::canonicalize(out_dir).map_err(|e| e.to_string())?;
    let zf = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(zf).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let raw = normalize_zip_entry(file.name());
        if raw.is_empty() {
            continue;
        }
        if raw.ends_with('/') {
            let dir = safe_join_under(out_dir, raw.trim_end_matches('/'))?;
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            continue;
        }
        let target = safe_join_under(out_dir, &raw)?;
        if let Some(par) = target.parent() {
            fs::create_dir_all(par).map_err(|e| e.to_string())?;
        }
        let par_can = fs::canonicalize(target.parent().ok_or_else(|| "zip 路径无效".to_string())?)
            .map_err(|e| e.to_string())?;
        if !par_can.starts_with(&base_can) {
            return Err("zip 路径越界".into());
        }
        let mut out_f = File::create(&target).map_err(|e| e.to_string())?;
        io::copy(&mut file, &mut out_f).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn exe_name_score(file_name: &str, kind: &str) -> i32 {
    let n = file_name.to_lowercase();
    if n.contains("uninstall") || n.contains("updater") || n.contains("update.exe") {
        return -100;
    }
    let mut s = 10;
    match kind {
        "oclive" => {
            if n.contains("oclive") {
                s += 50;
            }
        }
        "editor" => {
            if n.contains("pack") || n.contains("editor") {
                s += 50;
            }
        }
        _ => {}
    }
    s
}

/// 在目录树中（浅层 BFS）寻找最像主程序的 `.exe`。
pub fn find_best_exe_under(root: &Path, kind: &str, max_depth: usize) -> Option<PathBuf> {
    let root = fs::canonicalize(root).ok()?;
    let mut best: Option<(i32, PathBuf)> = None;
    let mut q = VecDeque::new();
    q.push_back((root.clone(), 0usize));
    while let Some((dir, depth)) = q.pop_front() {
        if depth > max_depth {
            continue;
        }
        let rd = fs::read_dir(&dir).ok()?;
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                q.push_back((p, depth + 1));
                continue;
            }
            if p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("exe"))
                != Some(true)
            {
                continue;
            }
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let sc = exe_name_score(name, kind);
            if sc < 0 {
                continue;
            }
            let replace = match &best {
                None => true,
                Some((b, _)) => sc > *b,
            };
            if replace {
                best = Some((sc, p));
            }
        }
    }
    best.map(|(_, p)| p)
}

fn looks_like_windows_installer(path: &Path) -> bool {
    let n = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if n.ends_with(".msi") {
        return true;
    }
    if !n.ends_with(".exe") {
        return false;
    }
    n.contains("setup")
        || n.contains("nsis")
        || n.contains("_setup")
        || n.contains("-setup")
        || n.contains("installer")
}

fn validate_github_asset_url(url: &str) -> Result<(), String> {
    let u = url.trim();
    if !u.starts_with("https://") {
        return Err("仅支持 https 下载链接".into());
    }
    let ok = u.starts_with("https://github.com/")
        || u.starts_with("https://objects.githubusercontent.com/")
        || u.starts_with("https://release-assets.githubusercontent.com/");
    if !ok {
        return Err("下载地址须为 GitHub Release 资源（github.com / githubusercontent.com）".into());
    }
    Ok(())
}

#[tauri::command]
pub fn gh_latest_release_assets(owner: String, repo: String) -> Result<Vec<GhReleaseAsset>, String> {
    let owner = owner.trim();
    let repo = repo.trim();
    if owner.is_empty() || repo.is_empty() {
        return Err("请填写 GitHub owner 与仓库名（可在「版本」或配置里改）".into());
    }
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let client = github_http_client()?;
    let resp = client
        .get(&api_url)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "GitHub 返回 {}（仓库可能为私有或尚无 Release）",
            resp.status()
        ));
    }
    let v: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let Some(assets) = v.get("assets").and_then(|a| a.as_array()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for a in assets {
        let name = a
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let browser_download_url = a
            .get("browser_download_url")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let size = a.get("size").and_then(|x| x.as_u64()).unwrap_or(0);
        if name.is_empty() || browser_download_url.is_empty() {
            continue;
        }
        out.push(GhReleaseAsset {
            name,
            browser_download_url,
            size,
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn gh_download_release_asset(
    app: tauri::AppHandle,
    url: String,
    suggested_file_name: String,
    kind: String,
) -> Result<GhDownloadResult, String> {
    let kind = kind.trim().to_lowercase();
    if kind != "oclive" && kind != "editor" {
        return Err("kind 须为 oclive 或 editor".into());
    }
    let url = url.trim().to_string();
    validate_github_asset_url(&url)?;

    let suggested = suggested_file_name.trim();
    let default_name = if suggested.is_empty() {
        "download.bin".to_string()
    } else {
        suggested.to_string()
    };

    let dest = tauri::api::dialog::blocking::FileDialogBuilder::new()
        .set_file_name(&default_name)
        .save_file()
        .ok_or_else(|| "已取消保存".to_string())?;
    let dest_str = dest.to_string_lossy().into_owned();

    emit_dl_log(
        &app,
        match kind.as_str() {
            "oclive" => "oclive",
            _ => "editor",
        },
        &format!("开始下载：{}", url),
    );

    let client = github_http_client()?;
    let mut resp = client.get(&url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("下载失败：HTTP {}", resp.status()));
    }

    let mut f = File::create(&dest).map_err(|e| e.to_string())?;
    let n = io::copy(&mut resp, &mut f).map_err(|e| e.to_string())?;
    drop(f);

    emit_dl_log(
        &app,
        match kind.as_str() {
            "oclive" => "oclive",
            _ => "editor",
        },
        &format!("已保存 {}（{} 字节）", dest_str, n),
    );

    let path = PathBuf::from(&dest_str);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "zip" {
        let parent = path.parent().ok_or_else(|| "无法解析保存路径".to_string())?;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("extracted");
        let out_dir = parent.join(format!("{}_extracted", stem));
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        emit_dl_log(
            &app,
            match kind.as_str() {
                "oclive" => "oclive",
                _ => "editor",
            },
            &format!("正在解压到：{}", out_dir.display()),
        );
        extract_portable_zip(&path, &out_dir).map_err(|e| format!("解压失败：{}", e))?;
        let found = find_best_exe_under(&out_dir, kind.as_str(), 6);
        let resolved_exe = found.map(|p| p.to_string_lossy().into_owned());
        let hint = if resolved_exe.is_none() {
            Some("已解压 zip，但未自动找到主程序 exe，请在下方点「浏览」手动选择。".into())
        } else {
            None
        };
        return Ok(GhDownloadResult {
            saved_path: dest_str,
            resolved_exe,
            hint,
        });
    }

    if looks_like_windows_installer(&path) {
        return Ok(GhDownloadResult {
            saved_path: dest_str,
            resolved_exe: None,
            hint: Some(
                "这是安装包（setup/msi）。请先运行并完成安装，再在下方点「浏览」选择安装目录里的主程序 exe。"
                    .into(),
            ),
        });
    }

    if ext == "exe" {
        return Ok(GhDownloadResult {
            saved_path: dest_str.clone(),
            resolved_exe: Some(dest_str),
            hint: None,
        });
    }

    Ok(GhDownloadResult {
        saved_path: dest_str,
        resolved_exe: None,
        hint: Some("已保存文件；若这不是可直接运行的 exe，请自行安装或解压后再浏览选择主程序。".into()),
    })
}
