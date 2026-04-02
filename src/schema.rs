use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::error::ArkError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    #[serde(default)]
    pub directory: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
    #[serde(default)]
    pub archive: Option<ArchiveDef>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub registry: Option<String>,
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

/// Resolve schema inheritance by merging base fields into children.
/// Child schemas can override fields (by name), directory, prefix, archive, and template.
/// Circular inheritance is detected and returns an error.
fn resolve_inheritance(schemas: &mut HashMap<String, Schema>) -> Result<()> {
    // Collect which schemas extend others
    let extends_map: HashMap<String, String> = schemas
        .iter()
        .filter_map(|(name, schema)| {
            schema
                .extends
                .as_ref()
                .map(|base| (name.clone(), base.clone()))
        })
        .collect();

    // Detect circular inheritance
    for start in extends_map.keys() {
        let mut visited = HashSet::new();
        let mut current = start.as_str();
        while let Some(base) = extends_map.get(current) {
            if !visited.insert(current.to_string()) {
                anyhow::bail!(
                    "circular schema inheritance detected: '{}' is part of an inheritance cycle",
                    start
                );
            }
            current = base.as_str();
        }
    }

    // Resolve in topological order - process schemas whose bases are already resolved
    let mut resolved: HashSet<String> = schemas
        .keys()
        .filter(|name| !extends_map.contains_key(name.as_str()))
        .cloned()
        .collect();

    let mut pending: Vec<String> = extends_map.keys().cloned().collect();

    while !pending.is_empty() {
        let mut progress = false;
        let mut still_pending = Vec::new();

        for name in pending {
            let base_name = extends_map.get(&name).unwrap();
            if resolved.contains(base_name) {
                // Resolve this schema
                let base = schemas.get(base_name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "schema '{}' extends '{}', but '{}' was not found",
                        name,
                        base_name,
                        base_name
                    )
                })?;

                let base_fields = base.fields.clone();
                let base_directory = base.directory.clone();
                let base_prefix = base.prefix.clone();
                let base_archive = base.archive.clone();
                let base_template = base.template.clone();

                let child = schemas.get_mut(&name).unwrap();

                // Merge fields: start with base fields, then overlay child fields
                let child_field_names: HashSet<&str> =
                    child.fields.iter().map(|f| f.name.as_str()).collect();
                let mut merged_fields: Vec<FieldDef> = base_fields
                    .into_iter()
                    .filter(|f| !child_field_names.contains(f.name.as_str()))
                    .collect();
                merged_fields.append(&mut child.fields);
                child.fields = merged_fields;

                // Inherit directory and prefix if not set by child
                if child.directory.is_empty() {
                    child.directory = base_directory;
                }
                if child.prefix.is_empty() {
                    child.prefix = base_prefix;
                }
                if child.archive.is_none() {
                    child.archive = base_archive;
                }
                if child.template.is_none() {
                    child.template = base_template;
                }

                resolved.insert(name);
                progress = true;
            } else {
                still_pending.push(name);
            }
        }

        if !progress {
            anyhow::bail!(
                "could not resolve schema inheritance for: {}",
                still_pending.join(", ")
            );
        }

        pending = still_pending;
    }

    Ok(())
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

    // Resolve schema inheritance before validation
    resolve_inheritance(&mut schemas)?;

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
    // Skip schemas that serve only as inheritance bases (have children that extend them)
    let base_names: HashSet<String> = schemas.values().filter_map(|s| s.extends.clone()).collect();
    let dirs: Vec<(&str, &str)> = schemas
        .values()
        .filter(|s| !base_names.contains(&s.name))
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

