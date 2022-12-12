use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use super::definition::LivestreamMessagePayload;

#[async_trait]
pub trait LivestreamRepositoryInterface: Send + Sync + 'static {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), sqlx::Error>;
}

pub struct LivestreamRepository {
    _db: Pool<Postgres>,
}

impl LivestreamRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        LivestreamRepository { _db }
    }
}

#[async_trait]
impl LivestreamRepositoryInterface for LivestreamRepository {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
                insert into "device_livestream"
                ("time", "device_id", "num_of_faces")
                values ($1, $2, $3)
            "#,
        )
        .bind(message.timestamp)
        .bind(message.device_id)
        .bind(message.num_of_faces)
        .execute(&self._db)
        .await?;

        Ok(())
    }
}
