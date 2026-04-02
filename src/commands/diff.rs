use std::collections::HashMap;
use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::{Result, bail};

use crate::artifact::Artifact;
use crate::output::{OutputFormat, render_table};
use crate::schema::load_schemas;

/// ark diff <ref> [type] - show semantic changes to artifacts between a git ref and HEAD
pub fn run(
    ark_root: &Path,
    git_ref: &str,
    artifact_type: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    // Verify we're in a git repo
    let git_check = ProcessCommand::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(ark_root)
        .output();

    if !git_check.is_ok_and(|o| o.status.success()) {
        bail!("not a git repository. ark diff requires git.");
    }

    // Get list of files changed between ref and HEAD
    let output = ProcessCommand::new("git")
        .args(["diff", "--name-only", git_ref, "HEAD"])
        .current_dir(ark_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {}", stderr.trim());
    }

    let changed_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect();

    // Load current schemas
    let schemas = load_schemas(ark_root)?;

    // Identify which changed files are artifacts
    let mut changes: Vec<DiffEntry> = Vec::new();

    for schema in schemas.values() {
        if let Some(type_filter) = artifact_type
            && schema.name != type_filter
        {
            continue;
        }

        let dir_prefix = format!("{}/", schema.directory);
        let archive_prefix = schema.archive_directory().map(|d| format!("{}/", d));

        for file in &changed_files {
            let in_dir = file.starts_with(&dir_prefix);
            let in_archive = archive_prefix.as_ref().is_some_and(|p| file.starts_with(p));
            if !in_dir && !in_archive {
                continue;
            }
            if !file.ends_with(".md") && !file.ends_with(".feature") {
                continue;
            }

            let file_path = ark_root.join(file);

            // Get the old version from git ref
            let old_content = git_show_file(ark_root, git_ref, file);
            // Get current version from disk (or None if deleted)
            let new_content = std::fs::read_to_string(&file_path).ok();

            let change_type = match (&old_content, &new_content) {
                (None, Some(_)) => ChangeType::Added,
                (Some(_), None) => ChangeType::Removed,
                (Some(_), Some(_)) => ChangeType::Modified,
                (None, None) => continue,
            };

            let old_artifact = old_content
                .as_ref()
                .and_then(|c| Artifact::from_str(c, file_path.clone()).ok());
            let new_artifact = new_content
                .as_ref()
                .and_then(|c| Artifact::from_str(c, file_path.clone()).ok());

            let artifact = new_artifact.as_ref().or(old_artifact.as_ref());

            let id = artifact
                .and_then(|a| a.id().map(String::from))
                .unwrap_or_else(|| file.clone());
            let title = artifact
                .and_then(|a| a.title().map(String::from))
                .unwrap_or_default();

            // Detect field-level changes for modified artifacts
            let field_changes = if let (ChangeType::Modified, Some(old), Some(new)) =
                (&change_type, &old_artifact, &new_artifact)
            {
                diff_frontmatter(&old.frontmatter, &new.frontmatter)
            } else {
                Vec::new()
            };

            changes.push(DiffEntry {
                schema_name: schema.name.clone(),
                id,
                title,
                change_type,
                field_changes,
            });
        }
    }

    if changes.is_empty() {
        println!("No artifact changes between {} and HEAD.", git_ref);
        return Ok(());
    }

    // Sort: added first, then modified, then removed
    changes.sort_by_key(|c| match c.change_type {
        ChangeType::Added => 0,
        ChangeType::Modified => 1,
        ChangeType::Removed => 2,
    });

    let headers = &["change", "type", "id", "fields", "title"];
    let rows: Vec<Vec<String>> = changes
        .iter()
        .map(|c| {
            let change_str = match c.change_type {
                ChangeType::Added => "+",
                ChangeType::Modified => "~",
                ChangeType::Removed => "-",
            };
            let fields_str = if c.field_changes.is_empty() {
                String::new()
            } else {
                c.field_changes
                    .iter()
                    .map(|fc| format!("{}: {} -> {}", fc.field, fc.old_value, fc.new_value))
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            vec![
                change_str.to_string(),
                c.schema_name.clone(),
                c.id.clone(),
                fields_str,
                c.title.clone(),
            ]
        })
        .collect();

    println!("Artifact changes between {} and HEAD:\n", git_ref);
    println!("{}", render_table(headers, rows, format));

    Ok(())
}

#[derive(Debug)]
struct DiffEntry {
    schema_name: String,
    id: String,
    title: String,
    change_type: ChangeType,
    field_changes: Vec<FieldChange>,
}

#[derive(Debug)]
enum ChangeType {
    Added,
    Modified,
    Removed,
}

#[derive(Debug)]
struct FieldChange {
    field: String,
    old_value: String,
    new_value: String,
}

fn git_show_file(repo_root: &Path, git_ref: &str, file: &str) -> Option<String> {
    let spec = format!("{}:{}", git_ref, file);
    let output = ProcessCommand::new("git")
        .args(["show", &spec])
        .current_dir(repo_root)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn diff_frontmatter(
    old: &HashMap<String, serde_json::Value>,
    new: &HashMap<String, serde_json::Value>,
) -> Vec<FieldChange> {
    let mut changes = Vec::new();
    let skip_fields = ["updated"]; // don't report updated date as a change

    for (key, new_val) in new {
        if skip_fields.contains(&key.as_str()) {
            continue;
        }
        match old.get(key) {
            Some(old_val) if old_val != new_val => {
                changes.push(FieldChange {
                    field: key.clone(),
                    old_value: format_value(old_val),
                    new_value: format_value(new_val),
                });
            }
            None => {
                changes.push(FieldChange {
                    field: key.clone(),
                    old_value: "(none)".to_string(),
                    new_value: format_value(new_val),
                });
            }
            _ => {}
        }
    }

    // Check for removed fields
    for (key, old_val) in old {
        if skip_fields.contains(&key.as_str()) {
            continue;
        }
        if !new.contains_key(key) {
            changes.push(FieldChange {
                field: key.clone(),
                old_value: format_value(old_val),
                new_value: "(removed)".to_string(),
            });
        }
    }

    changes.sort_by(|a, b| a.field.cmp(&b.field));
    changes
}

fn format_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            format!("[{}]", items.join(", "))
        }
        other => other.to_string(),
    }
}
