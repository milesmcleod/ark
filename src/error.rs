use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ArkError {
    #[error("not an ark project (no .ark/ directory found). Run `ark init` to set up.")]
    NotInitialized,

    #[error("ark is already initialized in this directory")]
    AlreadyInitialized,

    #[error("no schemas defined in .ark/schemas/. Create schema files to define artifact types.")]
    NoSchemas,

    #[error("unknown artifact type: {0}. Run `ark types` to see available types.")]
    UnknownType(String),

    #[error(
        "unknown field '{field}' on artifact type '{artifact_type}'. Run `ark fields {artifact_type}` to see available fields."
    )]
    UnknownField {
        artifact_type: String,
        field: String,
    },

    #[error("artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("schema error in {}: {message}", path.display())]
    SchemaError { path: PathBuf, message: String },
}
