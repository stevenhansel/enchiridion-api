use async_trait::async_trait;
use sqlx::{Pool, Postgres};

pub struct InsertAnnouncementParams {
    pub title: String,
    pub media: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub notes: String,
    pub device_ids: Vec<i32>,
    pub user_id: i32,
}

#[async_trait]
pub trait AnnouncementRepositoryInterface {
    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error>;
}

pub struct AnnouncementRepository {
    _db: Pool<Postgres>,
}

impl AnnouncementRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        AnnouncementRepository { _db }
    }
}

#[async_trait]
impl AnnouncementRepositoryInterface for AnnouncementRepository {
    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                with cte_announcement as (
                    insert into "announcement" ("title", "media", "start_date", "end_date", "notes", "user_id")
                    values ($1, $2, $3, $4, $5, $6)
                    returning "id"
                )
                insert into "device_announcement" ("announcement_id", "device_id")
                values ((select "id" from "cte_announcement"), unnest($7::int4[]))
                returning (select "id" from "cte_announcement")
            "#,
            params.title,
            params.media,
            params.start_date,
            params.end_date,
            params.notes,
            params.user_id,
            &params.device_ids,
        ).fetch_one(&self._db).await?;

        match result.id {
            Some(id) => Ok(id),
            None => Err(sqlx::Error::RowNotFound),
        }
    }
}
