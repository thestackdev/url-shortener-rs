use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::{error::AppError, models::ShortenRequest};

pub struct UrlDb {
    pool: SqlitePool,
}

impl UrlDb {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_url(&self, request: ShortenRequest) -> Result<String, AppError> {
        let short_code = request.code.unwrap_or(nanoid::nanoid!(6));
        let created_at = Utc::now();
        let expires_at = request.ttl.map(|e| created_at + Duration::seconds(e));

        let _ = sqlx::query!(
            "insert into urls (short_code, original_url, created_at, expires_at, visits) values ($1, $2, $3, $4, $5)",
            short_code,
            request.url,
            created_at,
            expires_at,
            0
        ).execute(&self.pool).await?;

        Ok(short_code)
    }
}
