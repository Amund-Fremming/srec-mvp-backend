use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::adapters::{db::adapter::DbError, llm::adapter::LlmError, tmdb::adapter::TmdbError};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("TMDB error: {0}")]
    Tmdb(#[from] TmdbError),

    #[error("Database error: {0}")]
    Db(DbError),

    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Tmdb(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Llm(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ValidationError(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<DbError> for AppError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::UsernameConflict(u) => {
                AppError::Conflict(format!("username '{u}' already exists"))
            }
            DbError::NotFound => AppError::NotFound,
            _ => AppError::Db(e),
        }
    }
}
