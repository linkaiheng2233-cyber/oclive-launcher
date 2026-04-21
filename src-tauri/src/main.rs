#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod announcements;
mod oclive_env;
mod release_download;
mod role_creator_message;
mod role_pack;

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use tauri::Manager;

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
    /// 编写器「网页」模式下的地址；留空则使用 `https://{github_editor_owner}.github.io/{github_editor_repo}/`。
    #[serde(default)]
    editor_web_url: String,
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
    /// 启动器「随角色包寄语」跟随的 `roles` 子目录名（与 `manifest.json` 所在文件夹同名）；空表示不跟随。
    #[serde(default)]
    launcher_echo_role_id: String,
    /// `ollama` | `remote` — 注入 `OCLIVE_LLM_BACKEND`，运行时覆盖角色包内 `plugin_backends.llm`。
    #[serde(default = "default_oclive_llm_mode")]
    oclive_llm_mode: String,
    /// 云端 LLM：`OCLIVE_REMOTE_LLM_URL`（JSON-RPC 端点）。
    #[serde(default)]
    oclive_remote_llm_url: String,
    #[serde(default)]
    oclive_remote_llm_token: String,
    /// 可选：`OCLIVE_REMOTE_LLM_TIMEOUT_MS`（毫秒，正整数）。
    #[serde(default)]
    oclive_remote_llm_timeout_ms: String,
    /// 可选：`OCLIVE_REMOTE_PLUGIN_URL`（memory/emotion/event/prompt 共用 JSON-RPC 端点）。
    #[serde(default)]
    oclive_remote_plugin_url: String,
    /// 可选：`OCLIVE_REMOTE_PLUGIN_TOKEN`（Bearer）。
    #[serde(default)]
    oclive_remote_plugin_token: String,
    /// 可选：`OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS`（毫秒，正整数）。
    #[serde(default)]
    oclive_remote_plugin_timeout_ms: String,
    /// 开发者公告：可选 HTTPS/HTTP 地址，启动器可「拉取最新」覆盖本地 `announcements.md` 缓存。
    #[serde(default)]
    developer_announcements_url: String,
}

fn default_npm() -> String {
    "tauri:dev".to_string()
}

fn default_oclive_llm_mode() -> String {
    "ollama".to_string()
}

/// 上游仓库占位（用户可改为自己的 fork；仅当 owner+repo 均为空时由 `load_config` 填入）。
const UPSTREAM_GITHUB_OWNER: &str = "linkaiheng2233-cyber";
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
            editor_mode: "web".into(),
            editor_npm_script: default_npm(),
            editor_web_url: String::new(),
            oclive_project_root: String::new(),
            oclive_exe: String::new(),
            oclive_mode: "dev".into(),
            oclive_npm_script: default_npm(),
            github_editor_owner: String::new(),
            github_editor_repo: String::new(),
            github_oclive_owner: String::new(),
            github_oclive_repo: String::new(),
            oclive_roles_dir: String::new(),
            launcher_echo_role_id: String::new(),
            oclive_llm_mode: default_oclive_llm_mode(),
            oclive_remote_llm_url: String::new(),
            oclive_remote_llm_token: String::new(),
            oclive_remote_llm_timeout_ms: String::new(),
            oclive_remote_plugin_url: String::new(),
            oclive_remote_plugin_token: String::new(),
            oclive_remote_plugin_timeout_ms: String::new(),
            developer_announcements_url: String::new(),
        };
        ensure_github_upstream_defaults(&mut s);
        s
    }
}

struct AppState {
    editor: Arc<Mutex<Option<Child>>>,
    oclive: Arc<Mutex<Option<Child>>>,
}

