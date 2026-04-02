use std::path::Path;

use anyhow::Result;

use crate::artifact::{Artifact, find_artifact_by_id};
use crate::output::OutputFormat;

/// Format a frontmatter summary for a related artifact (pretty/tsv mode)
fn frontmatter_summary(artifact: &Artifact) -> String {
    let mut parts = Vec::new();
    if let Some(id) = artifact.id() {
        parts.push(format!("id: {}", id));
    }
    if let Some(title) = artifact.title() {
        parts.push(format!("title: {}", title));
    }
    if let Some(status) = artifact.status() {
        parts.push(format!("status: {}", status));
    }
    if let Some(priority) = artifact.priority() {
        parts.push(format!("priority: {}", priority));
    }
    // Include any other notable fields
    let skip = ["id", "title", "status", "priority", "body"];
    for (key, value) in &artifact.frontmatter {
        if !skip.contains(&key.as_str()) {
            if let Some(s) = value.as_str() {
                parts.push(format!("{}: {}", key, s));
            } else if let Some(arr) = value.as_array() {
                let items: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                if !items.is_empty() {
                    parts.push(format!("{}: [{}]", key, items.join(", ")));
                }
            } else {
                parts.push(format!("{}: {}", key, value));
            }
        }
    }
    parts.join("\n")
}

pub fn run(ark_root: &Path, id: &str, format: &OutputFormat) -> Result<()> {
    let primary = find_artifact_by_id(ark_root, id)?;

    // Get related IDs
    let related_ids = primary.get_list("related");

    // Resolve related artifacts (skip any that can't be found, warn on stderr)
    let mut related_artifacts = Vec::new();
    for related_id in &related_ids {
        match find_artifact_by_id(ark_root, related_id) {
            Ok(artifact) => related_artifacts.push(artifact),
            Err(_) => {
                eprintln!(
                    "warning: related artifact {} not found, skipping",
                    related_id
                );
            }
        }
    }

    match format {
        OutputFormat::Json => {
            let mut primary_map = serde_json::Map::new();
            for (k, v) in &primary.frontmatter {
                primary_map.insert(k.clone(), v.clone());
            }
            primary_map.insert(
                "body".into(),
                serde_json::Value::String(primary.body.clone()),
            );

            let related_json: Vec<serde_json::Value> = related_artifacts
                .iter()
                .map(|a| {
                    let mut map = serde_json::Map::new();
                    for (k, v) in &a.frontmatter {
                        map.insert(k.clone(), v.clone());
                    }
                    serde_json::Value::Object(map)
                })
                .collect();

            let output = serde_json::json!({
                "primary": primary_map,
                "related": related_json,
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Tsv => {
            // Primary: full content
            print!("{}", primary.raw);

            if !related_artifacts.is_empty() {
                println!("\n---related---");
                for artifact in &related_artifacts {
                    println!("{}", frontmatter_summary(artifact));
                    println!("---");
                }
            }
        }
        OutputFormat::Pretty => {
            // Primary: full content
            print!("{}", primary.raw);

            if !related_artifacts.is_empty() {
                println!();
                println!("Related:");
                for artifact in &related_artifacts {
                    println!();
                    println!("{}", frontmatter_summary(artifact));
                }
            }
        }
    }

    Ok(())
}
