use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use validator::Validate;

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct UrlData {
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub visits: i64,
}

pub struct AppState {
    pub pool: SqlitePool,
}

#[derive(Deserialize, Validate)]
pub struct ShortenRequest {
    #[validate(url(message = "Not a valid URL"))]
    pub url: String,

    #[validate(length(min = 6))]
    pub code: Option<String>,

    pub ttl: Option<i64>,
}

#[derive(Serialize)]
pub struct ShortenResponse {
    pub short_code: String,
    pub short_url: String,
}