fn app_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path_resolver()
        .app_config_dir()
        .ok_or_else(|| "无法解析应用配置目录".to_string())
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join("launcher-config.json"))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn mutex_lock<'a, T>(m: &'a Mutex<T>) -> Result<MutexGuard<'a, T>, String> {
    m.lock().map_err(|e| e.to_string())
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
    ensure_parent_dir(&path)?;
    let s = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, s).map_err(|e| e.to_string())
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
        .user_agent(concat!("oclive-launcher/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "GitHub 返回 {}（仓库可能为私有或尚无 Release）",
            resp.status()
        ));
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
    std::fs::read_dir(root).ok().is_some_and(|rd| {
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
    let (ed_ok, ed_pkg) = if config.editor_mode.trim() == "web" {
        (true, true)
    } else if config.editor_project_root.trim().is_empty() {
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
    let dir = app_config_dir(&app)?;
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
        .filter_map(|m| {
            m.get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
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
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let child = match cmd.spawn() {
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
        match drain_child_to_log(&app2, "ollama", child) {
            Ok(_) => emit_log(&app2, "ollama", "out", "--- ollama pull 已结束 ---"),
            Err(e) => emit_log(&app2, "ollama", "err", &format!("ollama pull：{}", e)),
        }
    });
    Ok(())
}

/// 是否可在本机调用 `winget`（主要用于 Windows 一键安装 Ollama）。
#[tauri::command]
fn winget_available() -> bool {
    #[cfg(windows)]
    {
        try_cmd_version("winget", &["--version"]).is_some()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Windows：通过 `winget install -e --id Ollama.Ollama` 安装官方 Ollama；日志发往 `winget` 频道。
/// 使用 `CREATE_NO_WINDOW` 隐藏 cmd 黑窗；UAC / 安装向导仍可由 winget 单独弹出。
#[tauri::command]
fn install_ollama_via_winget(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        return Err(
            "一键安装目前仅在 Windows 上通过 winget 提供。macOS 可用 Homebrew 或官网安装包；Linux 见 ollama.com。"
                .into(),
        );
    }
    #[cfg(windows)]
    {
        if try_cmd_version("winget", &["--version"]).is_none() {
            return Err(
                "未检测到 winget。请更新「应用安装程序」或从 https://ollama.com/download 手动安装。"
                    .into(),
            );
        }
        let app2 = app.clone();
        thread::spawn(move || {
            emit_log(
                &app2,
                "winget",
                "out",
                "--- 开始 winget install -e --id Ollama.Ollama（若弹出 UAC 或安装向导请按提示操作）---",
            );
            let mut cmd = Command::new("cmd");
            cmd.stdin(Stdio::null());
            cmd.args([
                "/C",
                "winget",
                "install",
                "-e",
                "--id",
                "Ollama.Ollama",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    emit_log(&app2, "winget", "err", &format!("启动 winget 失败：{}", e));
                    return;
                }
            };
            match drain_child_to_log(&app2, "winget", child) {
                Ok(s) if s.success() => {
                    emit_log(
                        &app2,
                        "winget",
                        "out",
                        "--- winget 安装命令已结束（成功）。若仍检测不到 ollama，请新开终端或重启后再点「重新检测」---",
                    );
                }
                Ok(s) => {
                    emit_log(
                        &app2,
                        "winget",
                        "err",
                        &format!(
                            "--- winget 退出码：{:?}。若已安装过 Ollama，可忽略；否则请见上方日志或改用官网安装包 ---",
                            s.code()
                        ),
                    );
                }
                Err(e) => emit_log(&app2, "winget", "err", &format!("winget：{}", e)),
            }
        });
        Ok(())
    }
}

/// 与 `tauri.conf.json` 中 `bundle.resources` 一致；发版前可将仓库根目录 `OllamaSetup.exe` 经 `scripts/sync-ollama-installer.mjs` 复制到本路径后打包。
const BUNDLED_OLLAMA_SETUP_REL: &str = "bundled/ollama/OllamaSetup.exe";

#[cfg(windows)]
fn resolve_bundled_ollama_installer(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path_resolver()
        .resolve_resource(BUNDLED_OLLAMA_SETUP_REL)
        .filter(|p| p.is_file())
}

/// 若打包时包含官方 Windows 安装包 `OllamaSetup.exe`（见资源路径），返回其绝对路径。
#[tauri::command]
fn bundled_ollama_installer_path(app: tauri::AppHandle) -> Option<String> {
    #[cfg(not(windows))]
    {
        let _ = app;
        None
    }
    #[cfg(windows)]
    {
        resolve_bundled_ollama_installer(&app).map(|p| p.to_string_lossy().into_owned())
    }
}

/// 启动附带的 Ollama 安装程序（不设 CREATE_NO_WINDOW，便于图形安装向导）。
#[tauri::command]
fn launch_bundled_ollama_installer(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        let _ = app;
        return Err("附带安装包目前仅提供 Windows 安装程序（OllamaSetup.exe）。".into());
    }
    #[cfg(windows)]
    {
        let p = resolve_bundled_ollama_installer(&app).ok_or_else(|| {
            "未找到附带安装包。请将官方 OllamaSetup.exe 放在启动器仓库根目录后执行构建（会自动同步到 bundled/ollama），或直接放入 src-tauri/bundled/ollama/。"
                .to_string()
        })?;
        let mut cmd = Command::new(&p);
        // 故意不使用 CREATE_NO_WINDOW，以便显示安装界面
        cmd.spawn()
            .map_err(|e| format!("无法启动安装程序：{}", e))?;
        emit_log(
            &app,
            "bundled-ollama",
            "out",
            &format!("--- 已启动附带安装程序：{} ---", p.display()),
        );
        Ok(())
    }
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

/// `kind` 为 `editor` | `oclive`（与 `spawn_managed_app` 约定一致）。
fn managed_exe_and_cwd(kind: &str, config: &LauncherConfig) -> Result<(PathBuf, PathBuf), String> {
    let (raw, empty_err, missing_err) = match kind {
        "editor" => (
            config.editor_exe.as_str(),
            "未设置编写器可执行文件路径",
            "编写器可执行文件不存在",
        ),
        "oclive" => (
            config.oclive_exe.as_str(),
            "未设置 oclive 可执行文件路径",
            "oclive 可执行文件不存在",
        ),
        _ => unreachable!(),
    };
    let t = raw.trim();
    if t.is_empty() {
        return Err(empty_err.into());
    }
    let p = PathBuf::from(t);
    if !p.is_file() {
        return Err(missing_err.into());
    }
    let cwd = p
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((p, cwd))
}

/// `kind` 为 `editor` | `oclive`。
fn resolve_editor_web_url(config: &LauncherConfig) -> String {
    let custom = config.editor_web_url.trim();
    if !custom.is_empty() {
        return custom.to_string();
    }
    let owner = config.github_editor_owner.trim();
    let repo = config.github_editor_repo.trim();
    let (owner, repo) = if owner.is_empty() || repo.is_empty() {
        (UPSTREAM_GITHUB_OWNER, UPSTREAM_EDITOR_REPO)
    } else {
        (owner, repo)
    };
    format!("https://{}.github.io/{}/", owner, repo)
}

fn managed_npm_root_and_script(
    kind: &str,
    config: &LauncherConfig,
) -> Result<(PathBuf, String), String> {
    match kind {
        "editor" => {
            if config.editor_project_root.trim().is_empty() {
                return Err("未设置编写器项目根目录".into());
            }
            validate_npm_script(&config.editor_npm_script)?;
            Ok((
                PathBuf::from(config.editor_project_root.trim()),
                config.editor_npm_script.clone(),
            ))
        }
        "oclive" => {
            if config.oclive_project_root.trim().is_empty() {
                return Err("未设置 oclive 项目根目录".into());
            }
            validate_npm_script(&config.oclive_npm_script)?;
            Ok((
                PathBuf::from(config.oclive_project_root.trim()),
                config.oclive_npm_script.clone(),
            ))
        }
        _ => unreachable!(),
    }
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

/// 将子进程 stdout/stderr 写入 launcher-log，并等待其结束（用于 `ollama pull` / `winget` 等）。
fn drain_child_to_log(
    app: &tauri::AppHandle,
    app_id: &str,
    mut child: Child,
) -> std::io::Result<std::process::ExitStatus> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("无法读取子进程 stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("无法读取子进程 stderr"))?;
    let app_o = app.clone();
    let aid = app_id.to_string();
    let h1 = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            emit_log(&app_o, &aid, "out", &line);
        }
    });
    let app_e = app.clone();
    let aid_e = app_id.to_string();
    let h2 = thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            emit_log(&app_e, &aid_e, "err", &line);
        }
    });
    let status = child.wait()?;
    let _ = h1.join();
    let _ = h2.join();
    Ok(status)
}

