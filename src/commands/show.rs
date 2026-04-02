use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::error::ArkError;
use crate::schema::load_schemas;

pub fn run(ark_root: &Path, id: &str) -> Result<()> {
    let schemas = load_schemas(ark_root)?;

    // Determine artifact type from ID prefix
    for schema in schemas.values() {
        if id.starts_with(&schema.prefix) {
            let artifacts = load_artifacts(ark_root, schema)?;
            // Also check archive directory
            let mut all_artifacts = artifacts;
            if let Some(archive_dir) = schema.archive_directory() {
                let archive_path = ark_root.join(archive_dir);
                if archive_path.is_dir() {
                    let archive_schema = crate::schema::Schema {
                        directory: archive_dir.to_string(),
                        ..schema.clone()
                    };
                    if let Ok(archived) = load_artifacts(ark_root, &archive_schema) {
                        all_artifacts.extend(archived);
                    }
                }
            }

            if let Some(artifact) = all_artifacts.iter().find(|a| a.id() == Some(id)) {
                print!("{}", artifact.raw);
                return Ok(())
            }
        }
    }

    Err(ArkError::ArtifactNotFound(id.to_string()).into())
}
