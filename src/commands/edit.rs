use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::artifact::load_artifacts;
use crate::cli::EditArgs;
use crate::error::ArkError;
use crate::schema::{load_schemas, FieldType};

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

            // Apply named field updates
            if let Some(ref status) = args.status {
                validate_enum(&schema, "status", status)?;
                updated
                    .frontmatter
                    .insert("status".into(), serde_json::Value::String(status.clone()));
            }
            if let Some(priority) = args.priority {
                updated
                    .frontmatter
                    .insert("priority".into(), serde_json::json!(priority));
            }
            if let Some(ref title) = args.title {
                updated
                    .frontmatter
                    .insert("title".into(), serde_json::Value::String(title.clone()));
            }
            if let Some(ref project) = args.project {
                validate_enum(&schema, "project", project)?;
                updated
                    .frontmatter
                    .insert("project".into(), serde_json::Value::String(project.clone()));
            }
            if let Some(ref item_type) = args.kind {
                validate_enum(&schema, "type", item_type)?;
                updated
                    .frontmatter
                    .insert("type".into(), serde_json::Value::String(item_type.clone()));
            }

            // Apply --set key=value fields
            for (key, value) in &args.fields {
                updated
                    .frontmatter
                    .insert(key.clone(), serde_json::Value::String(value.clone()));
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

fn validate_enum(
    schema: &crate::schema::Schema,
    field_name: &str,
    value: &str,
) -> Result<()> {
    if let Some(field) = schema.get_field(field_name) {
        if field.field_type == FieldType::Enum {
            if let Some(ref values) = field.values {
                if !values.contains(&value.to_string()) {
                    anyhow::bail!(
                        "invalid value '{}' for field '{}'. Valid values: {}",
                        value,
                        field_name,
                        values.join(", ")
                    );
                }
            }
        }
    }
    Ok(())
}
