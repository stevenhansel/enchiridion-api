use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use super::domain::MediaType;

pub struct InsertMediaParams {
    pub path: String,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
}

#[async_trait]
pub trait MediaRepositoryInterface: Send + Sync + 'static {
    async fn insert(&self, params: InsertMediaParams) -> Result<i32, sqlx::Error>;
}

pub struct MediaRepository {
    _db: Pool<Postgres>,
}

impl MediaRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        MediaRepository { _db }
    }
}

#[async_trait]
impl MediaRepositoryInterface for MediaRepository {
    async fn insert(&self, params: InsertMediaParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query(
            r#"
                insert into "media" ("path", "media_type", "media_duration")
                values ($1, $2, $3)
                returning "id"
            "#,
        )
        .bind(params.path)
        .bind(params.media_type)
        .bind(params.media_duration)
        .map(|row: PgRow| row.get("id"))
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }
}
