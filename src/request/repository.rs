use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use super::RequestActionType;

pub struct InsertRequestParams {
    pub action: RequestActionType,
    pub description: String,
    pub announcement_id: i32,
    pub user_id: i32,
}

#[async_trait]
pub trait RequestRepositoryInterface {
    async fn insert(&self, params: InsertRequestParams) -> Result<i32, sqlx::Error>;
}

pub struct RequestRepository {
    _db: Pool<Postgres>,
}

impl RequestRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        RequestRepository { _db }
    }
}

#[async_trait]
impl RequestRepositoryInterface for RequestRepository {
    async fn insert(&self, params: InsertRequestParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "request" ("action", "description", "announcement_id", "user_id")
            values ($1, $2, $3, $4)
            returning "id"
            "#,
            params.action as _,
            params.description,
            params.announcement_id,
            params.user_id,
        )
        .fetch_one(&self._db)
        .await?;

        return Ok(result.id);
    }
}
