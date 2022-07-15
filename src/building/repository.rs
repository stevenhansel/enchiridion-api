use async_trait::async_trait;
use shaku::{Component, Interface};
use sqlx::{Pool, Postgres};

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
pub trait BuildingRepositoryInterface: Interface {
    async fn create(&self, params: InsertBuildingParams) -> Result<i32, sqlx::Error>;
    async fn update(&self, params: UpdateBuildingParams) -> Result<i32, sqlx::Error>;
    async fn delete_by_id(&self, id: i32) -> Result<i32, sqlx::Error>;
}

#[derive(Component)]
#[shaku(interface = BuildingRepositoryInterface)]
pub struct BuildingRepository {
    _db: Pool<Postgres>,
}

#[async_trait]
impl BuildingRepositoryInterface for BuildingRepository {
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
            set name=$2, color=$3
            where id=$1
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
            delete from "building"
            where id = $1
            returning id
            "#,
            id
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }
}
