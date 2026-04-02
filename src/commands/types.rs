use std::path::Path;

use anyhow::Result;

use crate::output::{OutputFormat, render_table};
use crate::schema::load_schemas;

pub fn run(ark_root: &Path, format: &OutputFormat) -> Result<()> {
    let schemas = load_schemas(ark_root)?;

    let headers = &["name", "prefix", "directory", "fields"];
    let mut rows: Vec<Vec<String>> = schemas
        .values()
        .map(|s| {
            vec![
                s.name.clone(),
                s.prefix.clone(),
                s.directory.clone(),
                s.fields.len().to_string(),
            ]
        })
        .collect();

    rows.sort_by(|a, b| a[0].cmp(&b[0]));

    println!("{}", render_table(headers, rows, format));

    Ok(())
}
