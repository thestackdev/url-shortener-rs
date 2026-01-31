mod db;
mod error;
mod handlers;
mod models;

use models::AppState;

use axum::{
    Router,
    routing::{delete, get, post},
};
use dotenv::dotenv;
use sqlx::SqlitePool;
use std::{env, sync::Arc};

use handlers::{
    create_url_handler, delete_url_handler, handle_url_redirect, list_urls_handler,
    url_stats_handler,
};

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
        .route("/stats/:code", get(url_stats_handler))
        .route("/delete/:code", delete(delete_url_handler))
        .route("/:code", get(handle_url_redirect))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
