use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error {0}")]
    Internal(String),

    #[error("Configuration error {0}")]
    Config(#[from] envy::Error),

    #[error("Search error {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),

    #[error("PFF error {0}")]
    Pff(#[from] pff::error::Error),

    #[error("Invalid message ID")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Receiving channel for message body in web server thread has closed")]
    BodyChannelClosed,

    #[error("Pff manager channel already closed")]
    PffChannelClosed,

    #[error("Timed out waiting for message body to be located")]
    BodyTimeout,

    #[error("Message body not found")]
    BodyNotFound,

    #[error("Message body cache loop has closed")]
    BodyCacheLoopClosed,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
