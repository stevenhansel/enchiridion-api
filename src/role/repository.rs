use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use super::Role;

#[async_trait]
pub trait RoleRepositoryInterface {
    async fn find(&self) -> Result<Vec<Role>, sqlx::Error>;
}

pub struct RoleRepository {
    _db: Pool<Postgres>,
}

impl RoleRepository {
    pub fn new(db: Pool<Postgres>) -> RoleRepository {
        RoleRepository {
            _db: db,
        }
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
}
