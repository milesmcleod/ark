use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::schema::load_schema;

pub fn run(ark_root: &Path, artifact_type: &str) -> Result<()> {
    let lock_path = ark_root.join(".ark").join(".lock");
    let _lock = crate::lock::acquire_lock(&lock_path)?;

    let schema = load_schema(ark_root, artifact_type)?;

    let archive_value = match schema.archive_value() {
        Some(v) => v.to_string(),
        None => {
            println!(
                "No archive configuration for '{}'. Add an 'archive' section to the schema.",
                artifact_type
            );
            return Ok(());
        }
    };

    let archive_dir = match schema.archive_directory() {
        Some(d) => ark_root.join(d),
        None => {
            println!("No archive directory configured for '{}'.", artifact_type);
            return Ok(());
        }
    };

    let artifacts = load_artifacts(ark_root, &schema)?;
    let to_archive: Vec<_> = artifacts
        .iter()
        .filter(|a| a.status() == Some(archive_value.as_str()))
        .collect();

    if to_archive.is_empty() {
        println!(
            "No {} artifacts with status '{}' to archive.",
            artifact_type, archive_value
        );
        return Ok(());
    }

    std::fs::create_dir_all(&archive_dir)?;

    let mut count = 0;
    for artifact in &to_archive {
        let filename = artifact
            .path
            .file_name()
            .expect("artifact should have a filename");
        let dest = archive_dir.join(filename);
        std::fs::rename(&artifact.path, &dest)?;

        // Fire archive hooks
        if let Some(id) = artifact.id() {
            crate::commands::hooks::run_archive_hooks(ark_root, artifact_type, id);
        }

        count += 1;
    }

    println!(
        "Archived {} {} to {}.",
        count,
        artifact_type,
        archive_dir.display()
    );

    Ok(())
}
