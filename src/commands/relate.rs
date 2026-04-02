use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::artifact::{Artifact, find_artifact_by_id};

/// Add related IDs to an artifact's frontmatter (deduplicated) and write it back
fn add_related(artifact: &Artifact, new_ids: &[&str]) -> Result<()> {
    let mut updated = artifact.clone();

    // Get existing related list
    let mut related = updated.get_list("related");

    // Add new IDs, deduplicating
    let mut changed = false;
    for id in new_ids {
        if !related.iter().any(|r| r == id) {
            related.push(id.to_string());
            changed = true;
        }
    }

    if !changed {
        return Ok(());
    }

    // Write the related field back as a JSON array
    let related_value =
        serde_json::Value::Array(related.into_iter().map(serde_json::Value::String).collect());
    updated.frontmatter.insert("related".into(), related_value);

    // Update the updated date
    let today = Local::now().format("%Y-%m-%d").to_string();
    updated
        .frontmatter
        .insert("updated".into(), serde_json::Value::String(today));

    let content = updated.to_markdown();
    std::fs::write(&artifact.path, &content)?;

    Ok(())
}

pub fn run(ark_root: &Path, id: &str, related_ids: &[String]) -> Result<()> {
    // Validate that the primary artifact exists
    let primary = find_artifact_by_id(ark_root, id)?;

    // Validate that all related artifacts exist
    let mut related_artifacts = Vec::new();
    for related_id in related_ids {
        if related_id == id {
            anyhow::bail!("cannot relate an artifact to itself: {}", id);
        }
        let artifact = find_artifact_by_id(ark_root, related_id)?;
        related_artifacts.push(artifact);
    }

    // Add related IDs to the primary artifact
    let related_id_strs: Vec<&str> = related_ids.iter().map(|s| s.as_str()).collect();
    add_related(&primary, &related_id_strs)?;

    // Bidirectional: add primary ID to each related artifact
    for related_artifact in &related_artifacts {
        add_related(related_artifact, &[id])?;
    }

    if related_ids.len() == 1 {
        println!("Related {} <-> {}", id, related_ids[0]);
    } else {
        println!("Related {} <-> [{}]", id, related_ids.join(", "));
    }

    Ok(())
}
