use std::collections::BTreeMap;
use std::str;
use std::sync::Arc;

use actix_web::cookie::Cookie;
use argon2::{password_hash::PasswordHasher, Argon2};
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use rand::distributions::{Alphanumeric, DistString};
use secrecy::ExposeSecret;
use sha2::Sha256;

use crate::{
    config::Configuration,
    database::DatabaseError,
    email::{self, EmailParams},
    features::{
        role::{ApplicationPermission, RoleServiceInterface},
        user::{InsertRawUserParams, InsertUserParams, UserRepositoryInterface, UserStatus},
    },
};

use super::{
    AuthEntity, AuthError, AuthRepositoryInterface, AuthenticateError, ChangePasswordError,
    RefreshTokenResult, SeedDefaultUserError, UserAuthEntity,
};

type HmacSha256 = Hmac<Sha256>;

pub struct RegisterParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub reason: Option<String>,
    pub role: String,
    pub building_id: i32,
}

pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[async_trait]
pub trait AuthServiceInterface {
    fn generate_access_token(
        &self,
        user_id: i32,
        role: String,
        status: UserStatus,
    ) -> Result<String, AuthError>;
    fn generate_refresh_token(&self, user_id: i32) -> Result<String, AuthError>;
    fn decode_access_token(
        &self,
        access_token: String,
    ) -> Result<BTreeMap<String, String>, AuthError>;
    fn decode_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<BTreeMap<String, String>, AuthError>;
    async fn register(&self, params: RegisterParams) -> Result<(), AuthError>;
    async fn send_email_confirmation(&self, email: String) -> Result<(), AuthError>;
    async fn verify_email_confirmation_token(
        &self,
        token: String,
    ) -> Result<BTreeMap<String, String>, AuthError>;
    async fn confirm_email(&self, token: String) -> Result<AuthEntity, AuthError>;
    async fn login(&self, params: LoginParams) -> Result<AuthEntity, AuthError>;
    async fn refresh_token(&self, refresh_token: String) -> Result<RefreshTokenResult, AuthError>;
    async fn me(&self, user_id: i32) -> Result<UserAuthEntity, AuthError>;
    async fn logout(&self, user_id: i32) -> Result<(), AuthError>;
    async fn change_password(
        &self,
        user_id: i32,
        old_password: String,
        new_password: String,
    ) -> Result<(), ChangePasswordError>;
    async fn seed_default_user(&self) -> Result<(), SeedDefaultUserError>;
    async fn authenticate(
        &self,
        cookie: Option<Cookie<'static>>,
        permission: Option<ApplicationPermission>,
        require_email_confirmed: Option<bool>,
        status: Option<UserStatus>,
    ) -> Result<i32, AuthenticateError>;
}

pub struct AuthService {
    _user_repository: Arc<dyn UserRepositoryInterface + Send + Sync + 'static>,
    _auth_repository: Arc<dyn AuthRepositoryInterface + Send + Sync + 'static>,
    _role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    _email: email::Client,
    _configuration: Configuration,
}

