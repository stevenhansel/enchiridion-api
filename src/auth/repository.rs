use async_trait::async_trait;
use shaku::{Component, Interface};
use sqlx::{Pool, Postgres};

use crate::user::UserStatus;

use super::{PermissionAuthEntity, RoleAuthEntity, UserAuthEntity};

pub struct RawAuthEntity {
    user_id: i32,
    user_name: String,
    user_email: String,
    user_profile_picture: Option<String>,
    user_is_email_confirmed: bool,
    user_status: UserStatus,
    role_id: i32,
    role_name: String,
    permission_id: i32,
    permission_name: String,
}

#[async_trait]
pub trait AuthRepositoryInterface: Interface {
    async fn find_one_auth_entity_by_email(
        &self,
        email: String,
    ) -> Result<UserAuthEntity, sqlx::Error>;
}

#[derive(Component)]
#[shaku(interface = AuthRepositoryInterface)]
pub struct AuthRepository {
    _db: Pool<Postgres>,
}

impl AuthRepository {
    fn map_user_auth_entity(raw: &Vec<RawAuthEntity>) -> UserAuthEntity {
        let mut permissions: Vec<PermissionAuthEntity> = vec![];
        for data in raw {
            permissions.push(PermissionAuthEntity {
                id: data.permission_id,
                name: data.permission_name.to_string(),
            })
        }

        let role = RoleAuthEntity {
            id: raw[0].role_id,
            name: raw[0].role_name.to_string(),
            permissions,
        };

        let entity = UserAuthEntity {
            id: raw[0].user_id,
            name: raw[0].user_name.to_string(),
            email: raw[0].user_email.to_string(),
            profile_picture: raw[0].user_profile_picture.clone(),
            is_email_confirmed: raw[0].user_is_email_confirmed,
            user_status: raw[0].user_status.clone(),
            role,
        };

        entity
    }
}

#[async_trait]
impl AuthRepositoryInterface for AuthRepository {
    async fn find_one_auth_entity_by_email(
        &self,
        email: String,
    ) -> Result<UserAuthEntity, sqlx::Error> {
        let raw = sqlx::query_as!(
            RawAuthEntity,
            r#"
            select 
                "user"."id" as "user_id",
                "user"."name" as "user_name",
                "user"."email" as "user_email",
                "user"."profile_picture" as "user_profile_picture",
                "user"."is_email_confirmed" as "user_is_email_confirmed",
                "user"."status" as "user_status: UserStatus",
                "role"."id" as "role_id",
                "role"."name" as "role_name",
                "permission"."id" as "permission_id",
                "permission"."name" as "permission_name"
            from "user"
            join "role" on "role"."id" = "user"."id"
            join "role_permission" on "role_permission"."role_id" = "role"."id"
            join "permission" on "permission"."id" = "role_permission"."permission_id"
            where email = $1
            "#,
            email,
        )
        .fetch_all(&self._db)
        .await?;

        Ok(AuthRepository::map_user_auth_entity(&raw))
    }
}
