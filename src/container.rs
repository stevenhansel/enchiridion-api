use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use shaku::{module, Component, Interface};
use sqlx::{Pool, Postgres};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterParams {
    name: String,
    email: String,
    password: String,
    registration_reason: String,
}

pub struct InsertUserParams<'a> {
    name: String,
    email: String,
    password: &'a [u8],
    registration_reason: String,
}

#[async_trait]
pub trait UserRepositoryInterface: Interface {
    async fn create<'a>(&self, params: &InsertUserParams<'a>) -> Result<i32, sqlx::Error>;
}

#[derive(Component)]
#[shaku(interface = UserRepositoryInterface)]
pub struct UserRepository {
    _db: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(db: Pool<Postgres>) -> UserRepository {
        UserRepository { _db: db }
    }
}

#[async_trait]
impl UserRepositoryInterface for UserRepository {
    async fn create<'a>(&self, params: &InsertUserParams<'a>) -> Result<i32, sqlx::Error> {
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
}

#[async_trait]
pub trait AuthServiceInterface: Interface {
    async fn register<'a>(&self, params: &InsertUserParams<'a>) -> Result<bool, String>;
}

#[derive(Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct AuthService {
    #[shaku(inject)]
    _user_repository: Arc<dyn UserRepositoryInterface>,
}

impl AuthService {
    pub fn new(user_repository: Arc<dyn UserRepositoryInterface>) -> AuthService {
        AuthService {
            _user_repository: user_repository,
        }
    }
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn register<'a>(&self, params: &InsertUserParams<'a>) -> Result<bool, String> {
        let user_id = match self._user_repository.create(params).await {
            Ok(id) => id,
            Err(e) => return Err(e.to_string()),
        };
        println!("user_id: {}", user_id);

        Ok(true)
    }
}

module! {
    pub Container {
        components = [UserRepository, AuthService],
        providers = []
    }
}