/// Load a single schema by artifact type name.
/// Loads all schemas and resolves inheritance to ensure the returned
/// schema has all inherited fields merged.
pub fn load_schema(ark_root: &Path, type_name: &str) -> Result<Schema> {
    let schemas_dir = ark_root.join(".ark").join("schemas");
    if !schemas_dir.is_dir() {
        return Err(ArkError::NoSchemas.into());
    }

    // Load all raw schemas from disk into a map so we can resolve inheritance
    let mut all_schemas = HashMap::new();
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
            all_schemas.insert(schema.name.clone(), schema);
        }
    }

    if !all_schemas.contains_key(type_name) {
        return Err(ArkError::UnknownType(type_name.to_string()).into());
    }

    // Resolve inheritance for all schemas (needed to resolve chains)
    resolve_inheritance(&mut all_schemas)?;

    all_schemas
        .remove(type_name)
        .ok_or_else(|| ArkError::UnknownType(type_name.to_string()).into())
}

/// Load all raw schemas from .ark/schemas/ with their registry URLs.
/// Does NOT resolve inheritance. Used by registry-pull to find URLs.
pub fn load_schemas_raw(ark_root: &Path) -> Result<Vec<(PathBuf, Schema)>> {
    let schemas_dir = ark_root.join(".ark").join("schemas");
    if !schemas_dir.is_dir() {
        return Err(ArkError::NoSchemas.into());
    }

    let mut result = Vec::new();
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
            result.push((path, schema));
        }
    }

    Ok(result)
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
    values: [ecosystem, frontend, backend]
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

    #[test]
    fn test_inheritance_merges_fields() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "base-task".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: base-task
directory: backlog
prefix: BL
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, blocked, done]
    default: backlog
"#,
            )
            .unwrap(),
        );
        schemas.insert(
            "task".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: task
extends: base-task
fields:
  - name: priority
    type: integer
    required: true
    unique: true
  - name: project
    type: enum
    required: true
    values: [alpha, beta]
"#,
            )
            .unwrap(),
        );

        resolve_inheritance(&mut schemas).unwrap();

        let task = &schemas["task"];
        // Should have 5 fields: id, title, status from base + priority, project from child
        assert_eq!(task.fields.len(), 5);
        assert!(task.get_field("id").is_some());
        assert!(task.get_field("title").is_some());
        assert!(task.get_field("status").is_some());
        assert!(task.get_field("priority").is_some());
        assert!(task.get_field("project").is_some());
        // Should inherit directory and prefix from base
        assert_eq!(task.directory, "backlog");
        assert_eq!(task.prefix, "BL");
    }

    #[test]
    fn test_inheritance_child_overrides_field() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "base-task".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: base-task
directory: backlog
prefix: BL
fields:
  - name: status
    type: enum
    required: true
    values: [backlog, active, done]
"#,
            )
            .unwrap(),
        );
        schemas.insert(
            "task".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: task
extends: base-task
directory: tasks
prefix: TK
fields:
  - name: status
    type: enum
    required: true
    values: [open, closed]
"#,
            )
            .unwrap(),
        );

        resolve_inheritance(&mut schemas).unwrap();

        let task = &schemas["task"];
        // Child's override of status should win
        let status = task.get_field("status").unwrap();
        assert_eq!(status.values.as_ref().unwrap(), &["open", "closed"]);
        // Child overrides directory and prefix
        assert_eq!(task.directory, "tasks");
        assert_eq!(task.prefix, "TK");
    }

    #[test]
    fn test_inheritance_circular_detection() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "a".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: a
directory: a
prefix: A
extends: b
fields: []
"#,
            )
            .unwrap(),
        );
        schemas.insert(
            "b".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: b
directory: b
prefix: B
extends: a
fields: []
"#,
            )
            .unwrap(),
        );

        let result = resolve_inheritance(&mut schemas);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("circular") || err_msg.contains("could not resolve"),
            "expected circular inheritance error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_inheritance_missing_base() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "task".to_string(),
            serde_yml::from_str::<Schema>(
                r#"
name: task
extends: nonexistent
fields: []
"#,
            )
            .unwrap(),
        );

        let result = resolve_inheritance(&mut schemas);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_with_registry_field() {
        let schema: Schema = serde_yml::from_str(
            r#"
name: task
directory: backlog
prefix: BL
registry: https://example.com/task.yml
fields:
  - name: id
    type: string
    required: true
"#,
        )
        .unwrap();
        assert_eq!(
            schema.registry.as_deref(),
            Some("https://example.com/task.yml")
        );
    }
}
