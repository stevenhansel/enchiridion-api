use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use super::domain::Building;

pub struct InsertBuildingParams {
    pub name: String,
    pub color: String,
}

pub struct UpdateBuildingParams {
    pub id: i32,
    pub name: String,
    pub color: String,
}

#[async_trait]
pub trait BuildingRepositoryInterface {
    async fn find_buildings(&self) -> Result<Vec<Building>, sqlx::Error>;
    async fn create(&self, params: InsertBuildingParams) -> Result<i32, sqlx::Error>;
    async fn update(&self, params: UpdateBuildingParams) -> Result<i32, sqlx::Error>;
    async fn delete_by_id(&self, id: i32) -> Result<i32, sqlx::Error>;
}

pub struct BuildingRepository {
    _db: Pool<Postgres>,
}

impl BuildingRepository {
    pub fn new(_db: Pool<Postgres>) -> BuildingRepository {
        BuildingRepository { _db }
    }
}

#[async_trait]
impl BuildingRepositoryInterface for BuildingRepository {
    async fn find_buildings(&self) -> Result<Vec<Building>, sqlx::Error> {
        let result = sqlx::query_as!(
            Building,
            r#"
            select "id", "name", "color"
            from "building"
            where "deleted_at" is null
            order by "id" desc
            "#,
        )
        .fetch_all(&self._db)
        .await?;

        Ok(result)
    }

    async fn create(&self, params: InsertBuildingParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "building" (name, color)
            values($1, $2)
            returning id
            "#,
            params.name,
            params.color,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }

    async fn update(&self, params: UpdateBuildingParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            update "building"
            set name = $2, color = $3
            where id = $1 and "deleted_at" is null
            returning id
            "#,
            params.id,
            params.name,
            params.color,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }

    async fn delete_by_id(&self, id: i32) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            update "building"
            set "deleted_at" = now()
            where "id" = $1
            returning "id"
            "#,
            id
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }
}
