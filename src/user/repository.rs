use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::database::PaginationResult;

use super::domain::{User, UserDetail, UserStatus};

pub struct FindUserParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub status: Option<UserStatus>,
    pub role_id: Option<i32>,
}

pub struct InsertUserParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub registration_reason: Option<String>,
    pub role_id: i32,
}

struct ListUserRow {
    pub count: i32,
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub role_id: i32,
    pub role_name: String,
    pub user_status: UserStatus,
    pub user_registration_reason: Option<String>,
}

#[async_trait]
pub trait UserRepositoryInterface {
    async fn create(&self, params: InsertUserParams) -> Result<i32, sqlx::Error>;
    async fn find(&self, params: FindUserParams) -> Result<PaginationResult<User>, sqlx::Error>;
    async fn find_one_by_id(&self, id: i32) -> Result<UserDetail, sqlx::Error>;
    async fn find_one_by_email(&self, email: String) -> Result<UserDetail, sqlx::Error>;
    async fn confirm_email(&self, id: i32) -> Result<(), sqlx::Error>;
    async fn update_user_approval(&self, user_id: i32, approve: bool) -> Result<(), sqlx::Error>;
}

pub struct UserRepository {
    _db: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(_db: Pool<Postgres>) -> UserRepository {
        UserRepository { _db }
    }
}

#[async_trait]
impl UserRepositoryInterface for UserRepository {
    async fn create(&self, params: InsertUserParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "user" (name, email, password, registration_reason, role_id)
            values($1, $2, $3, $4, $5)
            returning id
            "#,
            params.name,
            params.email,
            params.password.as_bytes(),
            params.registration_reason,
            params.role_id,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }

    async fn find(&self, params: FindUserParams) -> Result<PaginationResult<User>, sqlx::Error> {
        let offset = (params.page - 1) * params.limit;

        let result = sqlx::query(
            r#"
            select
                cast("result"."count" as integer) as "count",
                "user"."id" as "user_id",
                "user"."name" as "user_name",
                "user"."email" as "user_email",
                "role"."id" as "role_id",
                "role"."name" as "role_name",
                "user"."status" as "user_status",
                "user"."registration_reason" as "user_registration_reason"
            from "user"
            join "role" on "role"."id" = "user"."role_id"
            left join lateral (
                select count(*) from "user"
                where
                    (
                        $3::text is null or 
                        "user"."id" = cast(
                            (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                        ) or
                        "user"."name" ilike concat('%', $3, '%')
                    ) and
                    ($4::user_status is null or "user"."status" = $4) and
                    ($5::integer is null or "user"."role_id" = $5)
            ) "result" on true
            where
                (
                    $3::text is null or 
                    "user"."id" = cast(
                        (coalesce(nullif(regexp_replace($3, '[^0-9]+', '', 'g'), ''), '0')) as integer    
                    ) or
                    "user"."name" ilike concat('%', $3, '%')
                ) and
                ($4::user_status is null or "user"."status" = $4) and
                ($5::integer is null or "user"."role_id" = $5)
            order by "user"."id" desc
            offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.query)
        .bind(params.status)
        .bind(params.role_id)
        .map(|row: PgRow| ListUserRow {
            count: row.get("count"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
            role_id: row.get("role_id"),
            role_name: row.get("role_name"),
            user_status: row.get("user_status"),
            user_registration_reason: row.get("user_registration_reason"),
        })
        .fetch_all(&self._db)
        .await?;

        let mut count = 0;
        if result.len() > 0 {
            count = result[0].count;
        }

        let total_pages = (count as f64 / params.limit as f64).ceil() as i32;
        let has_next = ((params.page as f64 * params.limit as f64) / count as f64) < 1.0;

        let contents: Vec<User> = result
            .into_iter()
            .map(|row| User {
                id: row.user_id,
                name: row.user_name,
                email: row.user_email,
                role_id: row.role_id,
                role_name: row.role_name,
                status: row.user_status,
                registration_reason: row.user_registration_reason,
            })
            .collect();

        Ok(PaginationResult {
            count,
            total_pages,
            has_next,
            contents,
        })
    }

    async fn find_one_by_id(&self, id: i32) -> Result<UserDetail, sqlx::Error> {
        let user = sqlx::query_as!(
            UserDetail,
            r#"
            select 
                id,
                name,
                email,
                password,
                registration_reason,
                is_email_confirmed,
                status as "status: UserStatus" 
            from "user"
            where id = $1
            "#,
            id
        )
        .fetch_one(&self._db)
        .await?;

        Ok(user)
    }

    async fn find_one_by_email(&self, email: String) -> Result<UserDetail, sqlx::Error> {
        let user = sqlx::query_as!(
            UserDetail,
            r#"
            select 
                id,
                name,
                email,
                password,
                registration_reason,
                is_email_confirmed,
                status as "status: UserStatus"
            from "user"
            where email = $1
            "#,
            email,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(user)
    }

    async fn confirm_email(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            update "user"
            set is_email_confirmed = true
            where id = $1
            "#,
            id,
        )
        .execute(&self._db)
        .await?;

        Ok(())
    }

    async fn update_user_approval(&self, user_id: i32, approve: bool) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
                update "user"
                set "status" = $2
                where "id" = $1
                "#,
            user_id,
            match approve {
                true => UserStatus::Approved,
                false => UserStatus::Rejected,
            } as _,
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
