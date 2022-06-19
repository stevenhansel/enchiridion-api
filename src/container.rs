use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher},
    Argon2,
};
use async_trait::async_trait;
use shaku::{module, Component, Interface};
use sqlx::{Pool, Postgres};

pub struct RegisterParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub reason: String,
}

pub struct InsertUserParams<'a> {
    name: String,
    email: String,
    password: &'a [u8],
    registration_reason: String,
}

pub struct User {
    id: i32,
    name: String,
    email: String,
    password: Vec<u8>,
    registration_reason: String,
}

#[async_trait]
pub trait UserRepositoryInterface: Interface {
    async fn create<'a>(&self, params: &'a InsertUserParams) -> Result<i32, sqlx::Error>;
    async fn find_one_by_id(&self, id: i32) -> Result<User, sqlx::Error>;
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
}

#[async_trait]
pub trait AuthServiceInterface: Interface {
    async fn register(&self, params: RegisterParams) -> Result<User, String>;
}

#[derive(Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct AuthService {
    #[shaku(inject)]
    _user_repository: Arc<dyn UserRepositoryInterface>,
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn register(&self, params: RegisterParams) -> Result<User, String> {
        let hash = match Argon2::default()
            .hash_password(params.password.as_bytes(), "secretsecretsecretsecret")
        {
            Ok(p) => p.serialize(),
            Err(e) => return Err(e.to_string()),
        };
        let password = hash.as_bytes();

        let params = InsertUserParams {
            name: params.name,
            email: params.password,
            registration_reason: params.reason,
            password,
        };

        let user_id = match self._user_repository.create(&params).await {
            Ok(id) => id,
            Err(e) => return Err(e.to_string()),
        };
        let user = match self._user_repository.find_one_by_id(user_id).await {
            Ok(user) => user,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }
}

module! {
    pub Container {
        components = [UserRepository, AuthService],
        providers = []
    }
}
