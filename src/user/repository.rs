use async_trait::async_trait;
use shaku::{Component, Interface};
use sqlx::{Pool, Postgres};

use super::domain::User;

pub struct InsertUserParams<'a> {
    pub name: String,
    pub email: String,
    pub password: &'a [u8],
    pub registration_reason: String,
}

#[async_trait]
pub trait UserRepositoryInterface: Interface {
    async fn create<'a>(&self, params: &'a InsertUserParams) -> Result<i32, sqlx::Error>;
    async fn find_one_by_id(&self, id: i32) -> Result<User, sqlx::Error>;
    async fn find_one_by_email(&self, email: String) -> Result<User, sqlx::Error>;
}

#[derive(Component)]
#[shaku(interface = UserRepositoryInterface)]
pub struct UserRepository {
    _db: Pool<Postgres>,
}

#[async_trait]
impl UserRepositoryInterface for UserRepository {
    async fn create<'a>(&self, params: &'a InsertUserParams) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            insert into "user" (name, email, password, registration_reason)
            values($1, $2, $3, $4)
            returning id
            "#,
            params.name,
            params.email,
            params.password,
            params.registration_reason,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(result.id)
    }

    async fn find_one_by_id(&self, id: i32) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            select id, name, email, password, registration_reason from "user"
            where id = $1
            "#,
            id
        )
        .fetch_one(&self._db)
        .await?;

        Ok(user)
    }

    async fn find_one_by_email(&self, email: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            select id, name, email, password, registration_reason from "user"
            where email = $1
            "#,
            email,
        )
        .fetch_one(&self._db)
        .await?;

        Ok(user)
    }
}