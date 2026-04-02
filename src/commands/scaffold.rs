use std::path::Path;

use anyhow::{Result, bail};

/// ark scaffold <template-dir> - set up a new project from a template directory containing schemas
///
/// The template directory should contain .yml schema files (no .ark/ wrapper needed).
/// ark scaffold copies them into .ark/schemas/, runs init if needed, and reports what was set up.
pub fn run(cwd: &Path, template: &str) -> Result<()> {
    let template_path = Path::new(template);

    // Template can be a local directory path
    if !template_path.is_dir() {
        bail!(
            "template '{}' is not a directory. Provide a path to a directory containing .yml schema files.",
            template
        );
    }

    // Initialize ark if needed
    let ark_dir = cwd.join(".ark");
    let schemas_dir = ark_dir.join("schemas");
    if !ark_dir.is_dir() {
        std::fs::create_dir_all(&schemas_dir)?;
        println!("Initialized ark in .ark/");
    } else if !schemas_dir.is_dir() {
        std::fs::create_dir_all(&schemas_dir)?;
    }

    // Copy schema files from template
    let entries = std::fs::read_dir(template_path)?;
    let mut count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
            let filename = entry.file_name();
            let dest = schemas_dir.join(&filename);

            if dest.exists() {
                eprintln!(
                    "  warning: {} already exists, skipping (use --force to overwrite)",
                    filename.to_string_lossy()
                );
                continue;
            }

            std::fs::copy(&path, &dest)?;
            println!("  Copied {}", filename.to_string_lossy());
            count += 1;
        }
    }

    if count == 0 {
        println!("No new schema files to copy from {}.", template);
    } else {
        println!(
            "\nScaffolded {} schema(s) from {}.",
            count,
            template_path.display()
        );
        println!("Run `ark types` to verify, then `ark new <type>` to create artifacts.");
    }

    Ok(())
}
