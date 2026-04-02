use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::artifact::load_artifacts;
use crate::cli::EditArgs;
use crate::error::ArkError;
use crate::schema::load_schemas;
use crate::validate::validate_field_value;

pub fn run(ark_root: &Path, args: &EditArgs) -> Result<()> {
    let schemas = load_schemas(ark_root)?;
    let id = &args.id;

    // Find the artifact across all types
    for schema in schemas.values() {
        if !id.starts_with(&schema.prefix) {
            continue;
        }

        let artifacts = load_artifacts(ark_root, schema)?;
        let artifact = artifacts.iter().find(|a| a.id() == Some(id.as_str()));

        if let Some(artifact) = artifact {
            let mut updated = artifact.clone();
            let mut changed = false;

            // Apply named field updates
            if let Some(ref status) = args.status {
                validate_field_value(schema, "status", status)?;
                updated
                    .frontmatter
                    .insert("status".into(), serde_json::Value::String(status.clone()));
                changed = true;
            }
            if let Some(priority) = args.priority {
                // Validate unique priority (excluding self)
                crate::validate::validate_unique_priority(&artifacts, priority, Some(id))?;
                updated
                    .frontmatter
                    .insert("priority".into(), serde_json::json!(priority));
                changed = true;
            }
            if let Some(ref title) = args.title {
                if title.trim().is_empty() {
                    anyhow::bail!("title cannot be empty");
                }
                updated
                    .frontmatter
                    .insert("title".into(), serde_json::Value::String(title.clone()));
                changed = true;
            }
            if let Some(ref project) = args.project {
                validate_field_value(schema, "project", project)?;
                updated
                    .frontmatter
                    .insert("project".into(), serde_json::Value::String(project.clone()));
                changed = true;
            }
            if let Some(ref item_type) = args.kind {
                validate_field_value(schema, "type", item_type)?;
                updated
                    .frontmatter
                    .insert("type".into(), serde_json::Value::String(item_type.clone()));
                changed = true;
            }

            // Apply --set key=value fields (validated and type-coerced against schema)
            // Detect conflicts with named flags
            let named_fields: &[(&str, bool)] = &[
                ("status", args.status.is_some()),
                ("priority", args.priority.is_some()),
                ("title", args.title.is_some()),
                ("project", args.project.is_some()),
                ("type", args.kind.is_some()),
            ];
            for (key, value) in &args.fields {
                for (named_key, is_set) in named_fields {
                    if key == named_key && *is_set {
                        anyhow::bail!(
                            "conflict: --{} and --set {}= both set the same field. Use one or the other.",
                            named_key,
                            named_key
                        );
                    }
                }
                validate_field_value(schema, key, value)?;
                updated.frontmatter.insert(
                    key.clone(),
                    crate::validate::coerce_value(schema, key, value),
                );
                changed = true;
            }

            if !changed {
                println!("No changes to {}.", id);
                return Ok(());
            }

            // Update the updated date
            let today = Local::now().format("%Y-%m-%d").to_string();
            updated
                .frontmatter
                .insert("updated".into(), serde_json::Value::String(today));

            // Write back
            let content = updated.to_markdown();
            std::fs::write(&artifact.path, &content)?;

            println!("Updated {}", id);
            return Ok(());
        }
    }

    Err(ArkError::ArtifactNotFound(id.to_string()).into())
}
