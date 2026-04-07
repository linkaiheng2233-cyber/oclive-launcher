#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod role_pack;

use serde::{Deserialize, Serialize};
use tauri::Manager;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherConfig {
    #[serde(default)]
    editor_project_root: String,
    #[serde(default)]
    editor_exe: String,
    #[serde(default)]
    editor_mode: String,
    #[serde(default = "default_npm")]
    editor_npm_script: String,
    #[serde(default)]
    oclive_project_root: String,
    #[serde(default)]
    oclive_exe: String,
    #[serde(default)]
    oclive_mode: String,
    #[serde(default = "default_npm")]
    oclive_npm_script: String,
    #[serde(default)]
    github_editor_owner: String,
    #[serde(default)]
    github_editor_repo: String,
    #[serde(default)]
    github_oclive_owner: String,
    #[serde(default)]
    github_oclive_repo: String,
    /// 启动 oclive 时注入环境变量 `OCLIVE_ROLES_DIR`（须为已存在的目录：其下为各 `角色id/`）。
    #[serde(default)]
    oclive_roles_dir: String,
}

fn default_npm() -> String {
    "tauri:dev".to_string()
}

/// 上游仓库占位（用户可改为自己的 fork；仅当 owner+repo 均为空时由 `load_config` 填入）。
const UPSTREAM_GITHUB_OWNER: &str = "supermumu";
const UPSTREAM_EDITOR_REPO: &str = "oclive-pack-editor";
const UPSTREAM_OCLIVE_REPO: &str = "oclivenewnew";

fn ensure_github_upstream_defaults(c: &mut LauncherConfig) {
    if c.github_editor_owner.trim().is_empty() && c.github_editor_repo.trim().is_empty() {
        c.github_editor_owner = UPSTREAM_GITHUB_OWNER.into();
        c.github_editor_repo = UPSTREAM_EDITOR_REPO.into();
    }
    if c.github_oclive_owner.trim().is_empty() && c.github_oclive_repo.trim().is_empty() {
        c.github_oclive_owner = UPSTREAM_GITHUB_OWNER.into();
        c.github_oclive_repo = UPSTREAM_OCLIVE_REPO.into();
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        let mut s = Self {
            editor_project_root: String::new(),
            editor_exe: String::new(),
            editor_mode: "dev".into(),
            editor_npm_script: default_npm(),
            oclive_project_root: String::new(),
            oclive_exe: String::new(),
            oclive_mode: "dev".into(),
            oclive_npm_script: default_npm(),
            github_editor_owner: String::new(),
            github_editor_repo: String::new(),
            github_oclive_owner: String::new(),
            github_oclive_repo: String::new(),
            oclive_roles_dir: String::new(),
        };
        ensure_github_upstream_defaults(&mut s);
        s
    }
}

struct AppState {
    editor: Arc<Mutex<Option<Child>>>,
    oclive: Arc<Mutex<Option<Child>>>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path_resolver()
        .app_config_dir()
        .ok_or_else(|| "无法解析应用配置目录".to_string())?;
    Ok(dir.join("launcher-config.json"))
}

fn announcements_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path_resolver()
        .app_config_dir()
        .ok_or_else(|| "无法解析应用配置目录".to_string())?;
    Ok(dir.join("announcements.md"))
}

