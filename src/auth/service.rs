use std::sync::Arc;

use argon2::{password_hash::PasswordHasher, Argon2};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use shaku::{Component, Interface};

use crate::config::Configuration;
use crate::shared::DatabaseError;
use crate::user::{InsertUserParams, UserRepositoryInterface};

use super::AuthError;

pub struct RegisterParams<'a> {
    pub name: String,
    pub email: String,
    pub password: String,
    pub reason: Option<&'a String>,
    pub role_id: i32,
}

#[async_trait]
pub trait AuthServiceInterface: Interface {
    async fn register<'a>(&self, params: &'a RegisterParams<'a>) -> Result<(), AuthError>;
}

#[derive(Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct AuthService {
    #[shaku(inject)]
    _user_repository: Arc<dyn UserRepositoryInterface>,
    _configuration: Configuration,
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn register<'a>(&self, params: &'a RegisterParams<'a>) -> Result<(), AuthError> {
        let hash = match Argon2::default().hash_password(
            params.password.as_bytes(),
            self._configuration.password_secret.expose_secret(),
        ) {
            Ok(p) => p.serialize(),
            Err(e) => return Err(AuthError::InternalServerError(e.to_string())),
        };

        match self
            ._user_repository
            .create(&InsertUserParams {
                name: params.name.to_string(),
                email: params.email.to_string(),
                registration_reason: params.reason,
                password: hash.as_bytes(),
                role_id: params.role_id,
            })
            .await
        {
            Ok(id) => id,
            Err(e) => match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(raw_code) = db_error.code() {
                        let code = raw_code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(AuthError::EmailAlreadyExists(
                                "Email is already registered in our system".into(),
                            ));
                        } else if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(AuthError::RoleNotFound("Role not found".into()));
                        }
                    }

                    return Err(AuthError::InternalServerError(db_error.to_string()));
                }
                e => return Err(AuthError::InternalServerError(e.to_string())),
            },
        };

        Ok(())
    }
}
