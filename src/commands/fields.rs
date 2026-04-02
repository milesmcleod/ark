use std::path::Path;

use anyhow::Result;

use crate::error::ArkError;
use crate::output::{render_table, OutputFormat};
use crate::schema::load_schema;

pub fn run(
    ark_root: &Path,
    artifact_type: &str,
    field: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let schema = load_schema(ark_root, artifact_type)?;

    if let Some(field_name) = field {
        // Show values for a specific field
        let field_def = schema.get_field(field_name).ok_or_else(|| {
            ArkError::UnknownField {
                artifact_type: artifact_type.to_string(),
                field: field_name.to_string(),
            }
        })?;

        match &field_def.values {
            Some(values) => {
                for v in values {
                    println!("{}", v);
                }
            }
            None => {
                println!(
                    "Field '{}' is type '{}' with no enumerated values.",
                    field_name,
                    format!("{:?}", field_def.field_type).to_lowercase()
                );
            }
        }
    } else {
        // Show all fields
        let headers = &["name", "type", "required", "unique", "derived", "values"];
        let rows: Vec<Vec<String>> = schema
            .fields
            .iter()
            .map(|f| {
                vec![
                    f.name.clone(),
                    format!("{:?}", f.field_type).to_lowercase(),
                    if f.required { "yes" } else { "" }.into(),
                    if f.unique { "yes" } else { "" }.into(),
                    if f.derived { "yes" } else { "" }.into(),
                    f.values
                        .as_ref()
                        .map(|v| v.join(", "))
                        .unwrap_or_default(),
                ]
            })
            .collect();

        println!("{}", render_table(headers, rows, format));
    }

    Ok(())
}
