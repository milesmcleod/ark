use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::discover::{collect_type_info, discover_projects, load_matching_artifacts};
use crate::output::{OutputFormat, render_table};

/// ark scan types - show all artifact types across all nested projects
pub fn run_types(cwd: &Path, format: &OutputFormat) -> Result<()> {
    let projects = discover_projects(cwd)?;

    if projects.is_empty() {
        println!("No ark projects found below current directory.");
        println!(
            "Run `ark init` to initialize a project, or cd to a directory containing ark projects."
        );
        return Ok(());
    }

    let entries = collect_type_info(&projects);

    if entries.is_empty() {
        println!(
            "Found {} ark projects but none have valid schemas.",
            projects.len()
        );
        return Ok(());
    }

    let headers = &["project", "type", "prefix", "directory", "fields"];
    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            vec![
                e.project.clone(),
                e.name.clone(),
                e.prefix.clone(),
                e.directory.clone(),
                e.field_count.to_string(),
            ]
        })
        .collect();

    println!("{}", render_table(headers, rows, format));

    Ok(())
}

/// ark scan list <types> - list artifacts of matching types across all projects
pub fn run_list(
    cwd: &Path,
    type_names: &str,
    status: Option<&str>,
    project_filter: Option<&str>,
    limit: Option<usize>,
    format: &OutputFormat,
) -> Result<()> {
    let projects = discover_projects(cwd)?;
    let mut artifacts = load_matching_artifacts(&projects, type_names)?;

    // Apply filters
    if let Some(status) = status {
        artifacts.retain(|pa| pa.artifact.status() == Some(status));
    }
    if let Some(proj) = project_filter {
        artifacts.retain(|pa| pa.project == proj);
    }

    // Sort: by priority if available, then by project name, then by ID
    artifacts.sort_by(|a, b| {
        let pa = a.artifact.priority().unwrap_or(i64::MAX);
        let pb = b.artifact.priority().unwrap_or(i64::MAX);
        pa.cmp(&pb)
            .then(a.project.cmp(&b.project))
            .then(a.artifact.id().cmp(&b.artifact.id()))
    });

    if let Some(limit) = limit {
        artifacts.truncate(limit);
    }

    if artifacts.is_empty() {
        println!(
            "No artifacts matching type(s) '{}' found across {} projects.",
            type_names,
            projects.len()
        );
        return Ok(());
    }

    // Determine which columns to show based on what fields exist
    let has_priority = artifacts.iter().any(|pa| pa.artifact.priority().is_some());
    let has_status = artifacts.iter().any(|pa| pa.artifact.status().is_some());

    let mut headers = vec!["project", "id"];
    if has_priority {
        headers.push("pri");
    }
    if has_status {
        headers.push("status");
    }
    headers.push("title");

    let rows: Vec<Vec<String>> = artifacts
        .iter()
        .map(|pa| {
            let mut row = vec![
                pa.project.clone(),
                pa.artifact.id().unwrap_or("-").to_string(),
            ];
            if has_priority {
                row.push(
                    pa.artifact
                        .priority()
                        .map(|p| p.to_string())
                        .unwrap_or("-".into()),
                );
            }
            if has_status {
                row.push(pa.artifact.status().unwrap_or("-").to_string());
            }
            row.push(pa.artifact.title().unwrap_or("-").to_string());
            row
        })
        .collect();

    println!("{}", render_table(&headers, rows, format));

    Ok(())
}

/// ark scan next <types> - show active and top queued items across all projects
pub fn run_next(cwd: &Path, type_names: &str, count: usize, format: &OutputFormat) -> Result<()> {
    let projects = discover_projects(cwd)?;
    let artifacts = load_matching_artifacts(&projects, type_names)?;

    let active: Vec<_> = artifacts
        .iter()
        .filter(|pa| pa.artifact.status() == Some("active"))
        .collect();

    let mut queued: Vec<_> = artifacts
        .iter()
        .filter(|pa| {
            let status = pa.artifact.status().unwrap_or("");
            status != "active" && status != "blocked" && status != "done"
        })
        .collect();

    queued.sort_by(|a, b| {
        let pa = a.artifact.priority().unwrap_or(i64::MAX);
        let pb = b.artifact.priority().unwrap_or(i64::MAX);
        pa.cmp(&pb)
    });
    queued.truncate(count);

    if active.is_empty() && queued.is_empty() {
        println!(
            "No active or queued artifacts of type(s) '{}' found across {} projects.",
            type_names,
            projects.len()
        );
        return Ok(());
    }

    if !active.is_empty() {
        println!("Active:");
        let headers = &["project", "id", "pri", "title"];
        let rows: Vec<Vec<String>> = active
            .iter()
            .map(|pa| {
                vec![
                    pa.project.clone(),
                    pa.artifact.id().unwrap_or("-").into(),
                    pa.artifact
                        .priority()
                        .map(|p| p.to_string())
                        .unwrap_or("-".into()),
                    pa.artifact.title().unwrap_or("-").into(),
                ]
            })
            .collect();
        println!("{}", render_table(headers, rows, format));
    }

    if !queued.is_empty() {
        if !active.is_empty() {
            println!();
        }
        println!("Up next:");
        let headers = &["project", "id", "pri", "status", "title"];
        let rows: Vec<Vec<String>> = queued
            .iter()
            .map(|pa| {
                vec![
                    pa.project.clone(),
                    pa.artifact.id().unwrap_or("-").into(),
                    pa.artifact
                        .priority()
                        .map(|p| p.to_string())
                        .unwrap_or("-".into()),
                    pa.artifact.status().unwrap_or("-").into(),
                    pa.artifact.title().unwrap_or("-").into(),
                ]
            })
            .collect();
        println!("{}", render_table(headers, rows, format));
    }

    Ok(())
}

