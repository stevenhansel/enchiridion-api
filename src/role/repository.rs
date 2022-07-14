use async_trait::async_trait;
use shaku::{Component, Interface};
use sqlx::{Pool, Postgres};

use super::Role;

#[async_trait]
pub trait RoleRepositoryInterface: Interface {
    async fn find(&self) -> Result<Vec<Role>, sqlx::Error>;
}

#[derive(Component)]
#[shaku(interface = RoleRepositoryInterface)]
pub struct RoleRepository {
    _db: Pool<Postgres>,
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
}
