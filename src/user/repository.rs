use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use super::domain::{User, UserStatus};

pub struct InsertUserParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub registration_reason: Option<String>,
    pub role_id: i32,
}

#[async_trait]
pub trait UserRepositoryInterface {
    async fn create(&self, params: InsertUserParams) -> Result<i32, sqlx::Error>;
    async fn find_one_by_id(&self, id: i32) -> Result<User, sqlx::Error>;
    async fn find_one_by_email(&self, email: String) -> Result<User, sqlx::Error>;
    async fn confirm_email(&self, id: i32) -> Result<(), sqlx::Error>;
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

    async fn find_one_by_id(&self, id: i32) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
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

    async fn find_one_by_email(&self, email: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
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
}