/// ark scan stats - aggregate statistics across all projects
pub fn run_stats(
    cwd: &Path,
    type_filter: Option<&str>,
    group_by: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let projects = discover_projects(cwd)?;

    if projects.is_empty() {
        println!("No ark projects found below current directory.");
        return Ok(());
    }

    if let Some(type_names) = type_filter {
        let artifacts = load_matching_artifacts(&projects, type_names)?;

        if let Some(field) = group_by {
            let mut groups: HashMap<String, usize> = HashMap::new();
            for pa in &artifacts {
                let value = pa.artifact.get_str(field).unwrap_or("(none)").to_string();
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
            let mut counts: HashMap<String, usize> = HashMap::new();
            for pa in &artifacts {
                *counts.entry(pa.project.clone()).or_default() += 1;
            }

            let headers = &["project", "count"];
            let mut rows: Vec<Vec<String>> = counts
                .into_iter()
                .map(|(k, v)| vec![k, v.to_string()])
                .collect();
            rows.sort_by(|a, b| a[0].cmp(&b[0]));

            println!("{}", render_table(headers, rows, format));
        }
    } else {
        // Overview: count per project per type
        let headers = &["project", "type", "count"];
        let mut rows: Vec<Vec<String>> = Vec::new();

        for project in &projects {
            for schema in project.schemas.values() {
                let artifacts = crate::artifact::load_artifacts(&project.root, schema)?;
                rows.push(vec![
                    project.name.clone(),
                    schema.name.clone(),
                    artifacts.len().to_string(),
                ]);
            }
        }
        rows.sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])));

        println!("{}", render_table(headers, rows, format));
    }

    Ok(())
}

/// ark scan search <pattern> - search across all projects
pub fn run_search(
    cwd: &Path,
    pattern: &str,
    type_filter: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let re = regex_lite::Regex::new(pattern)
        .map_err(|e| anyhow::anyhow!("invalid regex pattern: {}", e))?;

    let projects = discover_projects(cwd)?;
    let project_count = projects.len();
    let mut match_rows: Vec<Vec<String>> = Vec::new();

    for project in &projects {
        for schema in project.schemas.values() {
            if let Some(type_names) = type_filter {
                let names: Vec<&str> = type_names.split(',').map(|s| s.trim()).collect();
                if !names.contains(&schema.name.as_str()) {
                    continue;
                }
            }

            let artifacts = crate::artifact::load_artifacts(&project.root, schema)?;
            for artifact in &artifacts {
                let in_body = re.is_match(&artifact.body);
                let in_title = artifact.title().is_some_and(|t| re.is_match(t));
                if in_body || in_title {
                    let match_in = match (in_title, in_body) {
                        (true, true) => "title, body",
                        (true, false) => "title",
                        (false, true) => "body",
                        _ => "",
                    };
                    match_rows.push(vec![
                        project.name.clone(),
                        schema.name.clone(),
                        artifact.id().unwrap_or("-").to_string(),
                        artifact.title().unwrap_or("-").to_string(),
                        match_in.to_string(),
                    ]);
                }
            }
        }
    }

    if match_rows.is_empty() {
        println!(
            "No artifacts matching '{}' found across {} projects.",
            pattern, project_count
        );
        return Ok(());
    }

    let headers = &["project", "type", "id", "title", "match_in"];
    println!("{}", render_table(headers, match_rows, format));

    Ok(())
}

/// ark scan lint - lint all artifacts across all projects
pub fn run_lint(cwd: &Path) -> Result<()> {
    let projects = discover_projects(cwd)?;

    if projects.is_empty() {
        println!("No ark projects found below current directory.");
        return Ok(());
    }

    let mut total_files = 0;
    let mut total_errors = 0;
    let mut project_count = 0;

    for project in &projects {
        for schema in project.schemas.values() {
            let artifacts = crate::artifact::load_artifacts(&project.root, schema)?;
            let json_schema = schema.to_json_schema();
            let validator = match jsonschema::validator_for(&json_schema) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "  error: {}/{} - failed to compile schema: {}",
                        project.name, schema.name, e
                    );
                    total_errors += 1;
                    continue;
                }
            };

            for artifact in &artifacts {
                total_files += 1;
                let data = artifact.frontmatter_as_json();
                for error in validator.iter_errors(&data) {
                    eprintln!(
                        "  error: {}/{} - {} at {}",
                        project.name,
                        artifact
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                        error,
                        error.instance_path
                    );
                    total_errors += 1;
                }
            }
        }
        project_count += 1;
    }

    println!();
    if total_errors == 0 {
        println!(
            "Scan lint passed. {} files across {} projects, no issues found.",
            total_files, project_count
        );
    } else {
        println!(
            "Scan lint complete. {} files across {} projects: {} errors.",
            total_files, project_count, total_errors
        );
    }

    if total_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}
