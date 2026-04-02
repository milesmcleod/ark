use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "ark",
    version,
    about = "Human-legible, agent-optimized project artifact management",
    long_about = "A local-first, git-native, schema-enforced CLI for managing structured markdown artifacts.\nThe CLI is the API. Markdown is the storage layer. Schemas make it queryable and enforceable."
)]
pub struct Cli {
    /// Output format
    #[arg(long, short = 'F', global = true, default_value = "pretty")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize ark in the current directory
    Init,

    /// List artifacts of a given type
    List(ListArgs),

    /// Show the highest priority backlog items
    Next(NextArgs),

    /// Show a single artifact by ID
    Show(ShowArgs),

    /// Create a new artifact
    New(NewArgs),

    /// Edit an artifact's frontmatter fields
    Edit(EditArgs),

    /// Validate artifacts against their schemas
    Lint(LintArgs),

    /// Move archived artifacts to their archive directory
    Archive(ArchiveArgs),

    /// Re-number priorities in increments of 10
    Rebalance(RebalanceArgs),

    /// Show valid values for a field on an artifact type
    Fields(FieldsArgs),

    /// List registered artifact types
    Types,

    /// Show artifact statistics
    Stats(StatsArgs),

    /// Search artifact bodies for a pattern
    Search(SearchArgs),

    /// Scan across nested ark projects (recursive discovery)
    Scan(ScanArgs),

    /// Generate shell completions
    Completions(CompletionsArgs),

    /// Show the schema file format reference
    SchemaHelp,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Artifact type to list
    pub artifact_type: String,

    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by project
    #[arg(long)]
    pub project: Option<String>,

    /// Filter by kind (the 'type' field on artifacts)
    #[arg(long)]
    pub kind: Option<String>,

    /// Filter by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Maximum number of items to show
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,
}

#[derive(clap::Args)]
pub struct ShowArgs {
    /// Artifact ID (e.g. BL-001, SPEC-003)
    pub id: String,
}

#[derive(clap::Args)]
pub struct NewArgs {
    /// Artifact type to create
    pub artifact_type: String,

    /// Title for the artifact
    #[arg(long)]
    pub title: String,

    /// Status (uses schema default if not provided)
    #[arg(long)]
    pub status: Option<String>,

    /// Priority (integer)
    #[arg(long)]
    pub priority: Option<i64>,

    /// Project name
    #[arg(long)]
    pub project: Option<String>,

    /// Kind (the 'type' field value)
    #[arg(long)]
    pub kind: Option<String>,

    /// Tags (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// Additional key=value fields
    #[arg(long = "set", value_parser = parse_key_value)]
    pub extra_fields: Option<Vec<(String, String)>>,
}

#[derive(clap::Args)]
pub struct EditArgs {
    /// Artifact ID (e.g. BL-001)
    pub id: String,

    /// Set a field value (can be repeated)
    #[arg(long = "set", value_parser = parse_key_value)]
    pub fields: Vec<(String, String)>,

    /// Set status
    #[arg(long)]
    pub status: Option<String>,

    /// Set priority
    #[arg(long)]
    pub priority: Option<i64>,

    /// Set title
    #[arg(long)]
    pub title: Option<String>,

    /// Set project
    #[arg(long)]
    pub project: Option<String>,

    /// Set kind (the 'type' field value)
    #[arg(long)]
    pub kind: Option<String>,
}

#[derive(clap::Args)]
pub struct LintArgs {
    /// Artifact type or specific ID to lint (lints all if omitted)
    pub target: Option<String>,

    /// Attempt to auto-fix issues
    #[arg(long)]
    pub fix: bool,
}

#[derive(clap::Args)]
pub struct ArchiveArgs {
    /// Artifact type to archive
    pub artifact_type: String,
}

#[derive(clap::Args)]
pub struct RebalanceArgs {
    /// Artifact type to rebalance
    pub artifact_type: String,

    /// Gap between priority values (default: 10)
    #[arg(long, default_value = "10")]
    pub gap: i64,
}

#[derive(clap::Args)]
pub struct FieldsArgs {
    /// Artifact type
    pub artifact_type: String,

    /// Specific field to show values for (shows all fields if omitted)
    pub field: Option<String>,
}

#[derive(clap::Args)]
pub struct NextArgs {
    /// Artifact type
    pub artifact_type: String,

    /// Number of items to show (default: 5)
    #[arg(default_value = "5")]
    pub count: usize,
}

#[derive(clap::Args)]
pub struct StatsArgs {
    /// Artifact type (shows all types if omitted)
    pub artifact_type: Option<String>,

    /// Group by this field
    #[arg(long)]
    pub by: Option<String>,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    /// Search pattern (regex supported)
    pub pattern: String,

    /// Restrict to artifact type
    #[arg(long)]
    pub artifact_type: Option<String>,
}

#[derive(clap::Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: clap_complete::Shell,
}

#[derive(clap::Args)]
pub struct ScanArgs {
    #[command(subcommand)]
    pub command: ScanCommand,
}

#[derive(Subcommand)]
pub enum ScanCommand {
    /// List all artifact types across nested projects
    Types,

    /// List artifacts of matching types across all projects
    List(ScanListArgs),

    /// Show active and top queued items across all projects
    Next(ScanNextArgs),

    /// Show aggregate statistics across all projects
    Stats(ScanStatsArgs),

    /// Search artifact bodies across all projects
    Search(ScanSearchArgs),

    /// Lint all artifacts across all projects
    Lint,
}

#[derive(clap::Args)]
pub struct ScanListArgs {
    /// Artifact type(s) to match, comma-separated (e.g. "task,story,ticket")
    pub types: String,

    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by project name
    #[arg(long)]
    pub project: Option<String>,

    /// Maximum number of items
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,
}

#[derive(clap::Args)]
pub struct ScanNextArgs {
    /// Artifact type(s) to match, comma-separated (e.g. "task,story,ticket")
    pub types: String,

    /// Number of queued items to show per project (default: 3)
    #[arg(long, short = 'n', default_value = "3")]
    pub count: usize,
}

#[derive(clap::Args)]
pub struct ScanStatsArgs {
    /// Artifact type(s) to filter (shows all if omitted)
    pub types: Option<String>,

    /// Group by this field
    #[arg(long)]
    pub by: Option<String>,
}

#[derive(clap::Args)]
pub struct ScanSearchArgs {
    /// Search pattern (regex)
    pub pattern: String,

    /// Restrict to artifact type(s), comma-separated
    #[arg(long)]
    pub types: Option<String>,
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no '=' found in '{s}'"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
