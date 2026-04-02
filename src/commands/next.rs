use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::output::{render_table, OutputFormat};
use crate::schema::load_schema;

pub fn run(ark_root: &Path, artifact_type: &str, count: usize, format: &OutputFormat) -> Result<()> {
    let schema = load_schema(ark_root, artifact_type)?;
    let mut artifacts = load_artifacts(ark_root, &schema)?;

    // Show active items first, then highest priority backlog items
    let archive_value = schema.archive_value().unwrap_or("done");

    // Split into active and backlog
    let active: Vec<_> = artifacts
        .iter()
        .filter(|a| a.status() == Some("active"))
        .cloned()
        .collect();

    artifacts.retain(|a| {
        a.status() != Some(archive_value)
            && a.status() != Some("active")
            && a.status() != Some("blocked")
    });

    // Sort backlog by priority
    artifacts.sort_by(|a, b| {
        let pa = a.priority().unwrap_or(i64::MAX);
        let pb = b.priority().unwrap_or(i64::MAX);
        pa.cmp(&pb)
    });
    artifacts.truncate(count);

    if active.is_empty() && artifacts.is_empty() {
        println!(
            "No active or queued {} artifacts. Create one with `ark new {}`.",
            artifact_type, artifact_type
        );
        return Ok(())
    }

    if !active.is_empty() {
        println!("Active:");
        let headers = &["id", "pri", "title"];
        let rows: Vec<Vec<String>> = active
            .iter()
            .map(|a| {
                vec![
                    a.id().unwrap_or("-").into(),
                    a.priority().map(|p| p.to_string()).unwrap_or("-".into()),
                    a.title().unwrap_or("-").into(),
                ]
            })
            .collect();
        println!("{}", render_table(headers, rows, format));
    }

    if !artifacts.is_empty() {
        if !active.is_empty() {
            println!();
        }
        println!("Up next:");
        let headers = &["id", "pri", "status", "title"];
        let rows: Vec<Vec<String>> = artifacts
            .iter()
            .map(|a| {
                vec![
                    a.id().unwrap_or("-").into(),
                    a.priority().map(|p| p.to_string()).unwrap_or("-".into()),
                    a.status().unwrap_or("-").into(),
                    a.title().unwrap_or("-").into(),
                ]
            })
            .collect();
        println!("{}", render_table(headers, rows, format));
    }

    Ok(())
}
