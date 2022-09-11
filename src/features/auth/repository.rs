use async_trait::async_trait;
use deadpool_redis::redis::{cmd, RedisError};
use sqlx::{Pool, Postgres};

use crate::{
    config::Configuration,
    features::{
        role::{PermissionObject, RoleObject, DEFAULT_ROLES},
        user::UserStatus,
    },
};

use super::UserAuthEntity;

pub struct RawAuthEntity {
    user_id: i32,
    user_name: String,
    user_email: String,
    user_profile_picture: Option<String>,
    user_is_email_confirmed: bool,
    user_status: UserStatus,
    user_role: String,
}

#[async_trait]
pub trait AuthRepositoryInterface {
    async fn find_one_auth_entity_by_email(
        &self,
        email: String,
    ) -> Result<UserAuthEntity, sqlx::Error>;
    async fn find_one_auth_entity_by_id(&self, id: i32) -> Result<UserAuthEntity, sqlx::Error>;
    async fn set_user_refresh_token(
        &self,
        user_id: i32,
        refresh_token: String,
    ) -> Result<(), redis::RedisError>;
    async fn delete_user_refresh_token(&self, user_id: i32) -> Result<(), redis::RedisError>;
}

pub struct AuthRepository {
    _db: Pool<Postgres>,
    _redis: deadpool_redis::Pool,
    _configuration: Configuration,
}

impl AuthRepository {
    pub fn new(
        _db: Pool<Postgres>,
        _redis: deadpool_redis::Pool,
        _configuration: Configuration,
    ) -> AuthRepository {
        AuthRepository {
            _db,
            _redis,
            _configuration,
        }
    }

    fn refresh_token_key_builder(&self, user_id: i32) -> String {
        format!("auth/refresh_token_{}", user_id)
    }

    fn map_user_auth_entity(&self, raw: RawAuthEntity) -> UserAuthEntity {
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

        let entity = UserAuthEntity {
            id: raw.user_id,
            name: raw.user_name.to_string(),
            email: raw.user_email.to_string(),
            profile_picture: raw.user_profile_picture.clone(),
            is_email_confirmed: raw.user_is_email_confirmed,
            user_status: raw.user_status.clone(),
            role: roles
                .into_iter()
                .find(|r| r.value == raw.user_role)
                .unwrap(),
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
                "user"."role" as "user_role"
            from "user"
            where email = $1
            "#,
            email,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(self.map_user_auth_entity(raw))
    }

    async fn find_one_auth_entity_by_id(&self, id: i32) -> Result<UserAuthEntity, sqlx::Error> {
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
                "user"."role" as "user_role"
            from "user"
            where "user"."id" = $1
            "#,
            id,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(self.map_user_auth_entity(raw))
    }

    async fn set_user_refresh_token(
        &self,
        user_id: i32,
        refresh_token: String,
    ) -> Result<(), RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        let key = self.refresh_token_key_builder(user_id);
        let expire_at = (chrono::Utc::now()
            + chrono::Duration::seconds(self._configuration.email_confirmation_expiration_seconds))
        .timestamp();

        cmd("SET")
            .arg(&[key.clone(), refresh_token])
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();
        cmd("EXPIREAT")
            .arg(&[key.clone(), expire_at.to_string()])
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn delete_user_refresh_token(&self, user_id: i32) -> Result<(), RedisError> {
        let mut conn = self
            ._redis
            .get()
            .await
            .expect("Cannot get redis connection");

        cmd("DEL")
            .arg(&[self.refresh_token_key_builder(user_id)])
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }
}
