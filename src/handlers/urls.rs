use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::Redirect,
};
use chrono::Utc;
use serde_json::{Value, json};
use validator::Validate;

use crate::{
    db::UrlDb,
    error::AppError,
    models::{AppState, ShortenRequest, ShortenResponse, UrlData},
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
        Err(e) => Err(e),
    }
}

pub async fn handle_url_redirect(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Redirect, AppError> {
    let url_db = UrlDb::new(app_state.pool.clone());

    let url = url_db.get_url(path).await;

    match url {
        Ok(data) => {
            if let Some(expires_at) = data.expires_at
                && expires_at < Utc::now()
            {
                return Err(AppError::UrlNotFound);
            }

            Ok(Redirect::permanent(&data.original_url))
        }
        Err(e) => Err(e),
    }
}

pub async fn list_urls_handler(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<UrlData>>, AppError> {
    let url_db = UrlDb::new(app_state.pool.clone());

    match url_db.list_urls().await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err(e),
    }
}

pub async fn url_stats_handler(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<UrlData>, AppError> {
    let url_db = UrlDb::new(app_state.pool.clone());

    match url_db.get_url(path).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err(e),
    }
}

pub async fn delete_url_handler(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<Value>, AppError> {
    let url_db = UrlDb::new(app_state.pool.clone());

    match url_db.delete_url(path).await {
        Ok(data) => {
            if data {
                Ok(Json(json!({
                    "message": "Url successfully deleted"
                })))
            } else {
                Err(AppError::UrlNotFound)
            }
        }
        Err(e) => Err(e),
    }
}
