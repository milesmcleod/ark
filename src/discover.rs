use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::artifact::{Artifact, load_artifacts};
use crate::schema::{self, Schema};

/// A discovered ark project with its root path, schemas, and loaded artifacts.
#[derive(Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub root: PathBuf,
    pub schemas: HashMap<String, Schema>,
}

/// An artifact with its project context attached.
#[derive(Debug)]
pub struct ProjectArtifact {
    pub project: String,
    pub artifact: Artifact,
}

/// Directories to always skip during recursive discovery.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".next",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
];

/// Load .arkignore patterns from a file. Returns an empty set if the file doesn't exist.
fn load_arkignore(root: &Path) -> GlobSet {
    let ignore_path = root.join(".arkignore");
    let content = match std::fs::read_to_string(&ignore_path) {
        Ok(c) => c,
        Err(_) => return GlobSet::empty(),
    };

    let mut builder = GlobSetBuilder::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Ok(glob) = Glob::new(line) {
            builder.add(glob);
        } else {
            eprintln!("  warning: invalid .arkignore pattern: {}", line);
        }
    }

    builder.build().unwrap_or_else(|_| GlobSet::empty())
}

/// Recursively discover all ark projects below a root directory.
/// Respects .arkignore at the scan root.
pub fn discover_projects(root: &Path) -> Result<Vec<ProjectInfo>> {
    let ignore = load_arkignore(root);
    let mut projects = Vec::new();
    discover_recursive(root, root, &ignore, &mut projects)?;

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

fn discover_recursive(
    dir: &Path,
    scan_root: &Path,
    ignore: &GlobSet,
    projects: &mut Vec<ProjectInfo>,
) -> Result<()> {
    let ark_dir = dir.join(".ark");
    if ark_dir.is_dir() {
        let name = if dir == scan_root {
            ".".to_string()
        } else {
            dir.strip_prefix(scan_root)
                .unwrap_or(dir)
                .to_string_lossy()
                .to_string()
        };

        if let Ok(schemas) = schema::load_schemas(dir) {
            projects.push(ProjectInfo {
                name,
                root: dir.to_path_buf(),
                schemas,
            });
        }
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();

        // Skip hidden directories
        if dir_name_str.starts_with('.') {
            continue;
        }

        // Skip known junk directories
        if SKIP_DIRS.contains(&dir_name_str.as_ref()) {
            continue;
        }

        // Skip directories matching .arkignore patterns
        let relative = path
            .strip_prefix(scan_root)
            .unwrap_or(&path)
            .to_string_lossy();
        if ignore.is_match(relative.as_ref()) || ignore.is_match(dir_name_str.as_ref()) {
            continue;
        }

        discover_recursive(&path, scan_root, ignore, projects)?;
    }

    Ok(())
}

/// Load all artifacts of matching type names across all discovered projects.
pub fn load_matching_artifacts(
    projects: &[ProjectInfo],
    type_names: &str,
) -> Result<Vec<ProjectArtifact>> {
    let names: Vec<&str> = type_names.split(',').map(|s| s.trim()).collect();
    let mut results = Vec::new();

    for project in projects {
        for name in &names {
            if let Some(schema) = project.schemas.get(*name) {
                let artifacts = load_artifacts(&project.root, schema).with_context(|| {
                    format!("failed to load {} artifacts from {}", name, project.name)
                })?;

                let archive_value = schema.archive_value();

                for artifact in artifacts {
                    if let Some(av) = archive_value
                        && artifact.status() == Some(av)
                    {
                        continue;
                    }
                    results.push(ProjectArtifact {
                        project: project.name.clone(),
                        artifact,
                    });
                }
            }
        }
    }

    Ok(results)
}

/// Collect all unique (project, type_name) pairs across discovered projects.
pub fn collect_type_info(projects: &[ProjectInfo]) -> Vec<TypeEntry> {
    let mut entries = Vec::new();
    for project in projects {
        for schema in project.schemas.values() {
            entries.push(TypeEntry {
                project: project.name.clone(),
                name: schema.name.clone(),
                prefix: schema.prefix.clone(),
                directory: schema.directory.clone(),
                field_count: schema.fields.len(),
            });
        }
    }
    entries.sort_by(|a, b| a.project.cmp(&b.project).then(a.name.cmp(&b.name)));
    entries
}

#[derive(Debug)]
pub struct TypeEntry {
    pub project: String,
    pub name: String,
    pub prefix: String,
    pub directory: String,
    pub field_count: usize,
}
