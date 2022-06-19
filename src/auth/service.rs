use std::sync::Arc;

use argon2::{password_hash::PasswordHasher, Argon2};
use async_trait::async_trait;
use shaku::{Component, Interface};

use crate::user::{domain::User, repository::InsertUserParams, service::UserServiceInterface};

pub struct RegisterParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub reason: String,
}

#[async_trait]
pub trait AuthServiceInterface: Interface {
    async fn register(&self, params: RegisterParams) -> Result<User, String>;
}

#[derive(Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct AuthService {
    #[shaku(inject)]
    _user_service: Arc<dyn UserServiceInterface>,
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn register(&self, params: RegisterParams) -> Result<User, String> {
        match self
            ._user_service
            .get_user_by_email(params.email.clone())
            .await
        {
            Ok(_) => return Err(String::from("Email already exists")),
            _ => (),
        };

        let hash = match Argon2::default()
            .hash_password(params.password.as_bytes(), "secretsecretsecretsecret")
        {
            Ok(p) => p.serialize(),
            Err(e) => return Err(e.to_string()),
        };
        let password = hash.as_bytes();

        let params = InsertUserParams {
            name: params.name,
            email: params.email,
            registration_reason: params.reason,
            password,
        };

        let user_id = match self._user_service.create(&params).await {
            Ok(id) => id,
            Err(e) => return Err(e.to_string()),
        };
        let user = match self._user_service.get_user_by_id(user_id).await {
            Ok(user) => user,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }
}
