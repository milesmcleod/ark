use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::output::{OutputFormat, render_table};
use crate::schema::{load_schema, load_schemas};

pub fn run(
    ark_root: &Path,
    pattern: &str,
    artifact_type: Option<&str>,
    ignore_case: bool,
    format: &OutputFormat,
) -> Result<()> {
    let effective_pattern = if ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let re = regex::Regex::new(&effective_pattern)
        .map_err(|e| anyhow::anyhow!("invalid regex pattern: {}", e))?;

    let schemas = if let Some(type_name) = artifact_type {
        let schema = load_schema(ark_root, type_name)?;
        vec![schema]
    } else {
        let schemas = load_schemas(ark_root)?;
        schemas.into_values().collect()
    };

    let mut matches = Vec::new();
    for schema in &schemas {
        let artifacts = load_artifacts(ark_root, schema)?;
        for artifact in artifacts {
            // Search in body and title
            let in_body = re.is_match(&artifact.body);
            let in_title = artifact.title().is_some_and(|t| re.is_match(t));
            if in_body || in_title {
                matches.push((schema.name.clone(), artifact));
            }
        }
    }

    if matches.is_empty() {
        println!("No artifacts matching '{}' found.", pattern);
        return Ok(());
    }

    let headers = &["type", "id", "title", "match_in"];
    let rows: Vec<Vec<String>> = matches
        .iter()
        .map(|(type_name, a)| {
            let in_body = re.is_match(&a.body);
            let in_title = a.title().is_some_and(|t| re.is_match(t));
            let match_in = match (in_title, in_body) {
                (true, true) => "title, body",
                (true, false) => "title",
                (false, true) => "body",
                _ => "",
            };
            vec![
                type_name.clone(),
                a.id().unwrap_or("-").to_string(),
                a.title().unwrap_or("-").to_string(),
                match_in.to_string(),
            ]
        })
        .collect();

    println!("{}", render_table(headers, rows, format));

    Ok(())
}
