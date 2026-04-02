use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::artifact::Artifact;
use crate::schema::{FieldType, Schema};

/// Validate a field value against a schema field definition.
/// Used by both `new` and `edit` commands for --set validation.
pub fn validate_field_value(schema: &Schema, field_name: &str, value: &str) -> Result<()> {
    if let Some(field) = schema.get_field(field_name) {
        // Don't allow setting derived fields
        if field.derived {
            bail!(
                "field '{}' is derived (auto-managed). It cannot be set manually.",
                field_name
            );
        }

        match field.field_type {
            FieldType::Enum => {
                if let Some(ref values) = field.values
                    && !values.contains(&value.to_string())
                {
                    bail!(
                        "invalid value '{}' for field '{}'. Valid values: {}",
                        value,
                        field_name,
                        values.join(", ")
                    );
                }
            }
            FieldType::Integer => {
                if value.parse::<i64>().is_err() {
                    bail!(
                        "field '{}' requires an integer value, got '{}'",
                        field_name,
                        value
                    );
                }
            }
            FieldType::Date => {
                if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
                    bail!(
                        "field '{}' requires a date in YYYY-MM-DD format, got '{}'",
                        field_name,
                        value
                    );
                }
            }
            FieldType::Boolean => {
                if !matches!(value, "true" | "false") {
                    bail!(
                        "field '{}' requires true or false, got '{}'",
                        field_name,
                        value
                    );
                }
            }
            FieldType::String => {
                // Validate pattern if present
                if let Some(ref pattern) = field.pattern
                    && let Ok(re) = regex::Regex::new(pattern)
                    && !re.is_match(value)
                {
                    bail!(
                        "value '{}' for field '{}' does not match pattern '{}'",
                        value,
                        field_name,
                        pattern
                    );
                }
            }
            FieldType::List => {
                // List values can't be set via --set as a single string
                // (would need special parsing). Warn the user.
                bail!(
                    "field '{}' is a list type. Use --tags for tag lists, or edit the file directly for other list fields.",
                    field_name
                );
            }
        }
    } else {
        // Field not in schema - warn on stderr but don't block
        eprintln!(
            "  warning: field '{}' is not defined in the schema. It will be stored but not validated.",
            field_name
        );
    }
    Ok(())
}

/// Convert a string value to the appropriate serde_json::Value based on schema field type.
/// This ensures booleans are stored as booleans, integers as integers, etc.
pub fn coerce_value(schema: &Schema, field_name: &str, value: &str) -> serde_json::Value {
    if let Some(field) = schema.get_field(field_name) {
        match field.field_type {
            FieldType::Integer => {
                if let Ok(n) = value.parse::<i64>() {
                    return serde_json::json!(n);
                }
            }
            FieldType::Boolean => return serde_json::Value::Bool(value == "true"),
            _ => {}
        }
    }
    serde_json::Value::String(value.to_string())
}

/// Check that all required non-derived fields are present in frontmatter
pub fn validate_required_fields(
    schema: &Schema,
    frontmatter: &HashMap<String, serde_json::Value>,
) -> Result<()> {
    let mut missing = Vec::new();
    for field in &schema.fields {
        if field.required && !field.derived && !frontmatter.contains_key(&field.name) {
            missing.push(field.name.as_str());
        }
    }
    if !missing.is_empty() {
        bail!(
            "missing required fields: {}. Use --set or named flags to provide them.",
            missing.join(", ")
        );
    }
    Ok(())
}

/// Check that a priority value is unique among existing artifacts
pub fn validate_unique_priority(
    existing: &[Artifact],
    priority: i64,
    exclude_id: Option<&str>,
) -> Result<()> {
    for artifact in existing {
        if artifact.priority() == Some(priority) {
            if let Some(exclude) = exclude_id
                && artifact.id() == Some(exclude)
            {
                continue;
            }
            bail!(
                "priority {} is already used by {}. Use `ark list` to see current priorities.",
                priority,
                artifact.id().unwrap_or("unknown")
            );
        }
    }
    Ok(())
}
