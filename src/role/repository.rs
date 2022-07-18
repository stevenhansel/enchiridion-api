use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use super::{Permission, Role};

pub struct RawPermission {
    pub permission_id: i32,
    pub permission_name: String,
    pub permission_label: String,
}

#[async_trait]
pub trait RoleRepositoryInterface {
    async fn find(&self) -> Result<Vec<Role>, sqlx::Error>;
    async fn find_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<Permission>, sqlx::Error>;
}

pub struct RoleRepository {
    _db: Pool<Postgres>,
}

impl RoleRepository {
    pub fn new(db: Pool<Postgres>) -> RoleRepository {
        RoleRepository { _db: db }
    }
}

#[async_trait]
impl RoleRepositoryInterface for RoleRepository {
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
                name: p.permission_name,
                label: p.permission_label,
            })
        }

        Ok(permissions)
    }
}