fn pipe_stream<R: std::io::Read + Send + 'static>(
    app: tauri::AppHandle,
    app_id: String,
    stream: String,
    reader: R,
) {
    thread::spawn(move || {
        let br = BufReader::new(reader);
        for line in br.lines().map_while(Result::ok) {
            emit_log(&app, &app_id, &stream, &line);
        }
    });
}

fn wait_child(slot: Arc<Mutex<Option<Child>>>, app: tauri::AppHandle, app_id: String) {
    thread::spawn(move || loop {
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
        let mut g = mutex_lock(&slot)?;
        if let Some(mut c) = g.take() {
            let _ = c.kill();
        }
    }

    if kind.as_str() == "editor" && config.editor_mode == "web" {
        let url = resolve_editor_web_url(&config);
        let t = url.trim();
        if t.is_empty() {
            return Err("编写器网页地址无效".into());
        }
        if !t.starts_with("http://") && !t.starts_with("https://") {
            return Err("编写器网页地址须以 http:// 或 https:// 开头".into());
        }
        open::that(t).map_err(|e| format!("打开浏览器失败：{}", e))?;
        emit_log(
            &app,
            "editor",
            "out",
            &format!("已在系统浏览器打开编写器：{}", t),
        );
        return Ok(());
    }

    let is_exe = match kind.as_str() {
        "editor" => config.editor_mode == "exe",
        "oclive" => config.oclive_mode == "exe",
        _ => false,
    };

    let mut child = if is_exe {
        let (exe, cwd) = managed_exe_and_cwd(kind.as_str(), &config)?;
        let mut cmd = Command::new(&exe);
        cmd.current_dir(&cwd);
        if kind == "oclive" {
            oclive_env::apply_oclive_process_env(&mut cmd, &config)?;
        }
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().map_err(|e| format!("启动失败：{}", e))?
    } else {
        let (root, npm_script) = managed_npm_root_and_script(kind.as_str(), &config)?;
        if !root.is_dir() {
            return Err("项目目录不存在或不是文件夹".into());
        }
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", "npm", "run", &npm_script]);
            c
        } else {
            let mut c = Command::new("npm");
            c.args(["run", &npm_script]);
            c
        };
        cmd.current_dir(&root);
        if kind == "oclive" {
            oclive_env::apply_oclive_process_env(&mut cmd, &config)?;
        }
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().map_err(|e| format!("启动失败：{}", e))?
    };

    let stdout = child.stdout.take().ok_or("无法读取子进程 stdout")?;
    let stderr = child.stderr.take().ok_or("无法读取子进程 stderr")?;

    {
        let mut g = mutex_lock(&slot)?;
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
        "--- 已启动：输出在启动器左侧「运行日志」里查看；本启动器已隐藏命令行窗口（Windows）。npm/子工具若再弹出窗口为其自身行为。---",
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
    let mut g = mutex_lock(slot)?;
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
            announcements::load_maintainer_announcements,
            announcements::save_maintainer_announcements,
            announcements::fetch_remote_announcement_text,
            announcements::load_creator_announcements,
            announcements::save_creator_announcements,
            role_creator_message::list_role_ids_with_manifest,
            role_creator_message::read_role_creator_message_lines,
            role_creator_message::write_role_creator_message,
            pick_folder,
            pick_exe,
            read_package_version,
            fetch_github_release,
            release_download::gh_latest_release_assets,
            release_download::gh_download_release_asset,
            open_url,
            diagnose_environment,
            reset_config_to_default,
            open_config_directory,
            suggest_roles_dir_from_oclive_root,
            pick_role_pack_zip,
            ollama_list_local_models,
            install_role_pack_zip,
            ollama_pull_model,
            winget_available,
            install_ollama_via_winget,
            bundled_ollama_installer_path,
            launch_bundled_ollama_installer,
            spawn_managed_app,
            stop_managed_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running oclive-launcher");
}
