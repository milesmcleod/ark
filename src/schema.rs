use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::error::ArkError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub directory: String,
    pub prefix: String,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
    #[serde(default)]
    pub archive: Option<ArchiveDef>,
    #[serde(default)]
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub derived: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub values: Option<Vec<String>>,
    #[serde(default)]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Integer,
    Date,
    Enum,
    List,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveDef {
    pub field: String,
    pub value: String,
    pub directory: String,
}

impl Schema {
    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn id_field(&self) -> Option<&FieldDef> {
        self.get_field("id")
    }

    pub fn priority_field(&self) -> Option<&FieldDef> {
        self.get_field("priority")
    }

    pub fn archive_value(&self) -> Option<&str> {
        self.archive.as_ref().map(|a| a.value.as_str())
    }

    pub fn archive_directory(&self) -> Option<&str> {
        self.archive.as_ref().map(|a| a.directory.as_str())
    }

    /// Generate a JSON Schema for validating artifact frontmatter
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for field in &self.fields {
            let field_schema = match field.field_type {
                FieldType::String => {
                    let mut s = serde_json::Map::new();
                    s.insert("type".into(), "string".into());
                    if let Some(ref pattern) = field.pattern {
                        s.insert("pattern".into(), pattern.clone().into());
                    }
                    serde_json::Value::Object(s)
                }
                FieldType::Integer => {
                    serde_json::json!({"type": "integer"})
                }
                FieldType::Date => {
                    serde_json::json!({"type": "string", "pattern": "^\\d{4}-\\d{2}-\\d{2}$"})
                }
                FieldType::Enum => {
                    let mut s = serde_json::Map::new();
                    s.insert("type".into(), "string".into());
                    if let Some(ref values) = field.values {
                        s.insert(
                            "enum".into(),
                            serde_json::Value::Array(
                                values
                                    .iter()
                                    .map(|v| serde_json::Value::String(v.clone()))
                                    .collect(),
                            ),
                        );
                    }
                    serde_json::Value::Object(s)
                }
                FieldType::List => {
                    serde_json::json!({"type": "array", "items": {"type": "string"}})
                }
                FieldType::Boolean => {
                    serde_json::json!({"type": "boolean"})
                }
            };

            properties.insert(field.name.clone(), field_schema);

            if field.required && !field.derived {
                required.push(serde_json::Value::String(field.name.clone()));
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }
}

/// Find the ark root by walking up from the current directory
pub fn find_ark_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let ark_dir = current.join(".ark");
        if ark_dir.is_dir() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }
    Err(ArkError::NotInitialized.into())
}

/// Load all schemas from .ark/schemas/
pub fn load_schemas(ark_root: &Path) -> Result<HashMap<String, Schema>> {
    let schemas_dir = ark_root.join(".ark").join("schemas");
    if !schemas_dir.is_dir() {
        return Err(ArkError::NoSchemas.into());
    }

    let mut schemas = HashMap::new();
    let entries = std::fs::read_dir(&schemas_dir).with_context(|| {
        format!(
            "failed to read schemas directory: {}",
            schemas_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read schema: {}", path.display()))?;
            let schema: Schema =
                serde_yml::from_str(&content).map_err(|e| ArkError::SchemaError {
                    path: path.clone(),
                    message: e.to_string(),
                })?;
            schemas.insert(schema.name.clone(), schema);
        }
    }

    if schemas.is_empty() {
        return Err(ArkError::NoSchemas.into());
    }

    // Validate directory containment - no escaping the project root
    for schema in schemas.values() {
        let resolved = ark_root.join(&schema.directory);
        // Check for path traversal (../ or absolute paths)
        if schema.directory.starts_with('/')
            || schema.directory.contains("..")
            || resolved
                .canonicalize()
                .unwrap_or(resolved.clone())
                .strip_prefix(ark_root.canonicalize().unwrap_or(ark_root.to_path_buf()))
                .is_err()
        {
            anyhow::bail!(
                "schema '{}' has directory '{}' that escapes the project root. Directories must be relative paths within the project.",
                schema.name,
                schema.directory
            );
        }
        if let Some(ref archive) = schema.archive
            && (archive.directory.starts_with('/') || archive.directory.contains(".."))
        {
            anyhow::bail!(
                "schema '{}' has archive directory '{}' that escapes the project root.",
                schema.name,
                archive.directory
            );
        }
    }

    // Validate no overlapping directories between schemas
    let dirs: Vec<(&str, &str)> = schemas
        .values()
        .map(|s| (s.name.as_str(), s.directory.as_str()))
        .collect();
    for (i, (name_a, dir_a)) in dirs.iter().enumerate() {
        for (name_b, dir_b) in dirs.iter().skip(i + 1) {
            if dir_a == dir_b {
                anyhow::bail!(
                    "schema conflict: '{}' and '{}' both use directory '{}'. Each artifact type must have its own directory.",
                    name_a,
                    name_b,
                    dir_a
                );
            }
        }
    }

    Ok(schemas)
}

/// Load a single schema by artifact type name
pub fn load_schema(ark_root: &Path, type_name: &str) -> Result<Schema> {
    let schemas = load_schemas(ark_root)?;
    schemas
        .into_iter()
        .find(|(name, _)| name == type_name)
        .map(|(_, schema)| schema)
        .ok_or_else(|| ArkError::UnknownType(type_name.to_string()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schema() -> Schema {
        serde_yml::from_str(
            r#"
name: task
directory: backlog
prefix: BL
fields:
  - name: id
    type: string
    required: true
    derived: true
    pattern: "^BL-\\d{3}$"
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, blocked, done]
    default: "backlog"
  - name: priority
    type: integer
    required: true
    unique: true
  - name: project
    type: enum
    required: true
    values: [ecosystem, alebrije, bellflower]
  - name: type
    type: enum
    required: true
    values: [feature, bug, chore, research, bench]
  - name: tags
    type: list
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
archive:
  field: status
  value: done
  directory: backlog/done
template: |
  ## Context

  ## Acceptance criteria

  - [ ]
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_schema_deserialization() {
        let schema = sample_schema();
        assert_eq!(schema.name, "task");
        assert_eq!(schema.prefix, "BL");
        assert_eq!(schema.directory, "backlog");
        assert_eq!(schema.fields.len(), 9);
    }

    #[test]
    fn test_get_field() {
        let schema = sample_schema();
        let status = schema.get_field("status").unwrap();
        assert_eq!(status.field_type, FieldType::Enum);
        assert_eq!(
            status.values.as_ref().unwrap(),
            &["backlog", "active", "blocked", "done"]
        );
    }

    #[test]
    fn test_json_schema_generation() {
        let schema = sample_schema();
        let json_schema = schema.to_json_schema();
        let props = json_schema["properties"].as_object().unwrap();
        assert!(props.contains_key("title"));
        assert!(props.contains_key("status"));

        let status_schema = &props["status"];
        let enums = status_schema["enum"].as_array().unwrap();
        assert_eq!(enums.len(), 4);

        // derived fields should not be in required
        let required = json_schema["required"].as_array().unwrap();
        let required_names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(!required_names.contains(&"id"));
        assert!(!required_names.contains(&"created"));
        assert!(required_names.contains(&"title"));
    }

    #[test]
    fn test_archive_config() {
        let schema = sample_schema();
        assert_eq!(schema.archive_value(), Some("done"));
        assert_eq!(schema.archive_directory(), Some("backlog/done"));
    }
}
