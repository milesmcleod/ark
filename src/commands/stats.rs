use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::output::{OutputFormat, render_table};
use crate::schema::{load_schema, load_schemas};

pub fn run(
    ark_root: &Path,
    artifact_type: Option<&str>,
    group_by: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    if let Some(type_name) = artifact_type {
        let schema = load_schema(ark_root, type_name)?;
        let artifacts = load_artifacts(ark_root, &schema)?;

        if let Some(field) = group_by {
            // Group by a specific field
            let mut groups: HashMap<String, usize> = HashMap::new();
            for artifact in &artifacts {
                let value = artifact.get_str(field).unwrap_or("(none)").to_string();
                *groups.entry(value).or_default() += 1;
            }

            let headers = &[field, "count"];
            let mut rows: Vec<Vec<String>> = groups
                .into_iter()
                .map(|(k, v)| vec![k, v.to_string()])
                .collect();
            rows.sort_by(|a, b| a[0].cmp(&b[0]));

            println!("{}", render_table(headers, rows, format));
        } else {
            println!("{}: {} artifacts", type_name, artifacts.len());
        }
    } else {
        // Show stats for all types
        let schemas = load_schemas(ark_root)?;
        let headers = &["type", "count", "directory"];
        let mut rows: Vec<Vec<String>> = Vec::new();

        for schema in schemas.values() {
            let artifacts = load_artifacts(ark_root, schema)?;
            rows.push(vec![
                schema.name.clone(),
                artifacts.len().to_string(),
                schema.directory.clone(),
            ]);
        }

        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        println!("{}", render_table(headers, rows, format));
    }

    Ok(())
}
