use std::collections::BTreeMap;
use std::str;
use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use secrecy::ExposeSecret;
use sha2::Sha256;
use shaku::{Component, Interface};

use crate::config::Configuration;
use crate::database::DatabaseError;
use crate::email::{self, EmailParams};
use crate::user::{InsertUserParams, UserRepositoryInterface, UserStatus};

use super::{AuthEntity, AuthError, AuthRepositoryInterface, RefreshTokenResult};

type HmacSha256 = Hmac<Sha256>;

pub struct RegisterParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub reason: Option<String>,
    pub role_id: i32,
}

pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[async_trait]
pub trait AuthServiceInterface: Interface {
    async fn register(&self, params: RegisterParams) -> Result<(), AuthError>;
    async fn send_email_confirmation(&self, email: String) -> Result<(), AuthError>;
    async fn verify_email_confirmation_token(
        &self,
        token: String,
    ) -> Result<BTreeMap<String, String>, AuthError>;
    async fn confirm_email(&self, token: String) -> Result<AuthEntity, AuthError>;
    async fn login(&self, params: LoginParams) -> Result<AuthEntity, AuthError>;
    async fn refresh_token(&self, refresh_token: String) -> Result<RefreshTokenResult, AuthError>;
}

#[derive(Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct AuthService {
    #[shaku(inject)]
    _user_repository: Arc<dyn UserRepositoryInterface>,
    #[shaku(inject)]
    _auth_repository: Arc<dyn AuthRepositoryInterface>,
    _email: email::Client,
    _configuration: Configuration,
}

impl AuthService {
    pub fn generate_access_token(&self, user_id: i32) -> Result<String, AuthError> {
        let access_token_key = match HmacSha256::new_from_slice(
            self._configuration
                .access_token_secret
                .expose_secret()
                .as_bytes(),
        ) {
            Ok(key) => key,
            _ => return Err(AuthError::InternalServerError),
        };

        let mut access_token_claims = BTreeMap::new();
        access_token_claims.insert("user_id", user_id.to_string());
        access_token_claims.insert("iat", chrono::Utc::now().timestamp().to_string());
        access_token_claims.insert(
            "exp",
            (chrono::Utc::now()
                + chrono::Duration::seconds(self._configuration.access_token_expiration_seconds))
            .timestamp()
            .to_string(),
        );

        let access_token = match access_token_claims.sign_with_key(&access_token_key) {
            Ok(token) => token,
            _ => return Err(AuthError::InternalServerError),
        };

        Ok(access_token)
    }

    pub fn generate_refresh_token(&self, user_id: i32) -> Result<String, AuthError> {
        let refresh_token_key = match HmacSha256::new_from_slice(
            self._configuration
                .refresh_token_secret
                .expose_secret()
                .as_bytes(),
        ) {
            Ok(key) => key,
            _ => return Err(AuthError::InternalServerError),
        };

        let mut refresh_token_claims = BTreeMap::new();
        refresh_token_claims.insert("user_id", user_id.to_string());
        refresh_token_claims.insert("iat", chrono::Utc::now().timestamp().to_string());
        refresh_token_claims.insert(
            "exp",
            (chrono::Utc::now()
                + chrono::Duration::seconds(self._configuration.refresh_token_expiration_seconds))
            .timestamp()
            .to_string(),
        );

        let refresh_token = match refresh_token_claims.sign_with_key(&refresh_token_key) {
            Ok(token) => token,
            _ => return Err(AuthError::InternalServerError),
        };

        Ok(refresh_token)
    }

