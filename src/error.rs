use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub enum AppError {
    CodeAlreadyExists,
    UrlNotFound,
    ValidationError(String),
    DatabaseError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::CodeAlreadyExists => (
                StatusCode::CONFLICT,
                json!({
                    "error": "The short_code already taken"
                }),
            ),
            AppError::ValidationError(error) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": format!("Validation Error: {}", error)
                }),
            ),
            AppError::UrlNotFound => (
                StatusCode::NOT_FOUND,
                json!({
                    "error": "Url expired or does not exists"
                }),
            ),
            AppError::DatabaseError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": format!("Database Error: {}", error) }),
            ),
        };

        (status, Json(message)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::UrlNotFound,
            sqlx::Error::Database(err) => {
                let err = err.to_string();

                if err.contains("UNIQUE constraint failed: urls.short_code") {
                    AppError::CodeAlreadyExists
                } else {
                    AppError::DatabaseError(err.to_string())
                }
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}
