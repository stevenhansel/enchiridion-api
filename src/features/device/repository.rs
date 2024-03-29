use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use deadpool_redis::redis::{cmd, RedisError};
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::{
    database::PaginationResult,
    features::device_status::definition::{
        DeviceStatus, DEVICE_STATUS_REDIS_KEY, TIMEOUT_DURATION_SECS,
    },
};

use super::{
    CountDeviceParams, Device, DeviceAuthCache, DeviceDetail, DeviceDetailLocation,
    ListDeviceParams,
};

pub struct InsertDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub secret_access_key_salt: String,
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
    pub device_active_announcements: i32,
}

#[async_trait]
pub trait DeviceRepositoryInterface {
    async fn count(&self, params: CountDeviceParams) -> Result<i32, sqlx::Error>;
    async fn find(&self, params: ListDeviceParams)
        -> Result<PaginationResult<Device>, sqlx::Error>;
    async fn find_one(&self, device_id: i32) -> Result<DeviceDetail, sqlx::Error>;
    async fn find_one_by_access_key_id(
        &self,
        access_key_id: String,
    ) -> Result<DeviceDetail, sqlx::Error>;
    async fn insert(&self, params: InsertDeviceParams) -> Result<i32, sqlx::Error>;
    async fn update(&self, device_id: i32, params: UpdateDeviceParams) -> Result<(), sqlx::Error>;
    async fn delete(&self, device_id: i32) -> Result<(), sqlx::Error>;
    async fn exists(&self, device_ids: &Vec<i32>) -> Result<bool, sqlx::Error>;
    async fn find_announcement_ids_in_device(
        &self,
        device_id: i32,
    ) -> Result<Vec<i32>, sqlx::Error>;
    async fn update_device_link(&self, device_id: i32, link: bool) -> Result<(), sqlx::Error>;
    async fn update_camera_enabled(
        &self,
        device_id: i32,
        camera_enabled: bool,
    ) -> Result<(), sqlx::Error>;
    async fn get_auth_cache(&self, access_key_id: String) -> Result<DeviceAuthCache, RedisError>;
    async fn set_auth_cache(
        &self,
        access_key_id: String,
        cache: DeviceAuthCache,
    ) -> Result<(), RedisError>;
    async fn del_auth_cache(&self, access_key_id: String) -> Result<(), RedisError>;
    async fn get_device_status(&self, device_id: i32) -> Result<DeviceStatus, RedisError>;
    async fn set_device_status(
        &self,
        device_id: i32,
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), RedisError>;
}

pub struct DeviceRepository {
    _db: Pool<Postgres>,
    _redis: deadpool_redis::Pool,
}

impl DeviceRepository {
    pub fn new(_db: Pool<Postgres>, _redis: deadpool_redis::Pool) -> Self {
        DeviceRepository { _db, _redis }
    }

    pub fn device_auth_key_builder(&self, access_key_id: String) -> String {
        format!("device/pub-key-{}", access_key_id)
    }
}

