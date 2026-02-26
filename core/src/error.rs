use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NuError {
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("unsupported environment: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, NuError>;
