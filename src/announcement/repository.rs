use async_trait::async_trait;
use sqlx::{Pool, Postgres};

pub struct InsertAnnouncementParams {
    pub title: String,
    pub media: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub notes: String,
    pub user_id: i32,
}

#[async_trait]
pub trait AnnouncementRepositoryInterface {
    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error>;
}

pub struct AnnouncementRepository {
    _db: Pool<Postgres>,
}

#[async_trait]
impl AnnouncementRepositoryInterface for AnnouncementRepository {
    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                insert into "announcement" ("title", "media", "start_date", "end_date", "notes", "user_id")
                values ($1, $2, $3, $4, $5, $6)
                returning "id"
            "#,
            params.title,
            params.media,
            params.start_date,
            params.end_date,
            params.notes,
            params.user_id,
        ).fetch_one(&self._db).await?;

        Ok(result.id)
    }
}
