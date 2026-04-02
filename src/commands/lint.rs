use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::schema::{Schema, load_schema, load_schemas};

pub fn run(ark_root: &Path, target: Option<&str>) -> Result<()> {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_files = 0;

    if let Some(target) = target {
        // Check if target is a type name or a specific ID
        if let Ok(schema) = load_schema(ark_root, target) {
            let (files, errors, warnings) = lint_type(ark_root, &schema)?;
            total_files += files;
            total_errors += errors;
            total_warnings += warnings;
        } else {
            // Try to find as a specific artifact ID
            let schemas = load_schemas(ark_root)?;
            let mut found = false;
            for schema in schemas.values() {
                if target.starts_with(&schema.prefix) {
                    let artifacts = load_artifacts(ark_root, schema)?;
                    if let Some(artifact) = artifacts.iter().find(|a| a.id() == Some(target)) {
                        let (errors, warnings) = lint_artifact(artifact, schema);
                        total_files = 1;
                        total_errors = errors;
                        total_warnings = warnings;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                anyhow::bail!("unknown type or artifact ID: {}", target);
            }
        }
    } else {
        // Lint all types
        let schemas = load_schemas(ark_root)?;
        for schema in schemas.values() {
            let (files, errors, warnings) = lint_type(ark_root, schema)?;
            total_files += files;
            total_errors += errors;
            total_warnings += warnings;
        }
    }

    println!();
    if total_errors == 0 && total_warnings == 0 {
        println!(
            "Lint passed. {} {} checked, no issues found.",
            total_files,
            pluralize("file", total_files)
        );
    } else {
        println!(
            "Lint complete. {} {} checked: {} {}, {} {}.",
            total_files,
            pluralize("file", total_files),
            total_errors,
            pluralize("error", total_errors),
            total_warnings,
            pluralize("warning", total_warnings),
        );
    }

    if total_errors > 0 {
        anyhow::bail!("lint failed with {} error(s)", total_errors);
    }

    Ok(())
}

fn lint_type(ark_root: &Path, schema: &Schema) -> Result<(usize, usize, usize)> {
    let artifacts = load_artifacts(ark_root, schema)?;
    let mut total_errors = 0;
    let mut total_warnings = 0;

    // Check for duplicate IDs
    let mut seen_ids: std::collections::HashMap<String, &std::path::Path> =
        std::collections::HashMap::new();
    for artifact in &artifacts {
        if let Some(id) = artifact.id() {
            if let Some(prev_path) = seen_ids.get(id) {
                report_error(
                    &artifact.path,
                    &format!("duplicate ID '{}' (also in {})", id, prev_path.display()),
                );
                total_errors += 1;
            } else {
                seen_ids.insert(id.to_string(), &artifact.path);
            }
        }
    }

    // Check for duplicate priorities
    let mut seen_priorities: std::collections::HashMap<i64, &std::path::Path> =
        std::collections::HashMap::new();
    for artifact in &artifacts {
        if let Some(priority) = artifact.priority() {
            if let Some(prev_path) = seen_priorities.get(&priority) {
                report_warning(
                    &artifact.path,
                    &format!(
                        "duplicate priority {} (also in {})",
                        priority,
                        prev_path.display()
                    ),
                );
                total_warnings += 1;
            } else {
                seen_priorities.insert(priority, &artifact.path);
            }
        }
    }

    // Validate each artifact against schema
    let json_schema = schema.to_json_schema();
    let validator = jsonschema::validator_for(&json_schema)
        .map_err(|e| anyhow::anyhow!("failed to compile schema for '{}': {}", schema.name, e))?;

    for artifact in &artifacts {
        let (errors, warnings) = lint_artifact_with_validator(artifact, schema, &validator);
        total_errors += errors;
        total_warnings += warnings;
    }

    Ok((artifacts.len(), total_errors, total_warnings))
}

fn lint_artifact(artifact: &crate::artifact::Artifact, schema: &Schema) -> (usize, usize) {
    let json_schema = schema.to_json_schema();
    match jsonschema::validator_for(&json_schema) {
        Ok(validator) => lint_artifact_with_validator(artifact, schema, &validator),
        Err(e) => {
            report_error(&artifact.path, &format!("failed to compile schema: {}", e));
            (1, 0)
        }
    }
}

fn lint_artifact_with_validator(
    artifact: &crate::artifact::Artifact,
    schema: &Schema,
    validator: &jsonschema::Validator,
) -> (usize, usize) {
    let mut errors = 0;
    let warnings = 0;

    let data = artifact.frontmatter_as_json();

    // Check ID format
    if let Some(id) = artifact.id() {
        if let Some(field) = schema.id_field()
            && let Some(ref pattern) = field.pattern
            && let Ok(re) = regex::Regex::new(pattern)
            && !re.is_match(id)
        {
            report_error(
                &artifact.path,
                &format!("ID '{}' does not match pattern '{}'", id, pattern),
            );
            errors += 1;
        }
    } else if schema.id_field().is_some_and(|f| f.required) {
        report_error(&artifact.path, "missing required field 'id'");
        errors += 1;
    }

    // JSON Schema validation for enum, required, type constraints
    for error in validator.iter_errors(&data) {
        report_error(
            &artifact.path,
            &format!("{} at {}", error, error.instance_path),
        );
        errors += 1;
    }

    (errors, warnings)
}

fn report_error(path: &Path, message: &str) {
    eprintln!("  error: {} - {}", path.display(), message);
}

fn report_warning(path: &Path, message: &str) {
    eprintln!("  warning: {} - {}", path.display(), message);
}

fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{}s", word)
    }
}
