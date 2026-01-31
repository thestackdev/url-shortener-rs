use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::Redirect,
};
use chrono::Utc;
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

    let url_db = UrlDb::new(app_state.pool.clone());

    match url_db.create_url(request).await {
        Ok(short_code) => Ok(Json(ShortenResponse {
            short_code: short_code.clone(),
            short_url: format!("http://localhost:3000/{}", short_code),
        })),
        Err(_) => Err(AppError::CodeAlreadyExists),
    }
}

pub async fn handle_url_redirect(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Redirect, AppError> {
    let url_db = UrlDb::new(app_state.pool.clone());

    let url = url_db.get_url(path).await;

    if let Some(data) = url {
        if let Some(expires_at) = data.expires_at
            && expires_at < Utc::now()
        {
            return Err(AppError::UrlNotFound);
        }

        Ok(Redirect::permanent(&data.original_url))
    } else {
        Err(AppError::UrlNotFound)
    }
}
