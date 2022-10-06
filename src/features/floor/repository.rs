use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::database::PaginationResult;

use super::{BuildingFloorContent, DeviceFloorContent, Floor};

pub struct RawFloorRow {
    count: i32,
    floor_id: i32,
    floor_name: String,
    building_id: i32,
    building_name: String,
    building_color: String,
}

pub struct RawDeviceRow {
    device_id: i32,
    device_name: String,
    device_description: String,
    device_floor_id: i32,
    device_active_announcements: i32,
}

pub struct FindFloorParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub building_id: Option<i32>,
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
    async fn find(&self, params: FindFloorParams) -> Result<PaginationResult<Floor>, sqlx::Error>;
    async fn insert(&self, params: InsertFloorParams) -> Result<i32, sqlx::Error>;
    async fn update(&self, floor_id: i32, params: UpdateFloorParams) -> Result<(), sqlx::Error>;
    async fn delete(&self, floor_id: i32) -> Result<(), sqlx::Error>;
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
    async fn find(&self, params: FindFloorParams) -> Result<PaginationResult<Floor>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
            select
                cast("result"."count" as integer) as "count",
                "floor"."id" as "floor_id",
                "floor"."name" as "floor_name",
                "building"."id" as "building_id",
                "building"."name" as "building_name",
                "building"."color" as "building_color"
            from "floor"
            join "building" on "building"."id" = "floor"."building_id"
            left join lateral (
                select count(*) from "floor"
                join "building" on "building"."id" = "floor"."building_id"
                where
                    (
                        $3::text is null or 
                        "floor"."id" = cast(
                            (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                        ) or
                        "floor"."name" ilike concat('%', $3, '%')
                    ) and
                    ($4::integer is null or "building"."id" = $4) and
                    "floor"."deleted_at" is null
            ) "result" on true
            where 
                (
                    $3::text is null or 
                    "floor"."id" = cast(
                        (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                    ) or
                    "floor"."name" ilike concat('%', $3, '%')
                ) and
                ($4::integer is null or "building"."id" = $4) and
                "floor"."deleted_at" is null
            group by "floor"."id", "building"."id", "result"."count"
            order by "floor"."id" desc
            offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.query.clone())
        .bind(params.building_id)
        .map(|row: PgRow| RawFloorRow {
            count: row.get("count"),
            floor_id: row.get("floor_id"),
            floor_name: row.get("floor_name"),
            building_id: row.get("building_id"),
            building_name: row.get("building_name"),
            building_color: row.get("building_color")
        })
        .fetch_all(&self._db)
        .await?;

        let mut count = 0;
        if result.len() > 0 {
            count = result[0].count;
        }

        let total_pages = (count as f64 / params.limit as f64).ceil() as i32;
        let has_next = ((params.page as f64 * params.limit as f64) / count as f64) < 1.0;

        let floor_ids: Vec<i32> = result.iter().map(|row| row.floor_id).collect();

        let device_result = sqlx::query(
            r#"
            select 
                "device"."id" as "device_id",
                "device"."name" as "device_name",
                "device"."description" as "device_description",
                "device"."floor_id" as "device_floor_id",
                cast("result"."count" as integer) as "device_active_announcements"
            from "device"
            left join lateral (
                select count(*) as "count" from "device_announcement"
                join "announcement" on "announcement"."id" = "device_announcement"."announcement_id"
                where 
                    "device_announcement"."device_id" = "device"."id" and
                    "announcement"."status" = 'active'
            ) "result" on true
            where "floor_id" = any($1)
            "#,
        )
        .bind(&floor_ids)
        .map(|row: PgRow| RawDeviceRow {
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_description: row.get("device_description"),
            device_floor_id: row.get("device_floor_id"),
            device_active_announcements: row.get("device_active_announcements"),
        })
        .fetch_all(&self._db)
        .await?;

        let mut contents: Vec<Floor> = Vec::new();
        for row in result {
            let devices: Vec<DeviceFloorContent> = device_result
                .iter()
                .filter(|device_row| device_row.device_floor_id == row.floor_id)
                .map(|device_row| DeviceFloorContent {
                    id: device_row.device_id,
                    name: device_row.device_name.to_string(),
                    description: device_row.device_description.to_string(),
                    total_announcements: device_row.device_active_announcements,
                })
                .collect();

            contents.push(Floor {
                id: row.floor_id,
                name: row.floor_name,
                building: BuildingFloorContent {
                    id: row.building_id,
                    name: row.building_name,
                    color: row.building_color,
                },
                devices,
            })
        }

        Ok(PaginationResult {
            count,
            total_pages,
            has_next,
            contents,
        })
    }

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

    async fn update(&self, floor_id: i32, params: UpdateFloorParams) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
                update "floor"
                set
                    "name" = $2,
                    "building_id" = $3
                where "id" = $1 and "deleted_at" is null
            "#,
            floor_id,
            params.name,
            params.building_id,
        )
        .execute(&self._db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn delete(&self, floor_id: i32) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
                update "floor"
                set "deleted_at" = now()
                where "id" = $1
            "#,
            floor_id,
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
