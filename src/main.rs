mod db;
mod error;
mod handlers;
mod models;

use models::{AppState, UrlData};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use dotenv::dotenv;
use serde_json::json;
use sqlx::SqlitePool;
use std::{env, sync::Arc};

use error::AppError;

use handlers::create_url_handler;

use crate::handlers::{handle_url_redirect, list_urls_handler};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to sqlite db");

    let state = Arc::new(AppState { pool });

    let app = Router::new()
        .route("/shorten", post(create_url_handler))
        .route("/list", get(list_urls_handler))
        .route("/stats/:code", get(stats_handler))
        .route("/delete/:code", delete(delete_url_handler))
        .route("/:code", get(handle_url_redirect))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn delete_url_handler(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = delete_url(&app_state.pool, path).await;

    if deleted {
        Ok(Json(json!({
            "success": true,
            "message": "Url deleted successfully!"
        })))
    } else {
        Err(AppError::UrlNotFound)
    }
}

async fn stats_handler(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<UrlData>, AppError> {
    match get_url(&app_state.pool, path).await {
        Some(data) => Ok(Json(data)),
        None => Err(AppError::UrlNotFound),
    }
}

async fn get_url(pool: &SqlitePool, short_code: String) -> Option<UrlData> {
    let response: Result<UrlData, _> = sqlx::query_as("select * from urls where short_code = $1")
        .bind(short_code)
        .fetch_one(pool)
        .await;

    response.ok()
}

async fn delete_url(pool: &SqlitePool, short_code: String) -> bool {
    let response = sqlx::query!("delete from urls where short_code = $1", short_code)
        .execute(pool)
        .await;

    match response {
        Ok(result) => result.rows_affected() >= 1,
        Err(_) => false,
    }
}
