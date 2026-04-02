use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gray_matter::Matter;
use gray_matter::engine::YAML;

use crate::schema::Schema;

#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: PathBuf,
    pub frontmatter: HashMap<String, serde_json::Value>,
    pub body: String,
    pub raw: String,
}

impl Artifact {
    /// Parse a markdown file into an Artifact
    pub fn from_file(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read artifact: {}", path.display()))?;
        Self::from_str(&raw, path.to_path_buf())
    }

    /// Parse a string into an Artifact
    pub fn from_str(content: &str, path: PathBuf) -> Result<Self> {
        let matter = Matter::<YAML>::new();
        let parsed = matter
            .parse(content)
            .unwrap_or_else(|_| gray_matter::ParsedEntity {
                data: None,
                content: content.to_string(),
                excerpt: None,
                orig: content.to_string(),
                matter: String::new(),
            });

        let frontmatter = if let Some(data) = parsed.data {
            pod_to_map(&data)
        } else {
            HashMap::new()
        };

        Ok(Self {
            path,
            frontmatter,
            body: parsed.content,
            raw: content.to_string(),
        })
    }

    pub fn id(&self) -> Option<&str> {
        self.frontmatter.get("id").and_then(|v| v.as_str())
    }

    pub fn title(&self) -> Option<&str> {
        self.frontmatter.get("title").and_then(|v| v.as_str())
    }

    pub fn status(&self) -> Option<&str> {
        self.frontmatter.get("status").and_then(|v| v.as_str())
    }

    pub fn priority(&self) -> Option<i64> {
        self.frontmatter.get("priority").and_then(|v| v.as_i64())
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.frontmatter.get(key).and_then(|v| v.as_str())
    }

