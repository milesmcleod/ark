use std::path::Path;

use anyhow::{bail, Result};
use chrono::Local;

use crate::artifact::{load_artifacts, next_id, slugify, Artifact};
use crate::cli::NewArgs;
use crate::schema::{load_schema, FieldType};

pub fn run(ark_root: &Path, args: &NewArgs) -> Result<()> {
    let schema = load_schema(ark_root, &args.artifact_type)?;

    // Validate title is not empty
    if args.title.trim().is_empty() {
        bail!("title cannot be empty");
    }

    // Load existing artifacts to determine next ID
    let existing = load_artifacts(ark_root, &schema)?;
    let next = next_id(&existing, &schema.prefix);
    let id = format!("{}-{:03}", schema.prefix, next);
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Build frontmatter
    let mut frontmatter = std::collections::HashMap::new();
    frontmatter.insert("id".into(), serde_json::Value::String(id.clone()));
    frontmatter.insert(
        "title".into(),
        serde_json::Value::String(args.title.clone()),
    );

    // Set status from args or schema default
    if let Some(ref status) = args.status {
        validate_enum_field(&schema, "status", status)?;
        frontmatter.insert("status".into(), serde_json::Value::String(status.clone()));
    } else if let Some(field) = schema.get_field("status") {
        if let Some(ref default) = field.default {
            frontmatter.insert("status".into(), default.clone());
        }
    }

    // Set priority
    if let Some(priority) = args.priority {
        frontmatter.insert("priority".into(), serde_json::json!(priority));
    }

    // Set project
    if let Some(ref project) = args.project {
        validate_enum_field(&schema, "project", project)?;
        frontmatter.insert("project".into(), serde_json::Value::String(project.clone()));
    }

    // Set type
    if let Some(ref item_type) = args.kind {
        validate_enum_field(&schema, "type", item_type)?;
        frontmatter.insert(
            "type".into(),
            serde_json::Value::String(item_type.clone()),
        );
    }

    // Set tags
    if let Some(ref tags) = args.tags {
        frontmatter.insert(
            "tags".into(),
            serde_json::Value::Array(
                tags.iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            ),
        );
    }

    // Set extra fields
    if let Some(ref extras) = args.extra_fields {
        for (key, value) in extras {
            frontmatter.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
    }

    // Set derived dates
    frontmatter.insert("created".into(), serde_json::Value::String(today.clone()));
    frontmatter.insert("updated".into(), serde_json::Value::String(today));

    // Build body from template
    let body = schema
        .template
        .as_ref()
        .map(|t| format!("\n{}", t))
        .unwrap_or_else(|| "\n".to_string());

    let artifact = Artifact {
        path: std::path::PathBuf::new(), // will be set below
        frontmatter,
        body,
        raw: String::new(),
    };

    // Ensure directory exists
    let dir = ark_root.join(&schema.directory);
    std::fs::create_dir_all(&dir)?;

    // Write file
    let slug = slugify(&args.title);
    let filename = format!("{}-{}.md", id, slug);
    let filepath = dir.join(&filename);
    let content = artifact.to_markdown();
    std::fs::write(&filepath, &content)?;

    println!("Created {} at {}", id, filepath.display());

    Ok(())
}

fn validate_enum_field(schema: &crate::schema::Schema, field_name: &str, value: &str) -> Result<()> {
    if let Some(field) = schema.get_field(field_name) {
        if field.field_type == FieldType::Enum {
            if let Some(ref values) = field.values {
                if !values.contains(&value.to_string()) {
                    bail!(
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
