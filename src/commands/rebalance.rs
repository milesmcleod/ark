use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::artifact::load_artifacts;
use crate::schema::load_schema;

pub fn run(ark_root: &Path, artifact_type: &str, gap: i64) -> Result<()> {
    let schema = load_schema(ark_root, artifact_type)?;

    if schema.priority_field().is_none() {
        println!("'{}' schema has no priority field. Nothing to rebalance.", artifact_type);
        return Ok(())
    }

    let mut artifacts = load_artifacts(ark_root, &schema)?;

    // Exclude archived
    if let Some(archive_value) = schema.archive_value() {
        artifacts.retain(|a| a.status() != Some(archive_value));
    }

    // Sort by current priority
    artifacts.sort_by(|a, b| {
        let pa = a.priority().unwrap_or(i64::MAX);
        let pb = b.priority().unwrap_or(i64::MAX);
        pa.cmp(&pb)
    });

    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut new_priority = gap;
    let mut count = 0;

    for mut artifact in artifacts {
        let old_priority = artifact.priority();
        if old_priority != Some(new_priority) {
            artifact
                .frontmatter
                .insert("priority".into(), serde_json::json!(new_priority));
            artifact
                .frontmatter
                .insert("updated".into(), serde_json::Value::String(today.clone()));
            let content = artifact.to_markdown();
            std::fs::write(&artifact.path, &content)?;
            count += 1;
        }
        new_priority += gap;
    }

    println!(
        "Rebalanced {} {} priorities with gap {}.",
        count, artifact_type, gap
    );

    Ok(())
}
