use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ArkError {
    NotInitialized,
    AlreadyInitialized,
    NoSchemas,
    UnknownType(String),
    UnknownField {
        artifact_type: String,
        field: String,
    },
    ArtifactNotFound(String),
    SchemaError {
        path: PathBuf,
        message: String,
    },
}

impl fmt::Display for ArkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(
                f,
                "not an ark project (no .ark/ directory found). Run `ark init` to set up."
            ),
            Self::AlreadyInitialized => write!(f, "ark is already initialized in this directory"),
            Self::NoSchemas => write!(
                f,
                "no schemas defined in .ark/schemas/. Create schema files to define artifact types."
            ),
            Self::UnknownType(t) => {
                write!(
                    f,
                    "unknown artifact type: {t}. Run `ark types` to see available types."
                )
            }
            Self::UnknownField {
                artifact_type,
                field,
            } => write!(
                f,
                "unknown field '{field}' on artifact type '{artifact_type}'. Run `ark fields {artifact_type}` to see available fields."
            ),
            Self::ArtifactNotFound(id) => write!(f, "artifact not found: {id}"),
            Self::SchemaError { path, message } => {
                write!(f, "schema error in {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ArkError {}
