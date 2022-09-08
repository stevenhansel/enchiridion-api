use std::collections::BTreeMap;

use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::database::PaginationResult;

use super::{Announcement, AnnouncementDetail, AnnouncementDetailDevices, AnnouncementStatus};

pub struct CountAnnouncementParams {
    pub query: Option<String>,
    pub status: Option<AnnouncementStatus>,
    pub user_id: Option<i32>,
    pub device_id: Option<i32>,

    pub start_date_gte: Option<chrono::DateTime<chrono::Utc>>,
    pub start_date_lt: Option<chrono::DateTime<chrono::Utc>>,

    pub end_date_lte: Option<chrono::DateTime<chrono::Utc>>,
}

impl CountAnnouncementParams {
    pub fn default() -> Self {
        CountAnnouncementParams {
            query: None,
            status: None,
            user_id: None,
            device_id: None,

            start_date_gte: None,
            start_date_lt: None,
            end_date_lte: None,
        }
    }

    pub fn query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    pub fn status(mut self, status: AnnouncementStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn device_id(mut self, device_id: i32) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn start_date_gte(mut self, start_date_gte: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_date_gte = Some(start_date_gte);
        self
    }

    pub fn start_date_gt(mut self, start_date_gt: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_date_lt = Some(start_date_gt);
        self
    }

    pub fn end_date_lte(mut self, start_date_gte: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_date_lte = Some(start_date_gte);
        self
    }
}

pub struct FindListAnnouncementParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub status: Option<AnnouncementStatus>,
    pub user_id: Option<i32>,
    pub device_id: Option<i32>,

    pub start_date_gte: Option<chrono::DateTime<chrono::Utc>>,
    pub start_date_lt: Option<chrono::DateTime<chrono::Utc>>,

    pub end_date_lte: Option<chrono::DateTime<chrono::Utc>>,
}

impl FindListAnnouncementParams {
    pub fn default() -> Self {
        FindListAnnouncementParams {
            page: 1,
            limit: 25,

            query: None,
            status: None,
            user_id: None,
            device_id: None,

            start_date_gte: None,
            start_date_lt: None,
            end_date_lte: None,
        }
    }

    pub fn page(mut self, page: i32) -> Self {
        self.page = page;
        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = limit;
        self
    }

