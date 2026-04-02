use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::artifact::{Artifact, load_artifacts, next_id, slugify};
use crate::cli::NewArgs;
use crate::schema::load_schema;
use crate::validate::{validate_field_value, validate_required_fields, validate_unique_priority};

pub fn run(ark_root: &Path, args: &NewArgs) -> Result<()> {
    let schema = load_schema(ark_root, &args.artifact_type)?;

    // Validate title
    if args.title.trim().is_empty() {
        bail!("title cannot be empty");
    }
    if args.title.contains('\n') || args.title.contains('\r') {
        bail!("title cannot contain newlines");
    }

    // Acquire lock for atomic ID generation
    let lock_path = ark_root.join(".ark").join(".lock");
    let _lock = crate::lock::acquire_lock(&lock_path)?;

    // Load existing artifacts to determine next ID
    let existing = load_artifacts(ark_root, &schema)?;
    let next = next_id(&existing, &schema.prefix);
    let width = if next >= 1000 {
        (next as f64).log10().floor() as usize + 1
    } else {
        3
    };
    let id = format!("{}-{:0>width$}", schema.prefix, next);
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
        validate_field_value(&schema, "status", status)?;
        frontmatter.insert("status".into(), serde_json::Value::String(status.clone()));
    } else if let Some(field) = schema.get_field("status")
        && let Some(ref default) = field.default
    {
        frontmatter.insert("status".into(), default.clone());
    }

    // Set priority
    if let Some(priority) = args.priority {
        validate_unique_priority(&existing, priority, None)?;
        frontmatter.insert("priority".into(), serde_json::json!(priority));
    }

    // Set project
    if let Some(ref project) = args.project {
        validate_field_value(&schema, "project", project)?;
        frontmatter.insert("project".into(), serde_json::Value::String(project.clone()));
    }

    // Set type (via --kind flag)
    if let Some(ref item_type) = args.kind {
        validate_field_value(&schema, "type", item_type)?;
        frontmatter.insert("type".into(), serde_json::Value::String(item_type.clone()));
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

    // Set extra fields (validated and type-coerced against schema)
    // Detect conflicts with named flags
    if let Some(ref extras) = args.extra_fields {
        let named_fields: &[(&str, bool)] = &[
            ("status", args.status.is_some()),
            ("priority", args.priority.is_some()),
            ("project", args.project.is_some()),
            ("type", args.kind.is_some()),
        ];
        for (key, value) in extras {
            for (named_key, is_set) in named_fields {
                if key == named_key && *is_set {
                    anyhow::bail!(
                        "conflict: --{} and --set {}= both set the same field. Use one or the other.",
                        if *named_key == "type" {
                            "kind"
                        } else {
                            named_key
                        },
                        named_key
                    );
                }
            }
            validate_field_value(&schema, key, value)?;
            frontmatter.insert(
                key.clone(),
                crate::validate::coerce_value(&schema, key, value),
            );
        }
    }

    // Set derived dates
    frontmatter.insert("created".into(), serde_json::Value::String(today.clone()));
    frontmatter.insert("updated".into(), serde_json::Value::String(today));

    // Validate all required fields are present
    validate_required_fields(&schema, &frontmatter)?;

    // Build body from template
    let body = schema
        .template
        .as_ref()
        .map(|t| format!("\n{}", t))
        .unwrap_or_else(|| "\n".to_string());

    let artifact = Artifact {
        path: std::path::PathBuf::new(),
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