    pub fn get_list(&self, key: &str) -> Vec<String> {
        self.frontmatter
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Convert frontmatter to serde_json::Value for schema validation
    pub fn frontmatter_as_json(&self) -> serde_json::Value {
        serde_json::Value::Object(
            self.frontmatter
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    }

    /// Serialize the artifact back to markdown with frontmatter
    pub fn to_markdown(&self) -> String {
        let yaml = frontmatter_to_yaml(&self.frontmatter);
        if self.body.is_empty() {
            format!("---\n{}---\n", yaml)
        } else if self.body.starts_with('\n') {
            format!("---\n{}---\n{}", yaml, self.body)
        } else {
            format!("---\n{}---\n\n{}", yaml, self.body)
        }
    }
}

/// Load all artifacts for a given schema
pub fn load_artifacts(ark_root: &Path, schema: &Schema) -> Result<Vec<Artifact>> {
    let dir = ark_root.join(&schema.directory);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut artifacts = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|e| e == "md" || e == "feature")
        {
            match Artifact::from_file(&path) {
                Ok(artifact) => artifacts.push(artifact),
                Err(e) => {
                    eprintln!("warning: failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(artifacts)
}

/// Find the next available ID number for a schema prefix
pub fn next_id(artifacts: &[Artifact], prefix: &str) -> u32 {
    let max = artifacts
        .iter()
        .filter_map(|a| a.id())
        .filter_map(|id| {
            id.strip_prefix(prefix)
                .and_then(|rest| rest.strip_prefix('-'))
                .and_then(|num| num.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    max + 1
}

/// Slugify a title for use in filenames
pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Convert gray_matter Pod to a HashMap of serde_json::Value
fn pod_to_map(pod: &gray_matter::Pod) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    if let Ok(hash) = pod.as_hashmap() {
        for (key, value) in &hash {
            map.insert(key.clone(), pod_to_json(value));
        }
    }
    map
}

/// Convert a gray_matter Pod to serde_json::Value
fn pod_to_json(pod: &gray_matter::Pod) -> serde_json::Value {
    use gray_matter::Pod;
    match pod {
        Pod::Null => serde_json::Value::Null,
        Pod::String(s) => serde_json::Value::String(s.clone()),
        Pod::Integer(i) => serde_json::json!(i),
        Pod::Float(f) => serde_json::json!(f),
        Pod::Boolean(b) => serde_json::Value::Bool(*b),
        Pod::Array(arr) => serde_json::Value::Array(arr.iter().map(pod_to_json).collect()),
        Pod::Hash(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), pod_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
    }
}

/// Serialize frontmatter HashMap to YAML string using serde_yml.
/// We control field ordering; serde_yml handles all quoting and escaping.
fn frontmatter_to_yaml(frontmatter: &HashMap<String, serde_json::Value>) -> String {
    use serde_yml::Value as YamlValue;

    // Build an ordered mapping
    let mut mapping = serde_yml::Mapping::new();

    // Priority keys first, in this order
    let priority_keys = [
        "id", "title", "status", "priority", "project", "type", "tags", "created", "updated",
    ];

    for key in &priority_keys {
        if let Some(value) = frontmatter.get(*key) {
            mapping.insert(YamlValue::String(key.to_string()), json_to_yaml(value));
        }
    }

    // Remaining keys alphabetically
    let mut remaining: Vec<_> = frontmatter
        .keys()
        .filter(|k| !priority_keys.contains(&k.as_str()))
        .collect();
    remaining.sort();
    for key in remaining {
        if let Some(value) = frontmatter.get(key.as_str()) {
            mapping.insert(YamlValue::String(key.to_string()), json_to_yaml(value));
        }
    }

    serde_yml::to_string(&mapping).unwrap_or_default()
}

/// Convert a serde_json::Value to a serde_yml::Value
fn json_to_yaml(value: &serde_json::Value) -> serde_yml::Value {
    match value {
        serde_json::Value::Null => serde_yml::Value::Null,
        serde_json::Value::Bool(b) => serde_yml::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_yml::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_yml::Value::Number(serde_yml::Number::from(f))
            } else {
                serde_yml::Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => serde_yml::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            serde_yml::Value::Sequence(arr.iter().map(json_to_yaml).collect())
        }
        serde_json::Value::Object(map) => {
            let mut m = serde_yml::Mapping::new();
            for (k, v) in map {
                m.insert(serde_yml::Value::String(k.clone()), json_to_yaml(v));
            }
            serde_yml::Value::Mapping(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_artifact() {
        let content = r#"---
id: BL-001
title: Build prototype
status: active
priority: 10
tags: [hardware, bellflower]
created: 2026-04-01
---

## Context

This is the body.
"#;
        let artifact = Artifact::from_str(content, PathBuf::from("test.md")).unwrap();
        assert_eq!(artifact.id(), Some("BL-001"));
        assert_eq!(artifact.title(), Some("Build prototype"));
        assert_eq!(artifact.status(), Some("active"));
        assert_eq!(artifact.priority(), Some(10));
        assert_eq!(artifact.get_list("tags"), vec!["hardware", "bellflower"]);
        assert!(artifact.body.contains("This is the body."));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "Just a plain markdown file.\n";
        let artifact = Artifact::from_str(content, PathBuf::from("test.md")).unwrap();
        assert!(artifact.frontmatter.is_empty());
        assert!(artifact.body.contains("Just a plain markdown file."));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify("Build Bellflower Prototype"),
            "build-bellflower-prototype"
        );
        assert_eq!(slugify("Fix bug #42"), "fix-bug-42");
        assert_eq!(slugify("  lots   of   spaces  "), "lots-of-spaces");
    }

    #[test]
    fn test_next_id() {
        let artifacts = vec![
            Artifact::from_str(
                "---\nid: BL-001\ntitle: First\n---\n",
                PathBuf::from("a.md"),
            )
            .unwrap(),
            Artifact::from_str(
                "---\nid: BL-003\ntitle: Third\n---\n",
                PathBuf::from("b.md"),
            )
            .unwrap(),
        ];
        assert_eq!(next_id(&artifacts, "BL"), 4);
    }

    #[test]
    fn test_next_id_empty() {
        assert_eq!(next_id(&[], "BL"), 1);
    }

    #[test]
    fn test_roundtrip_markdown() {
        let content = r#"---
id: BL-001
title: Test task
status: backlog
priority: 10
---

## Context

Some body text.
"#;
        let artifact = Artifact::from_str(content, PathBuf::from("test.md")).unwrap();
        let output = artifact.to_markdown();
        // Re-parse and verify
        let reparsed = Artifact::from_str(&output, PathBuf::from("test.md")).unwrap();
        assert_eq!(reparsed.id(), Some("BL-001"));
        assert_eq!(reparsed.title(), Some("Test task"));
        assert_eq!(reparsed.status(), Some("backlog"));
        assert_eq!(reparsed.priority(), Some(10));
    }

    #[test]
    fn test_roundtrip_title_with_colon() {
        let content = r#"---
id: BL-001
title: "Something: with a colon"
status: backlog
priority: 10
---

Body here.
"#;
        let artifact = Artifact::from_str(content, PathBuf::from("test.md")).unwrap();
        assert_eq!(artifact.title(), Some("Something: with a colon"));

        // Roundtrip should preserve the title
        let output = artifact.to_markdown();
        let reparsed = Artifact::from_str(&output, PathBuf::from("test.md")).unwrap();
        assert_eq!(reparsed.title(), Some("Something: with a colon"));
    }
}