#[tauri::command]
fn load_config(app: tauri::AppHandle) -> Result<LauncherConfig, String> {
    let path = config_path(&app)?;
    if !path.exists() {
        return Ok(LauncherConfig::default());
    }
    let s = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    match serde_json::from_str::<LauncherConfig>(&s) {
        Ok(mut c) => {
            ensure_github_upstream_defaults(&mut c);
            Ok(c)
        }
        Err(_) => {
            let bak = path.with_extension("json.corrupt.bak");
            let _ = std::fs::copy(&path, &bak);
            Ok(LauncherConfig::default())
        }
    }
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: LauncherConfig) -> Result<(), String> {
    let path = config_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let s = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, s).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_announcements(app: tauri::AppHandle) -> Result<String, String> {
    let path = announcements_path(&app)?;
    if !path.exists() {
        return Ok("# 公告\n\n在这里写面向创作者的通知（支持 Markdown 显示为纯文本）。\n".into());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_announcements(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let path = announcements_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
fn pick_folder() -> Option<String> {
    tauri::api::dialog::blocking::FileDialogBuilder::new()
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn pick_exe() -> Option<String> {
    tauri::api::dialog::blocking::FileDialogBuilder::new()
        .add_filter("可执行文件", &["exe"])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn read_package_version(project_root: String) -> Option<String> {
    let path = PathBuf::from(project_root).join("package.json");
    let s = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&s).ok()?;
    v.get("version")?.as_str().map(|s| s.to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseInfo {
    tag_name: String,
    name: Option<String>,
    html_url: String,
    published_at: Option<String>,
    body: Option<String>,
}

#[tauri::command]
fn fetch_github_release(owner: String, repo: String) -> Result<ReleaseInfo, String> {
    if owner.trim().is_empty() || repo.trim().is_empty() {
        return Err("请填写 GitHub owner 与仓库名".into());
    }
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner.trim(),
        repo.trim()
    );
    let client = reqwest::blocking::Client::builder()
        .user_agent("oclive-launcher/0.1")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("GitHub 返回 {}（仓库可能为私有或尚无 Release）", resp.status()));
    }
    let v: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    Ok(ReleaseInfo {
        tag_name: v
            .get("tag_name")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string(),
        name: v
            .get("name")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        html_url: v
            .get("html_url")
            .and_then(|x| x.as_str())
            .unwrap_or("#")
            .to_string(),
        published_at: v
            .get("published_at")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        body: v
            .get("body")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
    })
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if url.is_empty() {
        return Err("空链接".into());
    }
    open::that(&url).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvDiagnostics {
    node_version: Option<String>,
    npm_version: Option<String>,
    ollama_version: Option<String>,
    ollama_api_reachable: bool,
    editor_project_ok: bool,
    editor_package_json: bool,
    oclive_project_ok: bool,
    oclive_package_json: bool,
    oclive_roles_dir_ok: bool,
    oclive_roles_dir_has_role_hint: bool,
}

fn try_cmd_version(program: &str, args: &[&str]) -> Option<String> {
    let mut c = Command::new(program);
    c.args(args);
    c.stdout(Stdio::piped());
    c.stderr(Stdio::piped());
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    let out = c.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let a = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let b = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let merged = if !a.is_empty() { a } else { b };
    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn ollama_api_reachable() -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn roles_dir_looks_populated(root: &Path) -> bool {
    if !root.is_dir() {
        return false;
    }
    std::fs::read_dir(root).ok().map_or(false, |rd| {
        rd.flatten().any(|e| {
            let p = e.path();
            p.is_dir() && p.join("manifest.json").is_file()
        })
    })
}

fn dir_has_package_json(root: &str) -> (bool, bool) {
    let p = PathBuf::from(root.trim());
    if !p.is_dir() {
        return (false, false);
    }
    let pkg = p.join("package.json");
    (true, pkg.is_file())
}

/// 检测 Node / npm / Ollama 与当前填写的项目路径（便于「傻瓜化」排障，对标一键向导思路）。
#[tauri::command]
fn diagnose_environment(config: LauncherConfig) -> EnvDiagnostics {
    let node = try_cmd_version("node", &["--version"]);
    let npm = try_cmd_version("npm", &["--version"]);
    let ollama_v = try_cmd_version("ollama", &["--version"]);
    let ollama_api = ollama_api_reachable();
    let (ed_ok, ed_pkg) = if config.editor_project_root.trim().is_empty() {
        (false, false)
    } else {
        dir_has_package_json(&config.editor_project_root)
    };
    let (oc_ok, oc_pkg) = if config.oclive_project_root.trim().is_empty() {
        (false, false)
    } else {
        dir_has_package_json(&config.oclive_project_root)
    };
    let rd = config.oclive_roles_dir.trim();
    let roles_path = PathBuf::from(rd);
    let (roles_ok, roles_hint) = if rd.is_empty() {
        (false, false)
    } else if roles_path.is_dir() {
        (true, roles_dir_looks_populated(&roles_path))
    } else {
        (false, false)
    };
    EnvDiagnostics {
        node_version: node,
        npm_version: npm,
        ollama_version: ollama_v,
        ollama_api_reachable: ollama_api,
        editor_project_ok: ed_ok,
        editor_package_json: ed_pkg,
        oclive_project_ok: oc_ok,
        oclive_package_json: oc_pkg,
        oclive_roles_dir_ok: roles_ok,
        oclive_roles_dir_has_role_hint: roles_hint,
    }
}

#[tauri::command]
fn reset_config_to_default(app: tauri::AppHandle) -> Result<LauncherConfig, String> {
    let c = LauncherConfig::default();
    save_config(app, c.clone())?;
    Ok(c)
}

/// 若 `oclivenewnew` 仓库根下存在 `roles/` 目录，返回其绝对路径字符串，供一键填入。
#[tauri::command]
fn suggest_roles_dir_from_oclive_root(oclive_project_root: String) -> Option<String> {
    let root = PathBuf::from(oclive_project_root.trim());
    if !root.is_dir() {
        return None;
    }
    let roles = root.join("roles");
    if roles.is_dir() {
        std::fs::canonicalize(&roles)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    } else {
        None
    }
}

#[tauri::command]
fn open_config_directory(app: tauri::AppHandle) -> Result<(), String> {
    let dir = app
        .path_resolver()
        .app_config_dir()
        .ok_or_else(|| "无法解析应用配置目录".to_string())?;
    if !dir.is_dir() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    open::that(&dir).map_err(|e| e.to_string())
}

fn validate_ollama_model_name(s: &str) -> Result<(), String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("模型名为空".into());
    }
    if t.len() > 200 {
        return Err("模型名过长".into());
    }
    if !t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ":._-/".contains(c))
    {
        return Err("模型名含非法字符（仅允许 ASCII 字母数字与 : . _ - /）".into());
    }
    Ok(())
}

#[tauri::command]
fn pick_role_pack_zip() -> Option<String> {
    tauri::api::dialog::blocking::FileDialogBuilder::new()
        .add_filter("角色包", &["zip", "ocpak"])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}

/// 本机 Ollama 已拉取的模型名（`GET /api/tags`）；服务未启动时会报错。
#[tauri::command]
fn ollama_list_local_models() -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .map_err(|e| format!("无法连接 Ollama（是否已启动？）：{}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Ollama API 返回 {}", resp.status()));
    }
    let v: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let models = v
        .get("models")
        .and_then(|m| m.as_array())
        .ok_or_else(|| "响应中无 models".to_string())?;
    let mut names: Vec<String> = models
        .iter()
        .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

#[tauri::command]
fn install_role_pack_zip(
    zip_path: String,
    roles_root: String,
    model: String,
    overwrite_settings_model: bool,
) -> Result<String, String> {
    let zp = PathBuf::from(zip_path.trim());
    if !zp.is_file() {
        return Err("zip 文件不存在".into());
    }
    let root = PathBuf::from(roles_root.trim());
    if !root.is_dir() {
        return Err("请先填写有效的「角色包根目录」（须为已存在的文件夹）".into());
    }
    let model = model.trim();
    validate_ollama_model_name(model)?;
    let role_id = role_pack::extract_role_pack_zip(&zp, &root)?;
    let settings = root.join(&role_id).join("settings.json");
    role_pack::patch_settings_model(&settings, model, overwrite_settings_model)?;
    Ok(role_id)
}

#[tauri::command]
fn ollama_pull_model(app: tauri::AppHandle, model: String) -> Result<(), String> {
    let model = model.trim().to_string();
    validate_ollama_model_name(&model)?;
    let app2 = app.clone();
    thread::spawn(move || {
        emit_log(
            &app2,
            "ollama",
            "out",
            &format!("--- 开始 ollama pull {} ---", model),
        );
        let mut cmd = Command::new("ollama");
        cmd.args(["pull", &model]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                emit_log(
                    &app2,
                    "ollama",
                    "err",
                    &format!("启动 ollama pull 失败：{}", e),
                );
                return;
            }
        };
        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                emit_log(&app2, "ollama", "err", "无法读取 ollama stdout");
                return;
            }
        };
        let stderr = match child.stderr.take() {
            Some(s) => s,
            None => {
                emit_log(&app2, "ollama", "err", "无法读取 ollama stderr");
                return;
            }
        };
        let app_o = app2.clone();
        let h1 = thread::spawn(move || {
            for line in BufReader::new(stdout).lines().flatten() {
                emit_log(&app_o, "ollama", "out", &line);
            }
        });
        let app_e = app2.clone();
        let h2 = thread::spawn(move || {
            for line in BufReader::new(stderr).lines().flatten() {
                emit_log(&app_e, "ollama", "err", &line);
            }
        });
        let _ = child.wait();
        let _ = h1.join();
        let _ = h2.join();
        emit_log(&app2, "ollama", "out", "--- ollama pull 已结束 ---");
    });
    Ok(())
}

