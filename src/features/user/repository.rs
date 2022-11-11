use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use crate::{
    database::PaginationResult,
    features::role::{PermissionObject, RoleObject, DEFAULT_ROLES},
};

use super::{
    domain::{User, UserDetail, UserStatus},
    UserBuilding,
};

pub struct FindUserParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub status: Option<UserStatus>,
    pub role: Option<String>,
}

pub struct InsertUserParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_salt: String,
    pub registration_reason: Option<String>,
    pub role: String,
    pub building_id: i32,
}

pub struct InsertRawUserParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_salt: String,
    pub registration_reason: Option<String>,
    pub role: String,
    pub is_email_confirmed: bool,
    pub status: UserStatus,
    pub building_id: i32,
}

struct ListUserRow {
    pub count: i32,
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub user_status: UserStatus,
    pub user_is_email_confirmed: bool,
    pub user_registration_reason: Option<String>,
    pub user_role: String,
    pub building_id: Option<i32>,
    pub building_name: Option<String>,
    pub building_color: Option<String>,
    pub building_created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub building_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

struct RawUserDetailRow {
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub user_password: Vec<u8>,
    pub user_password_salt: String,
    pub user_registration_reason: Option<String>,
    pub user_is_email_confirmed: bool,
    pub user_status: UserStatus,
    pub building_id: Option<i32>,
    pub building_name: Option<String>,
    pub building_color: Option<String>,
    pub building_created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub building_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
pub trait UserRepositoryInterface {
    async fn create(&self, params: InsertUserParams) -> Result<i32, sqlx::Error>;
    async fn raw_create(&self, params: InsertRawUserParams) -> Result<i32, sqlx::Error>;
    async fn find(&self, params: FindUserParams) -> Result<PaginationResult<User>, sqlx::Error>;
    async fn find_one_by_id(&self, id: i32) -> Result<UserDetail, sqlx::Error>;
    async fn find_one_by_email(&self, email: String) -> Result<UserDetail, sqlx::Error>;
    async fn confirm_email(&self, id: i32) -> Result<(), sqlx::Error>;
    async fn update_user_approval(&self, user_id: i32, approve: bool) -> Result<(), sqlx::Error>;
    async fn update_password(
        &self,
        user_id: i32,
        password: String,
        password_salt: String,
    ) -> Result<(), sqlx::Error>;
}

pub struct UserRepository {
    _db: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(_db: Pool<Postgres>) -> UserRepository {
        UserRepository { _db }
    }

    pub fn populate_role(&self, role: String) -> RoleObject {
        let roles: Vec<RoleObject> = DEFAULT_ROLES
            .into_iter()
            .map(|r| RoleObject {
                name: r.name,
                value: r.value,
                description: r.description,
                permissions: r
                    .permissions
                    .into_iter()
                    .map(|p| PermissionObject {
                        label: p.label(),
                        value: p.value(),
                    })
                    .collect(),
            })
            .collect();

        roles.into_iter().find(|r| r.value == role).unwrap()
    }
}

#[async_trait]
impl UserRepositoryInterface for UserRepository {
    async fn create(&self, params: InsertUserParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "user" (name, email, password, password_salt, registration_reason, role, building_id)
            values($1, $2, $3, $4, $5, $6, $7)
            returning id
            "#,
            params.name,
            params.email,
            params.password.as_bytes(),
            params.password_salt,
            params.registration_reason,
            params.role,
            params.building_id,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }

