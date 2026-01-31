use std::sync::Arc;

use axum::{Json, extract::State};
use validator::Validate;

use crate::{
    db::UrlDb,
    error::AppError,
    models::{AppState, ShortenRequest, ShortenResponse},
};

pub async fn create_url_handler(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, AppError> {
    if let Err(e) = request.validate() {
        return Err(AppError::ValidationError(e.to_string()));
    }

    let urldb = UrlDb::new(app_state.pool.clone());

    match urldb.create_url(request).await {
        Ok(short_code) => Ok(Json(ShortenResponse {
            short_code: short_code.clone(),
            short_url: format!("http://localhost:3000/{}", short_code),
        })),
        Err(_) => Err(AppError::CodeAlreadyExists),
    }
}