fn validate_npm_script(s: &str) -> Result<(), String> {
    if s.is_empty() || s.len() > 80 {
        return Err("npm 脚本名长度无效".into());
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '-')
    {
        return Err("npm 脚本名含非法字符".into());
    }
    Ok(())
}

/// 若配置了 `oclive_roles_dir`，为子进程设置 `OCLIVE_ROLES_DIR`（绝对路径）。
fn apply_oclive_roles_env(cmd: &mut Command, config: &LauncherConfig) -> Result<(), String> {
    let t = config.oclive_roles_dir.trim();
    if t.is_empty() {
        return Ok(());
    }
    let p = PathBuf::from(t);
    if !p.is_dir() {
        return Err(format!(
            "角色包根目录无效（须为已存在的文件夹）：{}",
            t
        ));
    }
    let abs = std::fs::canonicalize(&p).unwrap_or(p);
    cmd.env("OCLIVE_ROLES_DIR", abs);
    Ok(())
}

fn emit_log(app: &tauri::AppHandle, app_id: &str, stream: &str, line: &str) {
    let line = if line.len() > 16_000 {
        format!("{}…", &line[..16_000])
    } else {
        line.to_string()
    };
    let _ = app.emit_all(
        "launcher-log",
        serde_json::json!({
            "app": app_id,
            "stream": stream,
            "line": line,
        }),
    );
}

