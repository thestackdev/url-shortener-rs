use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{ShortenRequest, UrlData},
};

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

    pub async fn get_url(&self, short_code: String) -> Result<UrlData, AppError> {
        let response: UrlData = sqlx::query_as("select * from urls where short_code = $1")
            .bind(short_code)
            .fetch_one(&self.pool)
            .await?;

        Ok(response)
    }

    pub async fn list_urls(&self) -> Result<Vec<UrlData>, AppError> {
        let response = sqlx::query_as("select * from urls")
            .fetch_all(&self.pool)
            .await?;

        Ok(response)
    }

    pub async fn delete_url(&self, path: String) -> Result<bool, AppError> {
        let response = sqlx::query!("delete from urls where short_code = $1", path)
            .execute(&self.pool)
            .await?;

        Ok(response.rows_affected() >= 1)
    }
}
