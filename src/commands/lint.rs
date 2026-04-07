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
    let mut artifacts = load_artifacts(ark_root, schema)?;
    let mut total_errors = 0;
    let mut total_warnings = 0;

    // Auto-fix supersession status drift: when an artifact declares
    // `supersedes: <ID>`, the target artifact's status must be
    // `superseded`. Only runs if the schema defines both a
    // `supersedes` field and a `status` field with `superseded` as
    // a valid value. Re-loads `artifacts` after writes so downstream
    // checks see the corrected state.
    let fixes = enforce_supersession_status(&mut artifacts, schema)?;
    if fixes > 0 {
        // Re-read from disk to pick up the new statuses for any
        // subsequent checks below.
        artifacts = load_artifacts(ark_root, schema)?;
    }

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

/// Auto-fix supersession status drift.
///
/// When an artifact declares `supersedes: <ID>` in its frontmatter,
/// the target artifact's `status` field must be `superseded`. Lint
/// scans the artifact set, finds every supersession pointer, and
/// rewrites any target whose status is anything else.
///
/// Returns the number of fixes applied. Errors propagate via
/// `Result` for I/O failures only - the schema check itself is
/// silent (no warning when the supersession declaration is fine).
///
/// The supersedes value is parsed liberally: free-form annotations
/// like `"ADR-001 (original, ESP32-S3 solo)"` are accepted - only
/// the leading `<PREFIX>-<NUMBER>` token is used to look up the
/// target. If the value doesn't start with a recognizable artifact
/// ID, the entry is silently skipped.
///
/// Status `deprecated` is not overwritten - a deliberately
/// deprecated artifact stays deprecated even if a newer one points
/// at it via supersedes (this is a corner case but worth respecting).
fn enforce_supersession_status(
    artifacts: &mut [crate::artifact::Artifact],
    schema: &Schema,
) -> Result<usize> {
    // Schema must have a `supersedes` field for this check to mean
    // anything, AND a `status` field whose enum includes `superseded`.
    let has_supersedes = schema.fields.iter().any(|f| f.name == "supersedes");
    if !has_supersedes {
        return Ok(0);
    }
    let status_accepts_superseded = schema
        .fields
        .iter()
        .find(|f| f.name == "status")
        .and_then(|f| f.values.as_ref())
        .map(|values| values.iter().any(|v| v == "superseded"))
        .unwrap_or(false);
    if !status_accepts_superseded {
        return Ok(0);
    }

    // Build an index from raw ID to (vector index) so we can mutate
    // entries in place after the scan.
    let mut by_id: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, a) in artifacts.iter().enumerate() {
        if let Some(id) = a.id() {
            by_id.insert(id.to_string(), i);
        }
    }

    // Collect (target_index, source_id) pairs to apply after the
    // scan. Two-phase to avoid mutable borrow during iteration.
    let mut to_fix: Vec<(usize, String)> = Vec::new();
    for a in artifacts.iter() {
        let supersedes_raw = match a.get_str("supersedes") {
            Some(s) => s,
            None => continue,
        };
        // Extract the leading ID token: split on whitespace or
        // opening paren.
        let target_id = supersedes_raw
            .split(|c: char| c.is_whitespace() || c == '(')
            .next()
            .unwrap_or("")
            .trim();
        if target_id.is_empty() {
            continue;
        }
        let target_idx = match by_id.get(target_id) {
            Some(&i) => i,
            None => continue, // Cross-project or unknown reference - skip
        };
        let target_status = artifacts[target_idx].status().unwrap_or("");
        if target_status == "superseded" || target_status == "deprecated" {
            continue;
        }
        let source_id = a.id().unwrap_or("?").to_string();
        to_fix.push((target_idx, source_id));
    }

    if to_fix.is_empty() {
        return Ok(0);
    }

    // Apply fixes.
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut applied = 0;
    for (target_idx, source_id) in to_fix {
        let target = &mut artifacts[target_idx];
        let target_id = target.id().unwrap_or("?").to_string();
        target.frontmatter.insert(
            "status".into(),
            serde_json::Value::String("superseded".into()),
        );
        target
            .frontmatter
            .insert("updated".into(), serde_json::Value::String(today.clone()));
        let content = target.to_markdown();
        std::fs::write(&target.path, &content).map_err(|e| {
            anyhow::anyhow!(
                "failed to write supersession-status fix to {}: {}",
                target.path.display(),
                e
            )
        })?;
        eprintln!(
            "  auto-fix: {} status -> superseded (declared by {} via supersedes:)",
            target_id, source_id
        );
        applied += 1;
    }

    Ok(applied)
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