fn pipe_stream<R: std::io::Read + Send + 'static>(
    app: tauri::AppHandle,
    app_id: String,
    stream: String,
    reader: R,
) {
    thread::spawn(move || {
        let br = BufReader::new(reader);
        for line in br.lines().flatten() {
            emit_log(&app, &app_id, &stream, &line);
        }
    });
}

fn wait_child(slot: Arc<Mutex<Option<Child>>>, app: tauri::AppHandle, app_id: String) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(400));
            let done = {
                let mut g = match slot.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                let Some(ref mut child) = *g else {
                    return;
                };
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let code = status.code();
                        *g = None;
                        Some(code)
                    }
                    Ok(None) => None,
                    Err(_) => {
                        *g = None;
                        Some(None)
                    }
                }
            };
            if let Some(code) = done {
                let _ = app.emit_all(
                    "launcher-exit",
                    serde_json::json!({ "app": app_id, "code": code }),
                );
                return;
            }
        }
    });
}

#[tauri::command]
fn spawn_managed_app(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    kind: String,
    config: LauncherConfig,
) -> Result<(), String> {
    let slot: Arc<Mutex<Option<Child>>> = match kind.as_str() {
        "editor" => Arc::clone(&state.editor),
        "oclive" => Arc::clone(&state.oclive),
        _ => return Err("未知应用：请使用 editor 或 oclive".into()),
    };

    {
        let mut g = slot.lock().map_err(|e| e.to_string())?;
        if let Some(mut c) = g.take() {
            let _ = c.kill();
        }
    }

    let is_exe = match kind.as_str() {
        "editor" => config.editor_mode == "exe",
        "oclive" => config.oclive_mode == "exe",
        _ => false,
    };

    let mut child = if is_exe {
        let (exe, cwd) = match kind.as_str() {
            "editor" => {
                if config.editor_exe.trim().is_empty() {
                    return Err("未设置编写器可执行文件路径".into());
                }
                let p = PathBuf::from(config.editor_exe.trim());
                if !p.is_file() {
                    return Err("编写器可执行文件不存在".into());
                }
                let cwd = p
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from("."));
                (p, cwd)
            }
            "oclive" => {
                if config.oclive_exe.trim().is_empty() {
                    return Err("未设置 oclive 可执行文件路径".into());
                }
                let p = PathBuf::from(config.oclive_exe.trim());
                if !p.is_file() {
                    return Err("oclive 可执行文件不存在".into());
                }
                let cwd = p
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from("."));
                (p, cwd)
            }
            _ => unreachable!(),
        };
        let mut cmd = Command::new(&exe);
        cmd.current_dir(&cwd);
        if kind == "oclive" {
            apply_oclive_roles_env(&mut cmd, &config)?;
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().map_err(|e| format!("启动失败：{}", e))?
    } else {
        let (root, npm_script) = match kind.as_str() {
            "editor" => {
                if config.editor_project_root.trim().is_empty() {
                    return Err("未设置编写器项目根目录".into());
                }
                validate_npm_script(&config.editor_npm_script)?;
                (
                    PathBuf::from(config.editor_project_root.trim()),
                    config.editor_npm_script.clone(),
                )
            }
            "oclive" => {
                if config.oclive_project_root.trim().is_empty() {
                    return Err("未设置 oclive 项目根目录".into());
                }
                validate_npm_script(&config.oclive_npm_script)?;
                (
                    PathBuf::from(config.oclive_project_root.trim()),
                    config.oclive_npm_script.clone(),
                )
            }
            _ => unreachable!(),
        };
        if !root.is_dir() {
            return Err("项目目录不存在或不是文件夹".into());
        }
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "npm", "run", &npm_script]);
        cmd.current_dir(&root);
        if kind == "oclive" {
            apply_oclive_roles_env(&mut cmd, &config)?;
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().map_err(|e| format!("启动失败：{}", e))?
    };

    let stdout = child.stdout.take().ok_or("无法读取子进程 stdout")?;
    let stderr = child.stderr.take().ok_or("无法读取子进程 stderr")?;

    {
        let mut g = slot.lock().map_err(|e| e.to_string())?;
        *g = Some(child);
    }

    let app_h = app.clone();
    let kid = kind.clone();
    pipe_stream(app_h.clone(), kid.clone(), "out".into(), stdout);
    let app_h2 = app.clone();
    pipe_stream(app_h2, kid.clone(), "err".into(), stderr);

    wait_child(Arc::clone(&slot), app.clone(), kind.clone());

    emit_log(
        &app,
        match kind.as_str() {
            "editor" => "editor",
            _ => "oclive",
        },
        "out",
        "--- 进程已启动（无单独终端窗口，日志见下方）---",
    );

    Ok(())
}

#[tauri::command]
fn stop_managed_app(state: tauri::State<AppState>, kind: String) -> Result<(), String> {
    let slot: &Arc<Mutex<Option<Child>>> = match kind.as_str() {
        "editor" => &state.editor,
        "oclive" => &state.oclive,
        _ => return Err("未知应用".into()),
    };
    let mut g = slot.lock().map_err(|e| e.to_string())?;
    if let Some(mut c) = g.take() {
        let _ = c.kill();
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            editor: Arc::new(Mutex::new(None)),
            oclive: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            load_announcements,
            save_announcements,
            pick_folder,
            pick_exe,
            read_package_version,
            fetch_github_release,
            open_url,
            diagnose_environment,
            reset_config_to_default,
            open_config_directory,
            suggest_roles_dir_from_oclive_root,
            pick_role_pack_zip,
            ollama_list_local_models,
            install_role_pack_zip,
            ollama_pull_model,
            spawn_managed_app,
            stop_managed_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running oclive-launcher");
}
