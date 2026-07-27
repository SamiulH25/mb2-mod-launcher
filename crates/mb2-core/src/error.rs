use thiserror::Error;

#[derive(Debug, Error)]
pub enum Mb2Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parse error: {0}")]
    Xml(String),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Load order error: {0}")]
    LoadOrder(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Mb2Error>;
