use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::database::PaginationResult;

use super::{Announcement, AnnouncementDetail, AnnouncementDetailDevices, AnnouncementStatus};

pub struct FindListAnnouncementParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub status: Option<AnnouncementStatus>,
    pub user_id: Option<i32>,
}

pub struct InsertAnnouncementParams {
    pub title: String,
    pub media: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub notes: String,
    pub device_ids: Vec<i32>,
    pub user_id: i32,
}

pub struct ListAnnouncementRow {
    count: i32,
    announcement_id: i32,
    announcement_title: String,
    announcement_start_date: chrono::DateTime<chrono::Utc>,
    announcement_end_date: chrono::DateTime<chrono::Utc>,
    announcement_status: AnnouncementStatus,
    announcement_media: String,
    announcement_created_at: chrono::DateTime<chrono::Utc>,
    announcement_updated_at: chrono::DateTime<chrono::Utc>,
    user_id: i32,
    user_name: String,
}

pub struct AnnouncementDetailRow {
    announcement_id: i32,
    announcement_title: String,
    announcement_start_date: chrono::DateTime<chrono::Utc>,
    announcement_end_date: chrono::DateTime<chrono::Utc>,
    announcement_status: AnnouncementStatus,
    announcement_media: String,
    announcement_notes: String,
    announcement_created_at: chrono::DateTime<chrono::Utc>,
    announcement_updated_at: chrono::DateTime<chrono::Utc>,
    user_id: i32,
    user_name: String,
    device_id: i32,
    device_name: String,
    device_description: String,
    device_floor_id: i32,
}

#[async_trait]
pub trait AnnouncementRepositoryInterface {
    async fn find(
        &self,
        params: FindListAnnouncementParams,
    ) -> Result<PaginationResult<Announcement>, sqlx::Error>;
    async fn find_one(&self, announcement_id: i32) -> Result<AnnouncementDetail, sqlx::Error>;
    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error>;
}

pub struct AnnouncementRepository {
    _db: Pool<Postgres>,
}

impl AnnouncementRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        AnnouncementRepository { _db }
    }
}

