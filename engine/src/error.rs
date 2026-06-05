use thiserror::Error;

#[derive(Debug, Error)]
pub enum CinemaError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("project not found: {0}")]
    ProjectNotFound(String),

    #[error("corrupted project state: {0}")]
    CorruptedState(String),

    #[error("schema migration required: v{from} -> v{to}")]
    MigrationRequired { from: u32, to: u32 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Database(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("recovery failed: {0}")]
    RecoveryFailed(String),
}

pub type Result<T> = std::result::Result<T, CinemaError>;
