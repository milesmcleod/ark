use std::path::Path;

use anyhow::Result;

use crate::artifact::load_artifacts;
use crate::cli::ListArgs;
use crate::output::{OutputFormat, render_table};
use crate::schema::{FieldType, load_schema};

pub fn run(ark_root: &Path, args: &ListArgs, format: &OutputFormat) -> Result<()> {
    let schema = load_schema(ark_root, &args.artifact_type)?;
    let mut artifacts = load_artifacts(ark_root, &schema)?;

    // Exclude archived items by default
    if let Some(archive_value) = schema.archive_value() {
        artifacts.retain(|a| a.status() != Some(archive_value));
    }

    // Apply filters
    if let Some(ref status) = args.status {
        artifacts.retain(|a| a.status() == Some(status.as_str()));
    }
    if let Some(ref project) = args.project {
        artifacts.retain(|a| a.get_str("project") == Some(project.as_str()));
    }
    if let Some(ref kind) = args.kind {
        // Filter by the 'kind' flag which maps to whatever enum-like field
        // the user intends. Check common field names.
        artifacts.retain(|a| {
            a.get_str("type") == Some(kind.as_str()) || a.get_str("kind") == Some(kind.as_str())
        });
    }
    if let Some(ref tag) = args.tag {
        artifacts.retain(|a| a.get_list("tags").iter().any(|t| t == tag));
    }

    // Sort by priority (items without priority go last)
    artifacts.sort_by(|a, b| {
        let pa = a.priority().unwrap_or(i64::MAX);
        let pb = b.priority().unwrap_or(i64::MAX);
        pa.cmp(&pb)
    });

    // Apply limit
    if let Some(limit) = args.limit {
        artifacts.truncate(limit);
    }

    if artifacts.is_empty() {
        let type_name = &args.artifact_type;
        println!("No {type_name} artifacts found. Create one with `ark new {type_name}`.");
        return Ok(());
    }

    // Build display columns dynamically from schema fields
    // Show: id, priority (as "pri"), then all non-derived scalar fields except id/title,
    // then title last (it's the widest)
    let display_fields: Vec<&crate::schema::FieldDef> = schema
        .fields
        .iter()
        .filter(|f| {
            !f.derived && f.name != "id" && f.name != "title" && f.field_type != FieldType::List
        })
        .collect();

    let mut headers: Vec<String> = vec!["id".into()];
    if schema.priority_field().is_some() {
        headers.push("pri".into());
    }
    for field in &display_fields {
        if field.name == "priority" {
            continue; // already added as "pri"
        }
        headers.push(field.name.clone());
    }
    headers.push("title".into());

    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();

    let rows: Vec<Vec<String>> = artifacts
        .iter()
        .map(|a| {
            let mut row = vec![a.id().unwrap_or("-").to_string()];
            if schema.priority_field().is_some() {
                row.push(a.priority().map(|p| p.to_string()).unwrap_or("-".into()));
            }
            for field in &display_fields {
                if field.name == "priority" {
                    continue;
                }
                row.push(a.get_str(&field.name).unwrap_or("-").to_string());
            }
            row.push(a.title().unwrap_or("-").to_string());
            row
        })
        .collect();

    println!("{}", render_table(&header_refs, rows, format));

    Ok(())
}
