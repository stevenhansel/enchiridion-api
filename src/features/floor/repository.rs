use std::collections::{btree_map::Entry, BTreeMap};

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
    device_id: Option<i32>,
    device_name: Option<String>,
    device_description: Option<String>,
    // device_total_announcements: i32,
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
                "building"."color" as "building_color",
                "device"."id" as "device_id",
                "device"."name" as "device_name",
                "device"."description" as "device_description"
            from "floor"
            join "building" on "building"."id" = "floor"."building_id"
            left join "device" on "device"."floor_id" = "floor"."id"
            left join lateral (
                select count(*) from "floor"
                where
                    (
                        $3::text is null or 
                        "floor"."id" = cast(
                            (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                        ) or
                        "floor"."name" ilike concat('%', $3, '%')
                    ) and
                    ($4::integer is null or "building"."id" = $4)
            ) "result" on true
            where 
                (
                    $3::text is null or 
                    "floor"."id" = cast(
                        (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                    ) or
                    "floor"."name" ilike concat('%', $3, '%')
                ) and
                ($4::integer is null or "building"."id" = $4)
            group by "floor"."id", "building"."id", "device"."id", "result"."count"
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
            building_color: row.get("building_color"),
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_description: row.get("device_description"),
        })
        .fetch_all(&self._db)
        .await?;

        let mut count = 0;
        if result.len() > 0 {
            count = result[0].count;
        }

        let total_pages = (count as f64 / params.limit as f64).ceil() as i32;
        let has_next = ((params.page as f64 * params.limit as f64) / count as f64) < 1.0;

        let mut device_map: BTreeMap<i32, DeviceFloorContent> = BTreeMap::new();
        for row in &result {
            if let Some(device_id) = row.device_id {
                if let Entry::Vacant(v) = device_map.entry(device_id) {
                    v.insert(DeviceFloorContent {
                        id: device_id,
                        name: row.device_name.clone().unwrap(),
                        description: row.device_description.clone().unwrap(),
                    });
                }
            }
        }

        let mut floor_map: BTreeMap<i32, Floor> = BTreeMap::new();
        for row in &result {
            match floor_map.entry(row.floor_id) {
                Entry::Vacant(v) => {
                    let mut devices: Vec<DeviceFloorContent> = vec![];
                    if let Some(device_id) = row.device_id {
                        let device = device_map.get(&device_id).unwrap();
                        devices.push(device.clone());
                    }

                    v.insert(Floor {
                        id: row.floor_id,
                        name: row.floor_name.clone(),
                        building: BuildingFloorContent {
                            id: row.building_id,
                            name: row.building_name.clone(),
                            color: row.building_color.clone(),
                        },
                        devices,
                    });
                }
                Entry::Occupied(o) => {
                    if let Some(device_id) = row.device_id {
                        let floor = o.into_mut();
                        let device = device_map.get(&device_id).unwrap();

                        floor.devices.push(device.clone());
                    }
                }
            }
        }

        let contents = floor_map
            .into_iter()
            .rev()
            .map(|(_, v)| v)
            .collect();

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
                where "id" = $1
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
                delete from "floor"
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
