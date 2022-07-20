use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::database::PaginationResult;

use super::{Device, DeviceDetail, ListDeviceParams};

pub struct InsertDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
    pub is_linked: bool,
}

pub struct UpdateDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
}

pub struct ListDeviceRow {
    pub count: i32,
    pub device_id: i32,
    pub device_name: String,
    pub device_location: String,
    pub device_description: String,
}

#[async_trait]
pub trait DeviceRepositoryInterface {
    async fn find(&self, params: ListDeviceParams)
        -> Result<PaginationResult<Device>, sqlx::Error>;
    async fn find_one(&self, device_id: i32) -> Result<DeviceDetail, sqlx::Error>;
    async fn insert(&self, params: InsertDeviceParams) -> Result<i32, sqlx::Error>;
    async fn update(&self, device_id: i32, params: UpdateDeviceParams) -> Result<(), sqlx::Error>;
    async fn delete(&self, device_id: i32) -> Result<(), sqlx::Error>;
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
    async fn find(
        &self,
        params: ListDeviceParams,
    ) -> Result<PaginationResult<Device>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
                select
                    cast(count("device".*) over () as integer) as "count",
                    "device"."id" as "device_id",
                    "device"."name" as "device_name",
                    concat("building"."name", ', ', "floor"."name") as "device_location",
                    "device"."description" as "device_description" 
                from "device" 
                join "floor" on "floor"."id" = "device"."floor_id"
                join "building" on "building"."id" = "floor"."building_id"
                where 
                    ($3::text is null or "device"."name" ilike concat('%', $3, '%')) and
                    ($4::integer is null or "building"."id" = $4) and
                    ($5::integer is null or "floor"."id" = $5)
                order by "device"."id" desc
                offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.query)
        .bind(params.building_id)
        .bind(params.floor_id)
        .map(|row: PgRow| ListDeviceRow {
            count: row.get("count"),
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_location: row.get("device_location"),
            device_description: row.get("device_description"),
        })
        .fetch_all(&self._db)
        .await?;

        let mut count = 0;
        if result.len() > 0 {
            count = result[0].count;
        }
        let total_pages = (count as f64 / params.limit as f64).ceil() as i32;
        let has_next = (params.page as f64 * params.limit as f64) < 1.0;
        let contents = result
            .iter()
            .map(|item| Device {
                id: item.device_id,
                name: item.device_name.to_string(),
                location: item.device_location.to_string(),
                description: item.device_description.to_string(),
            })
            .collect();

        Ok(PaginationResult::<Device> {
            total_pages,
            has_next,
            count,
            contents,
        })
    }

    async fn find_one(&self, device_id: i32) -> Result<DeviceDetail, sqlx::Error> {
        let result = sqlx::query(
            r#"
            select
                "device"."id" as "id",
                "device"."name" as "name",
                concat("building"."name", ', ', "floor"."name") as "location",
                "device"."description" as "description",
                "device"."created_at" as "created_at",
                "device"."updated_at" as "updated_at"
            from "device"
            join "floor" on "floor"."id" = "device"."floor_id"
            join "building" on "building"."id" = "floor"."building_id"
            where "device"."id" = $1
            "#,
        )
        .bind(device_id)
        .map(|row: PgRow| DeviceDetail {
            id: row.get("id"),
            name: row.get("name"),
            location: row.get("location"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }

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

    async fn update(&self, device_id: i32, params: UpdateDeviceParams) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
            update "device"
            set 
                "name" = $2,
                "description" = $3,
                "floor_id" = $4
            where "id" = $1
            "#,
            device_id,
            params.name,
            params.description,
            params.floor_id,
        )
        .execute(&self._db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn delete(&self, device_id: i32) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
                delete from "device"
                where "id" = $1
            "#,
            device_id,
        )
        .execute(&self._db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}
