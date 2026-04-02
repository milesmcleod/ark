use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result};

use crate::schema::load_schemas_raw;

pub fn run(ark_root: &Path) -> Result<()> {
    let raw_schemas = load_schemas_raw(ark_root)?;

    let registry_schemas: Vec<_> = raw_schemas
        .into_iter()
        .filter(|(_, schema)| schema.registry.is_some())
        .collect();

    if registry_schemas.is_empty() {
        println!("no schemas with registry URLs found");
        return Ok(());
    }

    let mut updated = 0;
    let mut errors = 0;

    for (path, schema) in &registry_schemas {
        let url = schema.registry.as_ref().unwrap();
        eprint!("pulling {} from {} ... ", schema.name, url);

        match fetch_url(url) {
            Ok(content) => {
                // Validate that the fetched content is valid YAML and a valid schema
                match serde_yml::from_str::<crate::schema::Schema>(&content) {
                    Ok(fetched) => {
                        if fetched.name != schema.name {
                            eprintln!(
                                "warning: fetched schema name '{}' does not match local name '{}', skipping",
                                fetched.name, schema.name
                            );
                            errors += 1;
                            continue;
                        }
                        std::fs::write(path, &content).with_context(|| {
                            format!("failed to write schema: {}", path.display())
                        })?;
                        eprintln!("ok");
                        updated += 1;
                    }
                    Err(e) => {
                        eprintln!("invalid schema: {}", e);
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("failed: {}", e);
                errors += 1;
            }
        }
    }

    println!(
        "registry pull complete: {} updated, {} errors",
        updated, errors
    );

    Ok(())
}

fn fetch_url(url: &str) -> Result<String> {
    let output = ProcessCommand::new("curl")
        .args(["-fsSL", "--max-time", "30", url])
        .output()
        .context("failed to run curl - is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("curl failed: {}", stderr.trim());
    }

    String::from_utf8(output.stdout).context("response was not valid UTF-8")
}