#[async_trait]
impl DeviceRepositoryInterface for DeviceRepository {
    async fn count(&self, params: CountDeviceParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query(
        r#"
            select
                cast(count("device".*) as integer) as "count"
            from "device"
            join "floor" on "floor"."id" = "device"."floor_id"
            join "building" on "building"."id" = "floor"."building_id"
            where 
                (
                    $1::text is null or 
                    "device"."id" = cast(
                        (coalesce(nullif(regexp_replace($1, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                    ) or
                    "device"."name" ilike concat('%', $1, '%')
                ) and
                ($2::integer is null or "building"."id" = $2) and
                ($3::integer is null or "floor"."id" = $3) and
                "device"."deleted_at" is null
        "#,
        )
        .bind(params.query)
        .bind(params.building_id)
        .bind(params.floor_id)
        .map(|row: PgRow| row.get("count"))
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }

    async fn find(
        &self,
        params: ListDeviceParams,
    ) -> Result<PaginationResult<Device>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
                select
                    cast("device_result"."count" as integer) as "count",
                    "device"."id" as "device_id",
                    "device"."name" as "device_name",
                    concat("building"."name", ', ', "floor"."name") as "device_location",
                    "device"."description" as "device_description",
                    cast("device_announcement_result"."count" as integer) as "device_active_announcements"
                from "device" 
                join "floor" on "floor"."id" = "device"."floor_id"
                join "building" on "building"."id" = "floor"."building_id"
                left join lateral (
                    select count(*) as "count" from "device"
                    join "floor" on "floor"."id" = "device"."floor_id"
                    join "building" on "building"."id" = "floor"."building_id"
                    where
                        (
                            $3::text is null or 
                            "device"."id" = cast(
                                (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                            ) or
                            "device"."name" ilike concat('%', $3, '%')
                        ) and
                        ($4::integer is null or "building"."id" = $4) and
                        ($5::integer is null or "floor"."id" = $5) and
                        "device"."deleted_at" is null
                ) "device_result" on true
                left join lateral (
                    select count(*) as "count" from "device_announcement"
                    join "announcement" on "announcement"."id" = "device_announcement"."announcement_id"
                    where 
                        "device_id" = "device"."id" and
                        "announcement"."status" = 'active'
                ) "device_announcement_result" on true
                where 
                    (
                        $3::text is null or 
                        "device"."id" = cast(
                            (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                        ) or
                        "device"."name" ilike concat('%', $3, '%')
                    ) and
                    ($4::integer is null or "building"."id" = $4) and
                    ($5::integer is null or "floor"."id" = $5) and
                    "device"."deleted_at" is null
                group by "device"."id", "building"."id", "floor"."id", "device_result"."count", "device_announcement_result"."count"
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
            device_active_announcements: row.get("device_active_announcements"),
        })
        .fetch_all(&self._db)
        .await?;

        let mut count = 0;
        if result.len() > 0 {
            count = result[0].count;
        }
        let total_pages = (count as f64 / params.limit as f64).ceil() as i32;
        let has_next = ((params.page as f64 * params.limit as f64) / count as f64) < 1.0;

        let contents = result
            .iter()
            .map(|item| Device {
                id: item.device_id,
                name: item.device_name.to_string(),
                location: item.device_location.to_string(),
                description: item.device_description.to_string(),
                active_announcements: item.device_active_announcements,
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
                "floor"."id" as "floor_id",
                "floor"."name" as "floor_name",
                "building"."id" as "building_id",
                "building"."name" as "building_name",
                "building"."color" as "building_color",
                "device"."description" as "description",
                cast("device_announcement_result"."count" as integer) as "active_announcements",
                "device"."access_key_id" as "access_key_id",
                "device"."secret_access_key" as "secret_access_key",
                "device"."secret_access_key_salt" as "secret_access_key_salt",
                "device"."created_at" as "created_at",
                "device"."updated_at" as "updated_at",
                "device"."linked_at" as "linked_at",
                "device"."camera_enabled" as "camera_enabled"
            from "device"
            join "floor" on "floor"."id" = "device"."floor_id"
            join "building" on "building"."id" = "floor"."building_id"
            left join lateral (
                select count(*) as "count" from "device_announcement"
                join "announcement" on "announcement"."id" = "device_announcement"."announcement_id"
                where 
                    "device_id" = "device"."id" and
                    "announcement"."status" = 'active'
            ) "device_announcement_result" on true
            where "device"."id" = $1 and "device"."deleted_at" is null
            "#,
        )
        .bind(device_id)
        .map(|row: PgRow| DeviceDetail {
            id: row.get("id"),
            name: row.get("name"),
            location: DeviceDetailLocation {
                text: row.get("location"),
                building_id: row.get("building_id"),
                building_name: row.get("building_name"),
                building_color: row.get("building_color"),
                floor_id: row.get("floor_id"),
                floor_name: row.get("floor_name"),
            },
            floor_id: row.get("floor_id"),
            building_id: row.get("building_id"),
            description: row.get("description"),
            active_announcements: row.get("active_announcements"),
            access_key_id: row.get("access_key_id"),
            secret_access_key: row.get("secret_access_key"),
            secret_access_key_salt: row.get("secret_access_key_salt"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            linked_at: row.get("linked_at"),
            camera_enabled: row.get("camera_enabled"),
        })
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }

    async fn find_one_by_access_key_id(
        &self,
        access_key_id: String,
    ) -> Result<DeviceDetail, sqlx::Error> {
        let result = sqlx::query(
            r#"
            select
                "device"."id" as "id",
                "device"."name" as "name",
                concat("building"."name", ', ', "floor"."name") as "location",
                "floor"."id" as "floor_id",
                "floor"."name" as "floor_name",
                "building"."id" as "building_id",
                "building"."name" as "building_name",
                "building"."color" as "building_color",
                "device"."description" as "description",
                cast("device_announcement_result"."count" as integer) as "active_announcements",
                "device"."access_key_id" as "access_key_id",
                "device"."secret_access_key" as "secret_access_key",
                "device"."secret_access_key_salt" as "secret_access_key_salt",
                "device"."created_at" as "created_at",
                "device"."updated_at" as "updated_at",
                "device"."linked_at" as "linked_at",
                "device"."camera_enabled" as "camera_enabled"
            from "device"
            join "floor" on "floor"."id" = "device"."floor_id"
            join "building" on "building"."id" = "floor"."building_id"
            left join lateral (
                select count(*) as "count" from "device_announcement"
                join "announcement" on "announcement"."id" = "device_announcement"."announcement_id"
                where 
                    "device_id" = "device"."id" and
                    "announcement"."status" = 'active'
            ) "device_announcement_result" on true
            where "device"."access_key_id" = $1 and "device"."deleted_at" is null
            "#,
        )
        .bind(access_key_id)
        .map(|row: PgRow| DeviceDetail {
            id: row.get("id"),
            name: row.get("name"),
            location: DeviceDetailLocation {
                text: row.get("location"),
                building_id: row.get("building_id"),
                building_name: row.get("building_name"),
                building_color: row.get("building_color"),
                floor_id: row.get("floor_id"),
                floor_name: row.get("floor_name"),
            },
            floor_id: row.get("floor_id"),
            building_id: row.get("building_id"),
            description: row.get("description"),
            active_announcements: row.get("active_announcements"),
            access_key_id: row.get("access_key_id"),
            secret_access_key: row.get("secret_access_key"),
            secret_access_key_salt: row.get("secret_access_key_salt"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            linked_at: row.get("linked_at"),
            camera_enabled: row.get("camera_enabled"),
        })
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }

    async fn insert(&self, params: InsertDeviceParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "device" ("name", "description", "floor_id", "access_key_id", "secret_access_key", "secret_access_key_salt")
            values ($1, $2, $3, $4, $5, $6)
            returning "id"
            "#,
            params.name,
            params.description,
            params.floor_id,
            params.access_key_id,
            params.secret_access_key.as_bytes(),
            params.secret_access_key_salt,
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
            where "id" = $1 and "deleted_at" is null
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
                update "device"
                set "deleted_at" = now()
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

    async fn exists(&self, device_ids: &Vec<i32>) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            select cast(count(*) as integer) as "count"
            from "device"
            where "id" = any($1) and "deleted_at" is null
            "#,
            device_ids,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.count.unwrap() == device_ids.len() as i32)
    }

    async fn find_announcement_ids_in_device(
        &self,
        device_id: i32,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            select "announcement_id" from "device_announcement"
            where "device_id" = $1
            "#,
            device_id,
        )
        .fetch_all(&self._db)
        .await?;

        Ok(result.into_iter().map(|row| row.announcement_id).collect())
    }

    async fn update_device_link(&self, device_id: i32, link: bool) -> Result<(), sqlx::Error> {
        let rows_affected = if link {
            sqlx::query!(
                r#"
            update "device"
            set "linked_at" = now()
            where "id" = $1
            "#,
                device_id,
            )
            .execute(&self._db)
            .await?
            .rows_affected()
        } else {
            sqlx::query!(
                r#"
            update "device"
            set "linked_at" = null
            where "id" = $1
            "#,
                device_id,
            )
            .execute(&self._db)
            .await?
            .rows_affected()
        };
        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
    async fn update_camera_enabled(
        &self,
        device_id: i32,
        camera_enabled: bool,
    ) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
            update "device"
            set "camera_enabled" = $2
            where "id" = $1
            "#,
            device_id,
            camera_enabled,
        )
        .execute(&self._db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn get_auth_cache(&self, access_key_id: String) -> Result<DeviceAuthCache, RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        let result = cmd("GET")
            .arg(&[self.device_auth_key_builder(access_key_id)])
            .query_async::<_, String>(&mut conn)
            .await?;

        let cache: DeviceAuthCache = serde_json::from_str(result.as_str()).unwrap();

        Ok(cache)
    }

    async fn set_auth_cache(
        &self,
        access_key_id: String,
        cache: DeviceAuthCache,
    ) -> Result<(), RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        cmd("SET")
            .arg(&[
                self.device_auth_key_builder(access_key_id),
                serde_json::to_string(&cache).unwrap(),
            ])
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn del_auth_cache(&self, access_key_id: String) -> Result<(), RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        cmd("DEL")
            .arg(&[self.device_auth_key_builder(access_key_id)])
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn get_device_status(&self, device_id: i32) -> Result<DeviceStatus, RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        let result = cmd("HGET")
            .arg(&[DEVICE_STATUS_REDIS_KEY, device_id.to_string().as_str()])
            .query_async::<_, Option<String>>(&mut conn)
            .await?;

        let raw_date = if let Some(res) = result {
            res
        } else {
            return Ok(DeviceStatus::Unregistered);
        };

        let parsed_date = match DateTime::parse_from_rfc3339(raw_date.as_str()) {
            Ok(date) => Some(date),
            Err(_) => None,
        };

        if let Some(date) = parsed_date {
            let now = Utc::now();

            if date + Duration::seconds(TIMEOUT_DURATION_SECS) < now {
                return Ok(DeviceStatus::Disconnected);
            }
        } else {
            return Ok(DeviceStatus::Unregistered);
        }

        return Ok(DeviceStatus::Connected);
    }

    async fn set_device_status(
        &self,
        device_id: i32,
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        let value = if let Some(date) = timestamp {
            date.to_rfc3339()
        } else {
            String::new()
        };

        cmd("HSET")
            .arg(&[
                DEVICE_STATUS_REDIS_KEY,
                device_id.to_string().as_str(),
                value.as_str(),
            ])
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }
}
