use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use redis::Commands;
use sqlx::{Pool, Postgres};

use super::{Permission, Role};

pub struct RawPermission {
    pub permission_id: i32,
    pub permission_name: String,
    pub permission_label: String,
}

#[async_trait]
pub trait RoleRepositoryInterface {
    fn role_permission_cache_key_builder(&self, role_id: i32) -> String;
    async fn find(&self) -> Result<Vec<Role>, sqlx::Error>;
    fn get_role_permission_cache(&self, role_id: i32)
        -> Result<Vec<Permission>, redis::RedisError>;
    fn set_role_permission_cache(
        &self,
        role_id: i32,
        permissions: Vec<Permission>,
    ) -> Result<(), redis::RedisError>;
    async fn find_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<Permission>, sqlx::Error>;
}

pub struct RoleRepository {
    _db: Pool<Postgres>,
    _redis: Arc<Mutex<redis::Connection>>,
}

impl RoleRepository {
    pub fn new(_db: Pool<Postgres>, _redis: Arc<Mutex<redis::Connection>>) -> RoleRepository {
        RoleRepository { _db, _redis }
    }
}

#[async_trait]
impl RoleRepositoryInterface for RoleRepository {
    fn role_permission_cache_key_builder(&self, role_id: i32) -> String {
        format!("role_cache_{}", role_id)
    }

    async fn find(&self) -> Result<Vec<Role>, sqlx::Error> {
        let result = sqlx::query_as!(
            Role,
            r#"
            select id, name, description
            from role
            "#
        )
        .fetch_all(&self._db)
        .await?;

        Ok(result)
    }

    async fn find_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let result = sqlx::query_as!(
            RawPermission,
            r#"
                select
                    "permission"."id" as "permission_id",
                    "permission"."name" as "permission_name",
                    "permission"."label" as "permission_label"
                from "role"
                join "role_permission" on "role_permission"."role_id" = "role"."id"
                join "permission" on "permission"."id" = "role_permission"."permission_id"
                where "role"."id" = $1
            "#,
            role_id,
        )
        .fetch_all(&self._db)
        .await?;

        let mut permissions: Vec<Permission> = vec![];
        for p in result.iter() {
            permissions.push(Permission {
                id: p.permission_id,
                name: p.permission_name.clone(),
                label: p.permission_label.clone(),
            })
        }

        Ok(permissions)
    }

    fn get_role_permission_cache(
        &self,
        role_id: i32,
    ) -> Result<Vec<Permission>, redis::RedisError> {
        let mut redis = self._redis.lock().expect("Cannot get redis connection");

        let cache = redis.hget::<String, String, String>(
            self.role_permission_cache_key_builder(role_id),
            "cache".into(),
        )?;
        let permissions = serde_json::from_str::<Vec<Permission>>(cache.as_str())
            .expect("Failed to deserialize cache");

        Ok(permissions)
    }

    fn set_role_permission_cache(
        &self,
        role_id: i32,
        permissions: Vec<Permission>,
    ) -> Result<(), redis::RedisError> {
        let mut redis = self._redis.lock().expect("Cannot get redis connection");

        let cache = serde_json::to_string::<Vec<Permission>>(&permissions)
            .expect("Failed to stringify cache data");
        redis.hset::<String, String, String, ()>(
            self.role_permission_cache_key_builder(role_id),
            "cache".into(),
            cache,
        )?;

        Ok(())
    }
}