    pub fn query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    pub fn status(mut self, status: AnnouncementStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn device_id(mut self, device_id: i32) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn start_date_gte(mut self, start_date_gte: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_date_gte = Some(start_date_gte);
        self
    }

    pub fn start_date_gt(mut self, start_date_gt: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_date_lt = Some(start_date_gt);
        self
    }

    pub fn end_date_lte(mut self, start_date_gte: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_date_lte = Some(start_date_gte);
        self
    }
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
    async fn count(&self, params: CountAnnouncementParams) -> Result<i32, sqlx::Error>;
    async fn find(
        &self,
        params: FindListAnnouncementParams,
    ) -> Result<PaginationResult<Announcement>, sqlx::Error>;
    async fn find_one(&self, announcement_id: i32) -> Result<AnnouncementDetail, sqlx::Error>;
    async fn insert(&self, params: InsertAnnouncementParams) -> Result<i32, sqlx::Error>;
    async fn update_status(
        &self,
        announcement_id: i32,
        status: AnnouncementStatus,
    ) -> Result<(), sqlx::Error>;
    async fn batch_update_status(
        &self,
        announcement_ids: Vec<i32>,
        status: AnnouncementStatus,
    ) -> Result<(), sqlx::Error>;
    async fn find_expired_waiting_for_approval_announcement_ids(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<i32>, sqlx::Error>;
    async fn find_announcement_device_map(
        &self,
        announcement_ids: Vec<i32>,
    ) -> Result<BTreeMap<i32, Vec<i32>>, sqlx::Error>;
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
    async fn count(&self, params: CountAnnouncementParams) -> Result<i32, sqlx::Error> {
        let result: i32 = sqlx::query(
            r#"
            select
                cast(count("announcement".*) as integer) as "count"
            from "announcement"
            join "user" on "user"."id" = "announcement"."user_id"
            join "device_announcement" on "device_announcement"."announcement_id" = "announcement"."id"
            where
                (
                    $1::text is null or 
                    "announcement"."id" = cast(
                        (coalesce(nullif(regexp_replace($1, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                    ) or
                    "announcement"."title" ilike concat('%', $1, '%')
                ) and
                ($2::text is null or "announcement"."status" = $2) and
                ($3::integer is null or "announcement"."user_id" = $3) and 
                ($4::integer is null or "device_announcement"."device_id" = $4) and
                ($5::timestamp is null or "announcement"."start_date" >= $5) and
                ($6::timestamp is null or "announcement"."start_date" < $6) and
                ($7::timestamp is null or "announcement"."end_date" <= $7)
            "#,
        )
        .bind(params.query.clone())
        .bind(params.status.clone())
        .bind(params.user_id.clone())
        .bind(params.device_id.clone())
        .bind(params.start_date_gte.clone())
        .bind(params.start_date_lt.clone())
        .bind(params.end_date_lte.clone())
        .map(|row: PgRow| row.get("count"))
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }

    async fn find(
        &self,
        params: FindListAnnouncementParams,
    ) -> Result<PaginationResult<Announcement>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
            select
                cast("result"."count" as integer) as "count",
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
            join "device_announcement" on "device_announcement"."announcement_id" = "announcement"."id"
            left join lateral (
                select count(*) from "announcement"
                where
                    (
                        $3::text is null or 
                        "announcement"."id" = cast(
                            (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                        ) or
                        "announcement"."title" ilike concat('%', $3, '%')
                    ) and
                    ($4::text is null or "announcement"."status" = $4) and
                    ($5::integer is null or "announcement"."user_id" = $5) and 
                    ($6::integer is null or "device_announcement"."device_id" = $6) and
                    ($7::timestamp is null or "announcement"."start_date" >= $7) and
                    ($8::timestamp is null or "announcement"."start_date" < $8) and
                    ($9::timestamp is null or "announcement"."end_date" <= $9)
            ) "result" on true
            where
                (
                    $3::text is null or 
                    "announcement"."id" = cast(
                        (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                    ) or
                    "announcement"."title" ilike concat('%', $3, '%')
                ) and
                ($4::text is null or "announcement"."status" = $4) and
                ($5::integer is null or "announcement"."user_id" = $5) and 
                ($6::integer is null or "device_announcement"."device_id" = $6) and
                ($7::timestamp is null or "announcement"."start_date" >= $7) and
                ($8::timestamp is null or "announcement"."start_date" < $8) and
                ($9::timestamp is null or "announcement"."end_date" <= $9)
            group by "announcement"."id", "user"."id", "result"."count"
            order by "announcement"."id" desc
            offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.query.clone())
        .bind(params.status.clone())
        .bind(params.user_id.clone())
        .bind(params.device_id.clone())
        .bind(params.start_date_gte.clone())
        .bind(params.start_date_lt.clone())
        .bind(params.end_date_lte.clone())
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
        let has_next = ((params.page as f64 * params.limit as f64) / count as f64) < 1.0;

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

    async fn update_status(
        &self,
        announcement_id: i32,
        status: AnnouncementStatus,
    ) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
            update "announcement"
            set "status" = $2
            where "id" = $1
            "#,
            announcement_id,
            status as _,
        )
        .execute(&self._db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn batch_update_status(
        &self,
        announcement_ids: Vec<i32>,
        status: AnnouncementStatus,
    ) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query(
            r#"
            update "announcement"
            set "status" = $2
            where "id" = any($1)
            "#,
        )
        .bind(&announcement_ids)
        .bind(status)
        .execute(&self._db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn find_expired_waiting_for_approval_announcement_ids(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            select "id"
            from "announcement"
            where
                "status" = 'waiting_for_approval' and
                "start_date" + '1 day' <= $1
            "#,
            now,
        )
        .fetch_all(&self._db)
        .await?;

        Ok(result.into_iter().map(|row| row.id).collect())
    }

    async fn find_announcement_device_map(
        &self,
        announcement_ids: Vec<i32>,
    ) -> Result<BTreeMap<i32, Vec<i32>>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            select "announcement_id", "device_id" 
            from "device_announcement"
            where "announcement_id" = any($1)
            "#,
            &announcement_ids
        )
        .fetch_all(&self._db)
        .await?;

        let mut map: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        for res in result {
            if !map.contains_key(&res.announcement_id) {
                map.insert(res.announcement_id, Vec::new());
            }

            let val = map.get_mut(&res.announcement_id).unwrap();
            val.push(res.device_id.into());
        }

        Ok(map)
    }
}
