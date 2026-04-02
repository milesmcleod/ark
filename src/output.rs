use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Cell, ContentArrangement, Table};

#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Pretty,
    Tsv,
    Json,
}

/// Render tabular data in the requested format
pub fn render_table(headers: &[&str], rows: Vec<Vec<String>>, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Pretty => render_pretty(headers, rows),
        OutputFormat::Tsv => render_tsv(headers, rows),
        OutputFormat::Json => render_json(headers, rows),
    }
}

fn render_pretty(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers.iter().map(|h| Cell::new(h.to_uppercase())));

    for row in rows {
        table.add_row(row.iter().map(Cell::new));
    }

    table.to_string()
}

fn render_tsv(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut out = headers.join("\t");
    for row in rows {
        out.push('\n');
        out.push_str(&row.join("\t"));
    }
    out
}

fn render_json(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let maps: Vec<serde_json::Map<String, serde_json::Value>> = rows
        .into_iter()
        .map(|row| {
            headers
                .iter()
                .zip(row)
                .map(|(h, v)| (h.to_string(), serde_json::Value::String(v)))
                .collect()
        })
        .collect();

    serde_json::to_string_pretty(&maps).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsv_output() {
        let headers = &["id", "title"];
        let rows = vec![vec!["BL-001".into(), "Test".into()]];
        let output = render_tsv(headers, rows);
        assert_eq!(output, "id\ttitle\nBL-001\tTest");
    }

    #[test]
    fn test_json_output() {
        let headers = &["id", "title"];
        let rows = vec![vec!["BL-001".into(), "Test".into()]];
        let output = render_json(headers, rows);
        let parsed: Vec<serde_json::Map<String, serde_json::Value>> =
            serde_json::from_str(&output).unwrap();
        assert_eq!(parsed[0]["id"], "BL-001");
        assert_eq!(parsed[0]["title"], "Test");
    }

    #[test]
    fn test_empty_pretty() {
        let headers = &["id", "title"];
        let rows: Vec<Vec<String>> = vec![];
        let output = render_pretty(headers, rows);
        assert!(output.is_empty());
    }
}
