use async_trait::async_trait;
use sqlx::{Pool, Postgres, QueryBuilder};

pub struct InsertDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
    pub is_linked: bool,
}

#[async_trait]
pub trait DeviceRepositoryInterface {
    async fn insert(&self, params: InsertDeviceParams) -> Result<i32, sqlx::Error>;
}

pub struct DeviceRepository {
    _db: Pool<Postgres>,
}

impl DeviceRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        DeviceRepository { _db }
    }
}

#[async_trait]
impl DeviceRepositoryInterface for DeviceRepository {
    async fn insert(&self, params: InsertDeviceParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "device" ("name", "description", "floor_id", "is_linked")
            values ($1, $2, $3, $4)
            returning "id"
        "#,
            params.name,
            params.description,
            params.floor_id,
            params.is_linked,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }
}
