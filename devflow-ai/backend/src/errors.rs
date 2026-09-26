use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Artifact validation error: {0}")]
    ArtifactValidation(String),

    #[error("Demo mode: mutation not allowed")]
    DemoModeMutation,

    #[error("Approval required")]
    ApprovalRequired,

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Path traversal detected")]
    PathTraversal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InvalidTransition(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
            AppError::Workspace(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::ArtifactValidation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::DemoModeMutation => (StatusCode::FORBIDDEN, "Mutation not allowed in demo mode".to_string()),
            AppError::ApprovalRequired => (StatusCode::PRECONDITION_REQUIRED, "Approval required".to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error".to_string()),
            AppError::PathTraversal => (StatusCode::BAD_REQUEST, "Path traversal detected".to_string()),
        };

        tracing::error!("API error: {:?}", self);

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type ApiResult<T> = Result<T, AppError>;
