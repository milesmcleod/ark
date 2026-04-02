use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result};

#[derive(Debug, serde::Deserialize)]
pub struct HooksConfig {
    #[serde(default)]
    pub on_status_change: Vec<HookDef>,
    #[serde(default)]
    pub on_create: Vec<HookDef>,
    #[serde(default)]
    pub on_archive: Vec<HookDef>,
}

#[derive(Debug, serde::Deserialize)]
pub struct HookDef {
    #[serde(rename = "type")]
    pub artifact_type: Option<String>,
    pub to_status: Option<String>,
    pub from_status: Option<String>,
    pub run: String,
}

impl HooksConfig {
    pub fn load(ark_root: &Path) -> Option<Self> {
        let hooks_path = ark_root.join(".ark").join("hooks.yml");
        let content = std::fs::read_to_string(&hooks_path).ok()?;
        serde_yml::from_str(&content).ok()
    }
}

/// Check if a hook's optional filter matches a value
fn matches_filter(filter: &Option<String>, value: &str) -> bool {
    filter.as_deref().is_none_or(|f| f == value)
}

pub fn run_status_change_hooks(
    ark_root: &Path,
    artifact_type: &str,
    artifact_id: &str,
    from_status: &str,
    to_status: &str,
) {
    let Some(config) = HooksConfig::load(ark_root) else {
        return;
    };

    for hook in &config.on_status_change {
        if !matches_filter(&hook.artifact_type, artifact_type)
            || !matches_filter(&hook.to_status, to_status)
            || !matches_filter(&hook.from_status, from_status)
        {
            continue;
        }

        execute_hook(
            &hook.run,
            ark_root,
            artifact_id,
            artifact_type,
            Some(from_status),
            Some(to_status),
        );
    }
}

pub fn run_create_hooks(ark_root: &Path, artifact_type: &str, artifact_id: &str) {
    let Some(config) = HooksConfig::load(ark_root) else {
        return;
    };

    for hook in &config.on_create {
        if !matches_filter(&hook.artifact_type, artifact_type) {
            continue;
        }
        execute_hook(&hook.run, ark_root, artifact_id, artifact_type, None, None);
    }
}

pub fn run_archive_hooks(ark_root: &Path, artifact_type: &str, artifact_id: &str) {
    let Some(config) = HooksConfig::load(ark_root) else {
        return;
    };

    for hook in &config.on_archive {
        if !matches_filter(&hook.artifact_type, artifact_type) {
            continue;
        }
        execute_hook(&hook.run, ark_root, artifact_id, artifact_type, None, None);
    }
}

fn execute_hook(
    command: &str,
    ark_root: &Path,
    artifact_id: &str,
    artifact_type: &str,
    from_status: Option<&str>,
    to_status: Option<&str>,
) {
    let result = ProcessCommand::new("sh")
        .args(["-c", command])
        .current_dir(ark_root)
        .env("ARK_ARTIFACT_ID", artifact_id)
        .env("ARK_ARTIFACT_TYPE", artifact_type)
        .env("ARK_FROM_STATUS", from_status.unwrap_or(""))
        .env("ARK_TO_STATUS", to_status.unwrap_or(""))
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "  warning: hook '{}' failed (exit {}): {}",
                    command,
                    output.status.code().unwrap_or(-1),
                    stderr.trim()
                );
            }
        }
        Err(e) => {
            eprintln!("  warning: hook '{}' failed to execute: {}", command, e);
        }
    }
}

/// ark hooks - list configured hooks
pub fn run_list(ark_root: &Path) -> Result<()> {
    let hooks_path = ark_root.join(".ark").join("hooks.yml");

    if !hooks_path.exists() {
        println!("No hooks configured. Create .ark/hooks.yml to define lifecycle hooks.");
        println!();
        println!("Example .ark/hooks.yml:");
        println!("  on_status_change:");
        println!("    - to_status: done");
        println!("      run: echo \"$ARK_ARTIFACT_ID completed\"");
        println!("  on_create:");
        println!("    - type: task");
        println!("      run: echo \"New task $ARK_ARTIFACT_ID created\"");
        println!("  on_archive:");
        println!("    - run: echo \"$ARK_ARTIFACT_ID archived\"");
        return Ok(());
    }

    let content = std::fs::read_to_string(&hooks_path)
        .with_context(|| format!("failed to read {}", hooks_path.display()))?;
    let config: HooksConfig = serde_yml::from_str(&content)
        .with_context(|| format!("failed to parse {}", hooks_path.display()))?;

    let mut total = 0;

    if !config.on_status_change.is_empty() {
        println!("on_status_change:");
        for hook in &config.on_status_change {
            let mut filters = Vec::new();
            if let Some(ref t) = hook.artifact_type {
                filters.push(format!("type={}", t));
            }
            if let Some(ref ts) = hook.to_status {
                filters.push(format!("to={}", ts));
            }
            if let Some(ref fs) = hook.from_status {
                filters.push(format!("from={}", fs));
            }
            let filter_str = if filters.is_empty() {
                "(all)".to_string()
            } else {
                filters.join(", ")
            };
            println!("  [{}] {}", filter_str, hook.run);
            total += 1;
        }
    }

    if !config.on_create.is_empty() {
        println!("on_create:");
        for hook in &config.on_create {
            let filter = hook
                .artifact_type
                .as_ref()
                .map(|t| format!("type={}", t))
                .unwrap_or_else(|| "(all)".to_string());
            println!("  [{}] {}", filter, hook.run);
            total += 1;
        }
    }

    if !config.on_archive.is_empty() {
        println!("on_archive:");
        for hook in &config.on_archive {
            let filter = hook
                .artifact_type
                .as_ref()
                .map(|t| format!("type={}", t))
                .unwrap_or_else(|| "(all)".to_string());
            println!("  [{}] {}", filter, hook.run);
            total += 1;
        }
    }

    if total == 0 {
        println!("No hooks defined in .ark/hooks.yml.");
    } else {
        println!("\n{} hook(s) configured.", total);
    }

    Ok(())
}
