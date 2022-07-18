use async_trait::async_trait;
use sqlx::{Postgres, Pool};

pub struct RawFloorResult {
    floor_count: i32,
    floor_id: i32,
    floor_name: String,
    building_id: i32,
    building_name: String,
    building_color: String,
    device_id: i32,
    device_name: String,
    device_description: String,
    // device_total_announcements: i32,
}

pub struct FindFloorParams {
    page: i32,
    limit: i32,
    query: Option<String>,
    building_id: Option<bool>,
    with_devices: Option<bool>,
}

pub struct InsertFloorParams {
    pub name: String,
    pub building_id: i32,
}

pub struct UpdateFloorParams {
    pub name: String,
    pub building_id: i32,
}

#[async_trait]
pub trait FloorRepositoryInterface {
    // async fn find(&self, params: FindFloorParams) -> Result<Vec<Floor>, sqlx::Error>;
    // async fn find_one(&self, floor_id: i32) -> Result<Floor, sqlx::Error>;
    async fn insert(&self, params: InsertFloorParams) -> Result<i32, sqlx::Error>;
    // async fn update(&self, params: UpdateFloorParams) -> Result<(), sqlx::Error>;
    // async fn delete(&self, floor_id: i32) -> Result<(), sqlx::Error>;
}

pub struct FloorRepository {
    _db: Pool<Postgres>,
}

impl FloorRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        FloorRepository { _db }
    }
}

#[async_trait]
impl FloorRepositoryInterface for FloorRepository {
    // async fn find(&self, params: FindFloorParams) -> Result<Vec<Floor>, sqlx::Error> {
    //     let result = sqlx::query_as!(
    //     RawFloorResult,
    //     r#"
    //     select
    //         "floor"."id" as "floor_id",
    //         "floor"."name" as "floor_name",
    //         "building"."id" as "building_id",
    //         "building"."name" as "building_name",
    //         "building"."color" as "building_color",
    //         "device"."id" as "device_id",
    //         "device"."name" as "device_name",
    //         "device"."description" as "device_description"
    //     from "floor"
    //     join "building" on "building"."id" = "floor"."building_id"
    //     join "device" on "device"."floor_id" = "floor"."id"
    //     "#
    //     )
    //     .fetch_all(&self._db)
    //     .await?;
    // }

    // async fn find_one(&self, floor_id: i32) -> Result<Floor, sqlx::Error> {}

    async fn insert(&self, params: InsertFloorParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                insert into "floor" ("name", "building_id")
                values ($1, $2)
                returning "id"
            "#,
            params.name,
            params.building_id,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }

    // async fn update(&self, params: UpdateFloorParams) -> Result<(), sqlx::Error> {}

    // async fn delete(&self, floor_id: i32) -> Result<(), sqlx::Error> {}
}