    async fn raw_create(&self, params: InsertRawUserParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "user" (
                name,
                email,
                password,
                password_salt,
                registration_reason,
                role,
                is_email_confirmed,
                status,
                building_id
            )
            values($1, $2, $3, $4, $5, $6, $7, $8, $9)
            returning id
            "#,
            params.name,
            params.email,
            params.password.as_bytes(),
            params.password_salt,
            params.registration_reason,
            params.role,
            params.is_email_confirmed,
            params.status as _,
            params.building_id,
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
                "user"."status" as "user_status",
                "user"."is_email_confirmed" as "user_is_email_confirmed",
                "user"."registration_reason" as "user_registration_reason",
                "user"."role" as "user_role",
                "building"."id" as "building_id",
                "building"."name" as "building_name",
                "building"."color" as "building_color",
                "building"."created_at" as "building_created_at",
                "building"."updated_at" as "building_updated_at"
            from "user"
            left join "building" on "building"."id" = "user"."building_id"
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
                    ($5::text is null or "user"."role" = $5)
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
                ($5::text is null or "user"."role" = $5)
            group by "user"."id", "result"."count", "building"."id"
            order by "user"."id" desc
            offset $1 limit $2
            "#,
        )
        .bind(offset)
        .bind(params.limit)
        .bind(params.query)
        .bind(params.status)
        .bind(params.role)
        .map(|row: PgRow| ListUserRow {
            count: row.get("count"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
            user_status: row.get("user_status"),
            user_is_email_confirmed: row.get("user_is_email_confirmed"),
            user_registration_reason: row.get("user_registration_reason"),
            user_role: row.get("user_role"),
            building_id: row.get("building_id"),
            building_name: row.get("building_name"),
            building_color: row.get("building_color"),
            building_created_at: row.get("building_created_at"),
            building_updated_at: row.get("building_updated_at"),
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
                status: row.user_status,
                registration_reason: row.user_registration_reason,
                is_email_confirmed: row.user_is_email_confirmed,
                role: self.populate_role(row.user_role),
                building: if let Some(_) = row.building_id {
                    Some(UserBuilding {
                        id: row.building_id.unwrap(),
                        name: row.building_name.unwrap(),
                        color: row.building_color.unwrap(),
                        created_at: row.building_created_at.unwrap(),
                        updated_at: row.building_updated_at.unwrap(),
                    })
                } else {
                    None
                },
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
        let row = sqlx::query(
            r#"
            select 
                "user"."id" as "user_id",
                "user"."name" as "user_name",
                "user"."email" as "user_email",
                "user"."password" as "user_password",
                "user"."password_salt" as "user_password_salt",
                "user"."registration_reason" as "user_registration_reason",
                "user"."is_email_confirmed" as "user_is_email_confirmed",
                "user"."status" as "user_status",
                "building"."id" as "building_id",
                "building"."name" as "building_name",
                "building"."color" as "building_color",
                "building"."created_at" as "building_created_at",
                "building"."updated_at" as "building_updated_at"
            from "user"
            left join "building" on "building"."id" = "user"."building_id"
            where id = $1
            "#,
        )
        .bind(id)
        .map(|row: PgRow| RawUserDetailRow {
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
            user_password: row.get("user_password"),
            user_password_salt: row.get("user_password_salt"),
            user_status: row.get("user_status"),
            user_is_email_confirmed: row.get("user_is_email_confirmed"),
            user_registration_reason: row.get("user_registration_reason"),
            building_id: row.get("building_id"),
            building_name: row.get("building_name"),
            building_color: row.get("building_color"),
            building_created_at: row.get("building_created_at"),
            building_updated_at: row.get("building_updated_at"),
        })
        .fetch_one(&self._db)
        .await?;

        let mut building: Option<UserBuilding> = None;
        if let Some(_) = row.building_id {
            building = Some(UserBuilding {
                id: row.building_id.unwrap(),
                name: row.building_name.unwrap(),
                color: row.building_color.unwrap(),
                created_at: row.building_created_at.unwrap(),
                updated_at: row.building_updated_at.unwrap(),
            })
        }

        Ok(UserDetail {
            id: row.user_id,
            name: row.user_name,
            email: row.user_email,
            password: row.user_password,
            password_salt: row.user_password_salt,
            is_email_confirmed: row.user_is_email_confirmed,
            registration_reason: row.user_registration_reason,
            status: row.user_status,
            building,
        })
    }

    async fn find_one_by_email(&self, email: String) -> Result<UserDetail, sqlx::Error> {
        let row = sqlx::query(
            r#"
            select 
                "user"."id" as "user_id",
                "user"."name" as "user_name",
                "user"."email" as "user_email",
                "user"."password" as "user_password",
                "user"."password_salt" as "user_password_salt",
                "user"."registration_reason" as "user_registration_reason",
                "user"."is_email_confirmed" as "user_is_email_confirmed",
                "user"."status" as "user_status",
                "building"."id" as "building_id",
                "building"."name" as "building_name",
                "building"."color" as "building_color",
                "building"."created_at" as "building_created_at",
                "building"."updated_at" as "building_updated_at"
            from "user"
            left join "building" on "building"."id" = "user"."building_id"
            where email = $1
            "#,
        )
        .bind(email)
        .map(|row: PgRow| RawUserDetailRow {
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            user_email: row.get("user_email"),
            user_password: row.get("user_password"),
            user_password_salt: row.get("user_password_salt"),
            user_status: row.get("user_status"),
            user_is_email_confirmed: row.get("user_is_email_confirmed"),
            user_registration_reason: row.get("user_registration_reason"),
            building_id: row.get("building_id"),
            building_name: row.get("building_name"),
            building_color: row.get("building_color"),
            building_created_at: row.get("building_created_at"),
            building_updated_at: row.get("building_updated_at"),
        })
        .fetch_one(&self._db)
        .await?;

        let mut building: Option<UserBuilding> = None;
        if let Some(_) = row.building_id {
            building = Some(UserBuilding {
                id: row.building_id.unwrap(),
                name: row.building_name.unwrap(),
                color: row.building_color.unwrap(),
                created_at: row.building_created_at.unwrap(),
                updated_at: row.building_updated_at.unwrap(),
            })
        }

        Ok(UserDetail {
            id: row.user_id,
            name: row.user_name,
            email: row.user_email,
            password: row.user_password,
            password_salt: row.user_password_salt,
            is_email_confirmed: row.user_is_email_confirmed,
            registration_reason: row.user_registration_reason,
            status: row.user_status,
            building,
        })
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

    async fn update_password(
        &self,
        user_id: i32,
        password: String,
        password_salt: String,
    ) -> Result<(), sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"
                update "user"
                set 
                    "password" = $2,
                    "password_salt" = $3
                where "id" = $1
                "#,
            user_id,
            password.as_bytes(),
            password_salt,
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