#[async_trait]
impl AnnouncementRepositoryInterface for AnnouncementRepository {
    async fn find(
        &self,
        params: FindListAnnouncementParams,
    ) -> Result<PaginationResult<Announcement>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
            select
                cast(count("announcement".*) over () as integer) as "count",
                "announcement"."id" as "announcement_id",
                "announcement"."title" as "announcement_title",
                "announcement"."start_date" as "announcement_start_date",
                "announcement"."end_date" as "announcement_end_date",
                "announcement"."status" as "announcement_status",
                "announcement"."media" as "announcement_media",
                "announcement"."created_at" as "announcement_created_at",
                "announcement"."updated_at" as "announcement_updated_at",
                "user"."id" as "user_id",
                "user"."name" as "user_name"
            from "announcement"
            join "user" on "user"."id" = "announcement"."user_id"
            where
                ($3::text is null or "announcement"."title" ilike concat('%', $3, '%')) and
                ($4::text is null or "announcement"."status" = $4) and
                ($5::integer is null or "announcement"."user_id" = $5)
            order by "announcement"."id" desc
            offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.query.clone())
        .bind(params.status.clone())
        .bind(params.user_id.clone())
        .map(|row: PgRow| ListAnnouncementRow {
            count: row.get("count"),
            announcement_id: row.get("announcement_id"),
            announcement_title: row.get("announcement_title"),
            announcement_start_date: row.get("announcement_start_date"),
            announcement_end_date: row.get("announcement_end_date"),
            announcement_status: row.get("announcement_status"),
            announcement_media: row.get("announcement_media"),
            announcement_created_at: row.get("announcement_created_at"),
            announcement_updated_at: row.get("announcement_updated_at"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
        })
        .fetch_all(&self._db)
        .await?;

        let mut count = 0;
        if result.len() > 0 {
            count = result[0].count;
        }

        let total_pages = (count as f64 / params.limit as f64).ceil() as i32;
        let has_next = (params.page as f64 * params.limit as f64) < 1.0;

        let contents: Vec<Announcement> = result
            .into_iter()
            .map(|row| Announcement {
                id: row.announcement_id,
                title: row.announcement_title,
                start_date: row.announcement_start_date,
                end_date: row.announcement_end_date,
                status: row.announcement_status,
                user_id: row.user_id,
                user_name: row.user_name,
                media: row.announcement_media,
                created_at: row.announcement_created_at,
                updated_at: row.announcement_updated_at,
            })
            .collect();

        Ok(PaginationResult {
            count,
            total_pages,
            has_next,
            contents,
        })
    }

    async fn find_one(&self, announcement_id: i32) -> Result<AnnouncementDetail, sqlx::Error> {
        let result = sqlx::query(
                r#"
                select
                    "announcement"."id" as "announcement_id",
                    "announcement"."title" as "announcement_title",
                    "announcement"."media" as "announcement_media",
                    "announcement"."notes" as "announcement_notes",
                    "announcement"."status" as "announcement_status",
                    "announcement"."start_date" as "announcement_start_date",
                    "announcement"."end_date" as "announcement_end_date",
                    "announcement"."created_at" as "announcement_created_at",
                    "announcement"."updated_at" as "announcement_updated_at",
                    "user"."id" as "user_id",
                    "user"."name" as "user_name",
                    "device"."id" as "device_id",
                    "device"."name" as "device_name",
                    "device"."description" as "device_description",
                    "device"."floor_id" as "device_floor_id"
                from "announcement"
                join "user" on "user"."id" = "announcement"."user_id"
                join "device_announcement" on "device_announcement"."announcement_id" = "announcement"."id"
                join "device" on "device"."id" = "device_announcement"."device_id"
                where "announcement"."id" = $1
                "#,
        )
        .bind(announcement_id)
        .map(|row: PgRow| AnnouncementDetailRow {
            announcement_id: row.get("announcement_id"),
            announcement_title: row.get("announcement_title"),
            announcement_start_date: row.get("announcement_start_date"),
            announcement_end_date: row.get("announcement_end_date"),
            announcement_status: row.get("announcement_status"),
            announcement_media: row.get("announcement_media"),
            announcement_notes: row.get("announcement_notes"),
            announcement_created_at: row.get("announcement_created_at"),
            announcement_updated_at: row.get("announcement_updated_at"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_description: row.get("device_description"),
            device_floor_id: row.get("device_floor_id"),
        })
        .fetch_all(&self._db)
        .await?;

        if result.len() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(AnnouncementDetail {
            id: result[0].announcement_id,
            title: result[0].announcement_title.clone(),
            media: result[0].announcement_media.clone(),
            notes: result[0].announcement_notes.clone(),
            status: result[0].announcement_status.clone(),
            start_date: result[0].announcement_start_date,
            end_date: result[0].announcement_end_date,
            created_at: result[0].announcement_created_at,
            updated_at: result[0].announcement_updated_at,
            user_id: result[0].user_id,
            user_name: result[0].user_name.clone(),
            devices: result
                .into_iter()
                .map(|row| AnnouncementDetailDevices {
                    id: row.device_id,
                    name: row.device_name,
                    description: row.device_description,
                    floor_id: row.device_floor_id,
                })
                .collect(),
        })
    }

    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                with cte_announcement as (
                    insert into "announcement" ("title", "media", "start_date", "end_date", "notes", "user_id")
                    values ($1, $2, $3, $4, $5, $6)
                    returning "id"
                )
                insert into "device_announcement" ("announcement_id", "device_id")
                values ((select "id" from "cte_announcement"), unnest($7::int4[]))
                returning (select "id" from "cte_announcement")
            "#,
            params.title,
            params.media,
            params.start_date,
            params.end_date,
            params.notes,
            params.user_id,
            &params.device_ids,
        ).fetch_one(&self._db).await?;

        match result.id {
            Some(id) => Ok(id),
            None => Err(sqlx::Error::RowNotFound),
        }
    }
}
