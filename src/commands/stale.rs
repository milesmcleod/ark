use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::output::{OutputFormat, render_table};
use crate::schema::load_schema;

/// ark stale <type> --days N - find artifacts with status active/backlog
/// but no git activity in their project within the last N days
pub fn run(ark_root: &Path, artifact_type: &str, days: u32, format: &OutputFormat) -> Result<()> {
    let schema = load_schema(ark_root, artifact_type)?;
    let mut artifacts = load_artifacts(ark_root, &schema)?;

    // Only check non-archived artifacts
    if let Some(archive_value) = schema.archive_value() {
        artifacts.retain(|a| a.status() != Some(archive_value));
    }

    let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

    let mut stale_items = Vec::new();

    for artifact in &artifacts {
        // Check the artifact's updated date in frontmatter
        let updated = artifact.get_str("updated").unwrap_or("");

        if updated.is_empty() || updated <= cutoff_str.as_str() {
            // Also check git log for the specific file
            let last_commit_date = get_last_commit_date(&artifact.path);
            let is_stale = match &last_commit_date {
                Some(date) => date.as_str() <= cutoff_str.as_str(),
                None => true, // not tracked by git = stale
            };

            if is_stale {
                stale_items.push((
                    artifact,
                    updated.to_string(),
                    last_commit_date.unwrap_or_else(|| "untracked".to_string()),
                ));
            }
        }
    }

    if stale_items.is_empty() {
        println!(
            "No stale {} artifacts found (threshold: {} days).",
            artifact_type, days
        );
        return Ok(());
    }

    let headers = &["id", "status", "updated", "last_commit", "title"];
    let rows: Vec<Vec<String>> = stale_items
        .iter()
        .map(|(a, updated, last_commit)| {
            vec![
                a.id().unwrap_or("-").to_string(),
                a.status().unwrap_or("-").to_string(),
                updated.clone(),
                last_commit.clone(),
                a.title().unwrap_or("-").to_string(),
            ]
        })
        .collect();

    println!(
        "Stale {} artifacts (no activity in {} days):\n",
        artifact_type, days
    );
    println!("{}", render_table(headers, rows, format));

    Ok(())
}

/// Get the date of the last git commit that touched a file
fn get_last_commit_date(path: &Path) -> Option<String> {
    let output = ProcessCommand::new("git")
        .args(["log", "-1", "--format=%cd", "--date=short", "--"])
        .arg(path)
        .output()
        .ok()?;

    if output.status.success() {
        let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if date.is_empty() { None } else { Some(date) }
    } else {
        None
    }
}
