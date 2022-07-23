use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::database::PaginationResult;

use super::{Request, RequestActionType};

pub struct FindRequestParams {
    pub page: i32,
    pub limit: i32,
    pub request_id: Option<i32>,
    pub announcement_id: Option<i32>,
    pub user_id: Option<i32>,
    pub action_type: Option<RequestActionType>,
    pub approved_by_lsc: Option<bool>,
    pub approved_by_bm: Option<bool>,
}

pub struct InsertRequestParams {
    pub action: RequestActionType,
    pub description: String,
    pub announcement_id: i32,
    pub user_id: i32,
}

pub struct UpdateApprovalParams {
    pub request_id: i32,
    pub approved_by_lsc: Option<bool>,
    pub lsc_approver: Option<i32>,
    pub approved_by_bm: Option<bool>,
    pub bm_approver: Option<i32>,
}

pub struct ListRequestRow {
    count: i32,
    request_id: i32,
    request_action: RequestActionType,
    request_description: String,
    request_approved_by_lsc: Option<bool>,
    request_lsc_approver: Option<i32>,
    request_approved_by_bm: Option<bool>,
    request_bm_approver: Option<i32>,
    request_created_at: chrono::DateTime<chrono::Utc>,
    announcement_id: i32,
    announcement_title: String,
    user_id: i32,
    user_name: String,
}

#[async_trait]
pub trait RequestRepositoryInterface {
    async fn find(
        &self,
        params: FindRequestParams,
    ) -> Result<PaginationResult<Request>, sqlx::Error>;
    async fn find_one(&self, request_id: i32) -> Result<Request, sqlx::Error>;
    async fn insert(&self, params: InsertRequestParams) -> Result<i32, sqlx::Error>;
    async fn update_approval(&self, params: UpdateApprovalParams) -> Result<(), sqlx::Error>;
}

pub struct RequestRepository {
    _db: Pool<Postgres>,
}

impl RequestRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        RequestRepository { _db }
    }
}

#[async_trait]
impl RequestRepositoryInterface for RequestRepository {
    async fn find(
        &self,
        params: FindRequestParams,
    ) -> Result<PaginationResult<Request>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
            select
                cast(count("request".*) over () as integer) as "count",
                "request"."id" as "request_id",
                "request"."action" as "request_action",
                "request"."description" as "request_description",
                "request"."approved_by_lsc" as "request_approved_by_lsc",
                "request"."lsc_approver" as "request_lsc_approver",
                "request"."approved_by_bm" as "request_approved_by_bm",
                "request"."bm_approver" as "request_bm_approver",
                "request"."created_at" as "request_created_at",
                "announcement"."id" as "announcement_id",
                "announcement"."title" as "announcement_title",
                "user"."id" as "user_id",
                "user"."name" as "user_name"
            from "request"
            join "announcement" on "announcement"."id" = "request"."announcement_id"
            join "user" on "user"."id"= "request"."user_id"
            where
                ($3::integer is null or "request"."id" = $3) and 
                ($4::integer is null or "announcement"."id" = $4) and 
                ($5::integer is null or "user"."id" = $5) and 
                ($6::text is null or "request"."action" = $6) and 
                ($7::bool is null or "request"."approved_by_lsc" = $7) and 
                ($8::bool is null or "request"."approved_by_bm" = $8)
            order by "request"."id" desc
            offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.request_id)
        .bind(params.announcement_id)
        .bind(params.user_id)
        .bind(params.action_type)
        .bind(params.approved_by_lsc)
        .bind(params.approved_by_bm)
        .map(|row: PgRow| ListRequestRow {
            count: row.get("count"),
            request_id: row.get("request_id"),
            request_action: row.get("request_action"),
            request_description: row.get("request_description"),
            request_approved_by_lsc: row.get("request_approved_by_lsc"),
            request_lsc_approver: row.get("request_lsc_approver"),
            request_approved_by_bm: row.get("request_approved_by_bm"),
            request_bm_approver: row.get("request_bm_approver"),
            request_created_at: row.get("request_created_at"),
            announcement_id: row.get("announcement_id"),
            announcement_title: row.get("announcement_title"),
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

        Ok(PaginationResult {
            count,
            total_pages,
            has_next,
            contents: result
                .into_iter()
                .map(|row| Request {
                    id: row.request_id,
                    action: row.request_action,
                    announcement_id: row.announcement_id,
                    announcement_title: row.announcement_title,
                    user_id: row.user_id,
                    user_name: row.user_name,
                    description: row.request_description,
                    approved_by_lsc: row.request_approved_by_lsc,
                    lsc_approver: row.request_lsc_approver,
                    approved_by_bm: row.request_approved_by_bm,
                    bm_approver: row.request_bm_approver,
                    created_at: row.request_created_at,
                })
                .collect(),
        })
    }

    async fn find_one(&self, request_id: i32) -> Result<Request, sqlx::Error> {
        let result = sqlx::query(
            r#"
            select
                "request"."id" as "request_id",
                "request"."action" as "request_action",
                "request"."description" as "request_description",
                "request"."approved_by_lsc" as "request_approved_by_lsc",
                "request"."lsc_approver" as "request_lsc_approver",
                "request"."approved_by_bm" as "request_approved_by_bm",
                "request"."bm_approver" as "request_bm_approver",
                "request"."created_at" as "request_created_at",
                "announcement"."id" as "announcement_id",
                "announcement"."title" as "announcement_title",
                "user"."id" as "user_id",
                "user"."name" as "user_name"
            from "request"
            join "announcement" on "announcement"."id" = "request"."announcement_id"
            join "user" on "user"."id"= "request"."user_id"
            where "request"."id" = $1
            "#,
        )
        .bind(request_id)
        .map(|row: PgRow| Request {
            id: row.get("request_id"),
            action: row.get("request_action"),
            description: row.get("request_description"),
            approved_by_lsc: row.get("request_approved_by_lsc"),
            lsc_approver: row.get("request_lsc_approver"),
            approved_by_bm: row.get("request_approved_by_bm"),
            bm_approver: row.get("request_bm_approver"),
            created_at: row.get("request_created_at"),
            announcement_id: row.get("announcement_id"),
            announcement_title: row.get("announcement_title"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
        })
        .fetch_one(&self._db)
        .await?;

        Ok(result)
    }

    async fn insert(&self, params: InsertRequestParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "request" ("action", "description", "announcement_id", "user_id")
            values ($1, $2, $3, $4)
            returning "id"
            "#,
            params.action as _,
            params.description,
            params.announcement_id,
            params.user_id,
        )
        .fetch_one(&self._db)
        .await?;

        return Ok(result.id);
    }

    async fn update_approval(&self, params: UpdateApprovalParams) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
            update "request"
            set
                "approved_by_lsc" = $2,
                "approved_by_bm" = $3,
                "lsc_approver" = $4,
                "bm_approver" = $5
            where "id" = $1
            "#,
            params.request_id,
            params.approved_by_lsc,
            params.approved_by_bm,
            params.lsc_approver,
            params.bm_approver,
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