impl AuthService {
    pub fn new(
        _user_repository: Arc<dyn UserRepositoryInterface + Send + Sync + 'static>,
        _auth_repository: Arc<dyn AuthRepositoryInterface + Send + Sync + 'static>,
        _role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
        _email: email::Client,
        _configuration: Configuration,
    ) -> AuthService {
        AuthService {
            _user_repository,
            _auth_repository,
            _role_service,
            _email,
            _configuration,
        }
    }
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    fn generate_access_token(
        &self,
        user_id: i32,
        role: String,
        status: UserStatus,
    ) -> Result<String, AuthError> {
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
        access_token_claims.insert("role", role);
        access_token_claims.insert("status", status.value().to_string());
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

    fn generate_refresh_token(&self, user_id: i32) -> Result<String, AuthError> {
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

    fn decode_access_token(
        &self,
        access_token: String,
    ) -> Result<BTreeMap<String, String>, AuthError> {
        let key = match HmacSha256::new_from_slice(
            self._configuration
                .access_token_secret
                .expose_secret()
                .as_bytes(),
        ) {
            Ok(key) => key,
            _ => return Err(AuthError::InternalServerError),
        };

        let claims: BTreeMap<String, String> = match access_token.verify_with_key(&key) {
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

    fn decode_refresh_token(
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

    async fn register(&self, params: RegisterParams) -> Result<(), AuthError> {
        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        let hash = match Argon2::default().hash_password(params.password.as_bytes(), &salt) {
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
                password_salt: salt,
                role: params.role,
                building_id: params.building_id,
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
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::UserNotFound("User not found".into()))
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        let access_token = self.generate_access_token(
            entity.id,
            entity.role.value.to_string(),
            entity.user_status.clone(),
        )?;
        let refresh_token = self.generate_refresh_token(entity.id)?;

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

        if user.status == UserStatus::Rejected {
            return Err(AuthError::UserNotVerified(
                "User registration is rejected by admin".into(),
            ));
        }

        let input_password_hash = match Argon2::default()
            .hash_password(params.password.as_bytes(), &user.password_salt)
        {
            Ok(p) => p,
            Err(_) => return Err(AuthError::InternalServerError),
        };

        let original_password_hash = match str::from_utf8(&user.password) {
            Ok(v) => v,
            Err(_) => return Err(AuthError::InternalServerError),
        };

        if input_password_hash.to_string() != original_password_hash {
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
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::UserNotFound("User not found".into()))
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        let access_token = self.generate_access_token(
            entity.id,
            entity.role.value.to_string(),
            entity.user_status.clone(),
        )?;
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
        let claims = self.decode_refresh_token(refresh_token.clone())?;
        let user_id: i32 = match claims["user_id"].parse() {
            Ok(id) => id,
            Err(_) => return Err(AuthError::InternalServerError),
        };

        let cached_refresh_token = match self._auth_repository.get_user_refresh_token(user_id).await
        {
            Ok(token) => token,
            Err(_) => {
                return Err(AuthError::AuthenticationFailed(
                    "Authentication failed, unable to refresh token".into(),
                ))
            }
        };

        if cached_refresh_token != refresh_token {
            return Err(AuthError::AuthenticationFailed(
                "Authentication failed, unable to refresh token".into(),
            ));
        }

        let entity = match self
            ._auth_repository
            .find_one_auth_entity_by_id(user_id)
            .await
        {
            Ok(entity) => entity,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::UserNotFound("User not found".into()))
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        let access_token = self.generate_access_token(
            entity.id,
            entity.role.value.to_string(),
            entity.user_status.clone(),
        )?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        if let Err(_) = self
            ._auth_repository
            .set_user_refresh_token(user_id, refresh_token.clone())
            .await
        {
            return Err(AuthError::InternalServerError);
        }

        Ok(RefreshTokenResult {
            access_token,
            refresh_token,
        })
    }

    async fn me(&self, user_id: i32) -> Result<UserAuthEntity, AuthError> {
        let entity = match self
            ._auth_repository
            .find_one_auth_entity_by_id(user_id)
            .await
        {
            Ok(entity) => entity,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(AuthError::UserNotFound("User not found".into()))
                }
                _ => return Err(AuthError::InternalServerError),
            },
        };

        Ok(entity)
    }

    async fn logout(&self, user_id: i32) -> Result<(), AuthError> {
        if let Err(_) = self._user_repository.find_one_by_id(user_id).await {
            return Err(AuthError::UserNotFound(
                "User with the given id is not found".into(),
            ));
        }

        if let Err(_) = self
            ._auth_repository
            .delete_user_refresh_token(user_id)
            .await
        {
            return Err(AuthError::InternalServerError);
        }

        Ok(())
    }

    async fn change_password(
        &self,
        user_id: i32,
        old_password: String,
        new_password: String,
    ) -> Result<(), ChangePasswordError> {
        let user = match self._user_repository.find_one_by_id(user_id).await {
            Ok(user) => user,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(ChangePasswordError::UserNotFound(
                        "Unable to find user in the system",
                    ))
                }
                _ => return Err(ChangePasswordError::InternalServerError),
            },
        };

        let input_old_password_hash =
            match Argon2::default().hash_password(old_password.as_bytes(), &user.password_salt) {
                Ok(p) => p,
                Err(_) => return Err(ChangePasswordError::InternalServerError),
            };

        let original_old_password_hash = match str::from_utf8(&user.password) {
            Ok(v) => v,
            Err(_) => return Err(ChangePasswordError::InternalServerError),
        };

        if input_old_password_hash.to_string() != original_old_password_hash {
            return Err(ChangePasswordError::UserInvalidOldPassword(
                "Unable to change password due to invalid old password",
            ));
        }

        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        let hash = match Argon2::default().hash_password(new_password.as_bytes(), &salt) {
            Ok(p) => p.serialize(),
            Err(_) => return Err(ChangePasswordError::InternalServerError),
        };

        if let Err(_) = self
            ._user_repository
            .update_password(user_id, hash.to_string(), salt)
            .await
        {
            return Err(ChangePasswordError::InternalServerError);
        }

        Ok(())
    }

    async fn seed_default_user(&self) -> Result<(), SeedDefaultUserError> {
        match self
            ._user_repository
            .find_one_by_email(
                self._configuration
                    .default_user_email
                    .expose_secret()
                    .clone(),
            )
            .await
        {
            Ok(_) => {
                return Ok(()) 
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => {}
                _ => return Err(SeedDefaultUserError::InternalServerError),
            },
        };

        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        let hash = match Argon2::default().hash_password(
            self._configuration
                .default_user_password
                .expose_secret()
                .as_bytes(),
            &salt,
        ) {
            Ok(p) => p.serialize(),
            Err(_) => return Err(SeedDefaultUserError::InternalServerError),
        };

        if let Err(_) = self
            ._user_repository
            .raw_create(InsertRawUserParams {
                name: self
                    ._configuration
                    .default_user_name
                    .expose_secret()
                    .clone(),
                email: self
                    ._configuration
                    .default_user_email
                    .expose_secret()
                    .clone(),
                password: hash.to_string(),
                password_salt: salt,
                role: self._configuration.default_user_role.clone(),
                is_email_confirmed: true,
                status: UserStatus::Approved,
                registration_reason: None,
                building_id: 1,
            })
            .await
        {
            return Err(SeedDefaultUserError::InternalServerError);
        }

        Ok(())
    }

    async fn authenticate(
        &self,
        cookie: Option<Cookie<'static>>,
        permission: Option<ApplicationPermission>,
        require_email_confirmed: Option<bool>,
        status: Option<UserStatus>,
    ) -> Result<i32, AuthenticateError> {
        let access_token = match cookie {
            Some(cookie) => cookie.value().to_string(),
            None => {
                return Err(AuthenticateError::AuthenticationFailed(
                    "Authentication failed, Token expired or invalid",
                ))
            }
        };

        let claims = match self.decode_access_token(access_token) {
            Ok(claims) => claims,
            Err(e) => match e {
                AuthError::TokenInvalid(_) => {
                    return Err(AuthenticateError::AuthenticationFailed(
                        "Authentication failed, Token expired or invalid",
                    ))
                }
                AuthError::TokenExpired(_) => {
                    return Err(AuthenticateError::AuthenticationFailed(
                        "Authentication failed, Token expired or invalid",
                    ))
                }
                _ => return Err(AuthenticateError::InternalServerError),
            },
        };

        let user_id = match claims["user_id"].parse::<i32>() {
            Ok(id) => id,
            Err(_) => {
                return Err(AuthenticateError::AuthenticationFailed(
                    "Authentication failed, Token expired or invalid",
                ))
            }
        };

        let user = match self._user_repository.find_one_by_id(user_id).await {
            Ok(user) => user,
            Err(_) => {
                return Err(AuthenticateError::InternalServerError);
            }
        };

        if let Some(permission) = permission {
            let role = match self
                ._role_service
                .get_role_by_value(claims["role"].as_str())
            {
                Ok(role) => role,
                Err(_) => return Err(AuthenticateError::InternalServerError),
            };
            let role_permissions: Vec<&'static str> =
                role.permissions.into_iter().map(|p| p.value).collect();

            if !role_permissions.contains(&permission.value()) {
                return Err(AuthenticateError::ForbiddenPermission(
                    "User doesn't have the permission to access the designated route",
                ));
            }
        }

        if let Some(require_email_confirmed) = require_email_confirmed {
            if user.is_email_confirmed != require_email_confirmed {
                return Err(AuthenticateError::ForbiddenPermission(
                    "User email has not been confirmed yet",
                ));
            }
        }

        if let Some(status) = status {
            if user.status != status {
                return Err(AuthenticateError::ForbiddenPermission(
                    "User has been approved yet by admin",
                ));
            }
        }

        Ok(user_id)
    }
}
