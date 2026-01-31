mod error;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{delete, get, post},
};
use chrono::{Duration, prelude::*};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::{env, sync::Arc};
use validator::Validate;

use error::AppError;

#[derive(Clone, Serialize, sqlx::FromRow)]
struct UrlData {
    short_code: String,
    original_url: String,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    visits: i64,
}

struct AppState {
    pool: SqlitePool,
}

#[derive(Deserialize, Validate)]
struct ShortenRequest {
    #[validate(url(message = "Not a valid URL"))]
    url: String,

    #[validate(length(min = 6))]
    code: Option<String>,

    ttl: Option<i64>,
}

#[derive(Serialize)]
struct ShortenResponse {
    short_code: String,
    short_url: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to sqlite db");

    let state = Arc::new(AppState { pool });

    let app = Router::new()
        .route("/shorten", post(shorten_url))
        .route("/list", get(list_urls))
        .route("/stats/:code", get(stats_handler))
        .route("/delete/:code", delete(delete_url_handler))
        .route("/:code", get(redirect_url))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn shorten_url(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::ValidationError(e.to_string()));
    }

    let code = match payload.code {
        Some(code) => code,
        _ => nanoid::nanoid!(6),
    };

    let expires_at = payload.ttl.map(|x| Utc::now() + Duration::seconds(x));

    let created_at = Utc::now();

    sqlx::query!(
        "insert into urls (short_code, original_url, visits, created_at, expires_at) values ($1, $2, $3, $4, $5)",
        code,
        payload.url,
        0,
        created_at,
        expires_at,
    ).fetch_one(&app_state.pool).await?;

    let response = ShortenResponse {
        short_code: code.clone(),
        short_url: format!("http://localhost:3000/{}", code),
    };

    Ok(Json(response))
}

async fn redirect_url(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Redirect, AppError> {
    match get_url(&app_state.pool, path.clone()).await {
        Some(row) => {
            if let Some(expires_at) = row.expires_at
                && expires_at < Utc::now()
            {
                let _ = delete_url(&app_state.pool, path).await;
                return Err(AppError::UrlNotFound);
            }

            let _ = sqlx::query!(
                "update urls set visits = visits + 1 where short_code = $1",
                path
            )
            .execute(&app_state.pool)
            .await;

            Ok(Redirect::permanent(&row.original_url))
        }
        None => Err(AppError::UrlNotFound),
    }
}

async fn list_urls(State(app_state): State<Arc<AppState>>) -> Result<Json<Vec<UrlData>>, AppError> {
    let records: Vec<UrlData> = sqlx::query_as("select * from urls")
        .fetch_all(&app_state.pool)
        .await?;

    Ok(Json(records))
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
