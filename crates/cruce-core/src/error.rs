use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CruceError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    IoGeneric(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Game '{0}' not found")]
    GameNotFound(String),

    #[error(
        "Game safe_name collision: '{new_name}' conflicts with existing game '{existing_name}'"
    )]
    NameCollision {
        new_name: String,
        existing_name: String,
    },

    #[error("Proton version '{0}' not found")]
    ProtonNotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Operation cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, CruceError>;
