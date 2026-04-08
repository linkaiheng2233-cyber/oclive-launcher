//! 仅为 **oclive 子进程** 配置环境变量：`OCLIVE_ROLES_DIR`、推理后端与 Remote LLM。
//! 编写器子进程不经过此模块。

use std::path::PathBuf;
use std::process::Command;

use super::LauncherConfig;

const LLM_OLLAMA: &str = "ollama";
const LLM_REMOTE: &str = "remote";

/// 配置 oclive 所需的 roles 目录与 LLM 相关变量（启动 oclive 时唯一入口，避免在 `spawn` 处分叉重复）。
pub(crate) fn apply_oclive_process_env(cmd: &mut Command, config: &LauncherConfig) -> Result<(), String> {
    apply_roles_dir(cmd, config)?;
    apply_llm_env(cmd, config)?;
    Ok(())
}

fn apply_roles_dir(cmd: &mut Command, config: &LauncherConfig) -> Result<(), String> {
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

fn clear_remote_llm_inheritance(cmd: &mut Command) {
    cmd.env_remove("OCLIVE_REMOTE_LLM_URL");
    cmd.env_remove("OCLIVE_REMOTE_LLM_TOKEN");
    cmd.env_remove("OCLIVE_REMOTE_LLM_TIMEOUT_MS");
}

fn apply_llm_env(cmd: &mut Command, config: &LauncherConfig) -> Result<(), String> {
    let mode = config.oclive_llm_mode.trim().to_lowercase();
    let mode = if mode.is_empty() {
        LLM_OLLAMA.to_string()
    } else {
        mode
    };

    if mode == LLM_OLLAMA {
        cmd.env("OCLIVE_LLM_BACKEND", LLM_OLLAMA);
        clear_remote_llm_inheritance(cmd);
        return Ok(());
    }

    if mode == LLM_REMOTE {
        let url = config.oclive_remote_llm_url.trim();
        if url.is_empty() {
            return Err(
                "云端模式需填写「Remote LLM URL」（JSON-RPC 端点，见 oclivenewnew REMOTE_PLUGIN_PROTOCOL）"
                    .into(),
            );
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("Remote LLM URL 须以 http:// 或 https:// 开头".into());
        }
        cmd.env("OCLIVE_LLM_BACKEND", LLM_REMOTE);
        cmd.env("OCLIVE_REMOTE_LLM_URL", url);
        let tok = config.oclive_remote_llm_token.trim();
        if !tok.is_empty() {
            cmd.env("OCLIVE_REMOTE_LLM_TOKEN", tok);
        } else {
            cmd.env_remove("OCLIVE_REMOTE_LLM_TOKEN");
        }
        let to = config.oclive_remote_llm_timeout_ms.trim();
        if !to.is_empty() {
            to.parse::<u64>()
                .map_err(|_| "Remote LLM 超时须为毫秒正整数".to_string())?;
            cmd.env("OCLIVE_REMOTE_LLM_TIMEOUT_MS", to);
        } else {
            cmd.env_remove("OCLIVE_REMOTE_LLM_TIMEOUT_MS");
        }
        return Ok(());
    }

    Err(format!(
        "未知的推理模式「{}」（请使用 {} 或 {}）",
        mode, LLM_OLLAMA, LLM_REMOTE
    ))
}