    pub fn decode_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<BTreeMap<String, String>, AuthError> {
        let key = match HmacSha256::new_from_slice(
            self._configuration
                .refresh_token_secret
                .expose_secret()
                .as_bytes(),
        ) {
            Ok(key) => key,
            _ => return Err(AuthError::InternalServerError),
        };

        let claims: BTreeMap<String, String> = match refresh_token.verify_with_key(&key) {
            Ok(claims) => claims,
            Err(_) => {
                return Err(AuthError::TokenInvalid(
                    "Authorization failed, token is invalid".into(),
                ))
            }
        };

        let expired_at: i64 = match claims["exp"].parse() {
            Ok(timestamp) => timestamp,
            _ => {
                return Err(AuthError::TokenInvalid(
                    "Authorization failed, token is invalid".into(),
                ))
            }
        };

        let now = chrono::Utc::now();
        let expired_at = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(expired_at, 0),
            chrono::Utc,
        );
        if now >= expired_at {
            return Err(AuthError::TokenExpired(
                "Token is already expired, please send a new confirmation email".into(),
            ));
        }

        Ok(claims)
    }
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn register(&self, params: RegisterParams) -> Result<(), AuthError> {
        let hash = match Argon2::default().hash_password(
            params.password.as_bytes(),
            self._configuration.password_secret.expose_secret(),
        ) {
            Ok(p) => p.serialize(),
            Err(_) => return Err(AuthError::InternalServerError),
        };

        match self
            ._user_repository
            .create(InsertUserParams {
                name: params.name.to_string(),
                email: params.email.to_string(),
                registration_reason: params.reason,
                password: hash.to_string(),
                role_id: params.role_id,
            })
            .await
        {
            Ok(id) => id,
            Err(e) => match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(AuthError::EmailAlreadyExists(
                                "Email is already registered in our system".into(),
                            ));
                        } else if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(AuthError::RoleNotFound("Role not found".into()));
                        }
                    }

                    return Err(AuthError::InternalServerError);
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        Ok(())
    }

    async fn send_email_confirmation(&self, email: String) -> Result<(), AuthError> {
        let user = match self._user_repository.find_one_by_email(email).await {
            Ok(user) => user,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::UserNotFound(
                        "User with the given email was not found".into(),
                    ))
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        if user.is_email_confirmed == true {
            return Err(AuthError::EmailAlreadyConfirmed(
                "The email is already confirmed".into(),
            ));
        }

        let key = match HmacSha256::new_from_slice(
            self._configuration
                .email_confirmation_secret
                .expose_secret()
                .as_bytes(),
        ) {
            Ok(key) => key,
            _ => return Err(AuthError::InternalServerError),
        };

        let mut claims = BTreeMap::new();
        claims.insert("user_id", user.id.to_string());
        claims.insert("iat", chrono::Utc::now().timestamp().to_string());
        claims.insert(
            "exp",
            (chrono::Utc::now()
                + chrono::Duration::seconds(
                    self._configuration.email_confirmation_expiration_seconds,
                ))
            .timestamp()
            .to_string(),
        );

        let token = match claims.sign_with_key(&key) {
            Ok(token) => token,
            _ => return Err(AuthError::InternalServerError),
        };

        let confirmation_link = format!(
            "{}/verification?token={}",
            self._configuration.dashboard_baseurl, token,
        );

        let html = format!(
            "<p>Please confirm your email address by clicking this <a href=\"{}\">link</a>",
            confirmation_link
        )
        .to_string();

        let email_params = EmailParams {
            from: "Enchiridion <noreply@stevenhansel.com>".into(),
            to: user.email,
            subject: "[Enchiridion] Please confirm your email address".into(),
            html,
        };
        if let Err(_) = self._email.send(email_params).await {
            return Err(AuthError::InternalServerError);
        }

        Ok(())
    }

    async fn verify_email_confirmation_token(
        &self,
        token: String,
    ) -> Result<BTreeMap<String, String>, AuthError> {
        let key = match HmacSha256::new_from_slice(
            self._configuration
                .email_confirmation_secret
                .expose_secret()
                .as_bytes(),
        ) {
            Ok(key) => key,
            _ => return Err(AuthError::InternalServerError),
        };

        let claims: BTreeMap<String, String> = match token.verify_with_key(&key) {
            Ok(claims) => claims,
            Err(_) => {
                return Err(AuthError::TokenInvalid(
                    "Authorization failed, token is invalid".into(),
                ))
            }
        };

        let expired_at: i64 = match claims["exp"].parse() {
            Ok(timestamp) => timestamp,
            _ => {
                return Err(AuthError::TokenInvalid(
                    "Authorization failed, token is invalid".into(),
                ))
            }
        };

        let now = chrono::Utc::now();
        let expired_at = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(expired_at, 0),
            chrono::Utc,
        );
        if now >= expired_at {
            return Err(AuthError::TokenExpired(
                "Token is already expired, please send a new confirmation email".into(),
            ));
        }

        Ok(claims)
    }

    async fn confirm_email(&self, token: String) -> Result<AuthEntity, AuthError> {
        let claims = match self.verify_email_confirmation_token(token).await {
            Ok(claims) => claims,
            Err(e) => return Err(e),
        };

        let user_id: i32 = match claims["user_id"].parse() {
            Ok(id) => id,
            Err(_) => return Err(AuthError::InternalServerError),
        };
        if let Err(e) = self._user_repository.confirm_email(user_id).await {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::UserNotFound("User not found".into()))
                }
                _ => return Err(AuthError::InternalServerError),
            }
        }

        let entity = match self
            ._auth_repository
            .find_one_auth_entity_by_id(user_id)
            .await
        {
            Ok(entity) => entity,
            Err(e) => match e {
                _ => return Err(AuthError::InternalServerError),
            },
        };

        let access_token = self.generate_access_token(user_id)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        if let Err(_) = self
            ._auth_repository
            .set_user_refresh_token(user_id, refresh_token.clone())
            .await
        {
            return Err(AuthError::InternalServerError);
        }

        Ok(AuthEntity {
            entity,
            access_token,
            refresh_token,
        })
    }

    async fn login(&self, params: LoginParams) -> Result<AuthEntity, AuthError> {
        let user = match self._user_repository.find_one_by_email(params.email).await {
            Ok(user) => user,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::AuthenticationFailed(
                        "Authentication failed, Invalid email or password".into(),
                    ))
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        if user.is_email_confirmed == false {
            return Err(AuthError::UserNotVerified(
                "Email is not confirmed yet".into(),
            ));
        }
        if user.status == UserStatus::WaitingForApproval {
            return Err(AuthError::UserNotVerified(
                "User is not approved yet by admin".into(),
            ));
        }
        if user.status == UserStatus::Rejected {
            return Err(AuthError::UserNotVerified(
                "User registration is rejected by admin".into(),
            ));
        }

        let password_str = match str::from_utf8(&user.password) {
            Ok(v) => v,
            _ => return Err(AuthError::InternalServerError),
        };
        let parsed_hash = match PasswordHash::new(password_str) {
            Ok(hash) => hash,
            _ => return Err(AuthError::InternalServerError),
        };
        let is_password_match = Argon2::default()
            .verify_password(params.password.as_bytes(), &parsed_hash)
            .is_ok();
        if is_password_match == false {
            return Err(AuthError::AuthenticationFailed(
                "Authentication failed, Invalid email or password".into(),
            ));
        }

        let entity = match self
            ._auth_repository
            .find_one_auth_entity_by_email(user.email)
            .await
        {
            Ok(entity) => entity,
            Err(e) => match e {
                _ => return Err(AuthError::InternalServerError),
            },
        };

        let access_token = self.generate_access_token(user.id)?;
        let refresh_token = self.generate_refresh_token(user.id)?;

        if let Err(_) = self
            ._auth_repository
            .set_user_refresh_token(user.id, refresh_token.clone())
            .await
        {
            return Err(AuthError::InternalServerError);
        }

        Ok(AuthEntity {
            entity,
            access_token,
            refresh_token,
        })
    }

    async fn refresh_token(&self, refresh_token: String) -> Result<RefreshTokenResult, AuthError> {
        let claims = self.decode_refresh_token(refresh_token)?;
        let user_id: i32 = match claims["user_id"].parse() {
            Ok(id) => id,
            Err(_) => return Err(AuthError::InternalServerError),
        };

        let access_token = self.generate_access_token(user_id)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(RefreshTokenResult {
            access_token,
            refresh_token,
        })
    }
}
