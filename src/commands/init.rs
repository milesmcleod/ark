use std::path::Path;

use anyhow::Result;

use crate::error::ArkError;

pub fn run(cwd: &Path) -> Result<()> {
    let ark_dir = cwd.join(".ark");
    if ark_dir.is_dir() {
        return Err(ArkError::AlreadyInitialized.into());
    }

    let schemas_dir = ark_dir.join("schemas");
    std::fs::create_dir_all(&schemas_dir)?;

    println!("Initialized ark in .ark/");
    println!();
    println!("ark needs schema files to manage artifacts. Schemas define artifact");
    println!("types (tasks, specs, ADRs, or whatever you need) and live in .ark/schemas/.");
    println!();
    println!("Each schema file declares:");
    println!("  - name, prefix, and directory for the artifact type");
    println!("  - fields with types, constraints, and valid values");
    println!("  - a template for the artifact body");
    println!();
    println!("Next steps:");
    println!("  - Create schema files in .ark/schemas/ (one .yml per artifact type)");
    println!("  - Run `ark types` to verify your schemas are loaded");
    println!("  - Run `ark new <type>` to create your first artifact");
    println!();
    println!("Run `ark schema-help` to see the schema format reference.");

    Ok(())
}
