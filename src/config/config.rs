use std::env;
use std::error;
use std::fmt;
use std::num;

use secrecy::Secret;

#[derive(Debug)]
pub enum ConfigError {
    DevelopmentConfigError,
    DeploymentConfigError,
    ParsingError,
}

impl error::Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::DevelopmentConfigError => write!(f, "An error occurred in the .env file"),
            ConfigError::DeploymentConfigError => {
                write!(f, "An error occured in the system environment variable")
            }
            ConfigError::ParsingError => {
                write!(f, "Something went wrong when parsing an integer")
            }
        }
    }
}

impl From<dotenvy::Error> for ConfigError {
    fn from(_: dotenvy::Error) -> Self {
        ConfigError::DevelopmentConfigError
    }
}

impl From<env::VarError> for ConfigError {
    fn from(_: env::VarError) -> Self {
        ConfigError::DeploymentConfigError
    }
}

impl From<num::ParseIntError> for ConfigError {
    fn from(_: num::ParseIntError) -> Self {
        ConfigError::ParsingError
    }
}

#[derive(Clone)]
pub struct Configuration {
    pub address: String,

    pub password_secret: Secret<String>,
    pub email_confirmation_secret: Secret<String>,
    pub forgot_password_secret: Secret<String>,
    pub access_token_secret: Secret<String>,
    pub refresh_token_secret: Secret<String>,

    pub email_confirmation_expiration_seconds: i64,
    pub forgot_password_expiration_seconds: i64,
    pub access_token_expiration_seconds: i64,
    pub refresh_token_expiration_seconds: i64,

    pub database_url: Secret<String>,
    pub redis_url: Secret<String>,

    pub mailgun_baseurl: String,
    pub mailgun_domain: Secret<String>,
    pub mailgun_api_key: Secret<String>,

    pub aws_access_key_id: Secret<String>,
    pub aws_access_key_secret: Secret<String>,
    pub aws_s3_bucket_name: String,
    pub aws_s3_presigning_url_expiration_seconds: i32,

    pub dashboard_baseurl: String,
}

impl Configuration {
    pub fn with_env_file() -> Result<Configuration, ConfigError> {
        Ok(Configuration {
            address: dotenvy::var("ADDRESS")?,

            password_secret: Secret::new(dotenvy::var("PASSWORD_SECRET")?),
            email_confirmation_secret: Secret::new(dotenvy::var("EMAIL_CONFIRMATION_SECRET")?),
            forgot_password_secret: Secret::new(dotenvy::var("FORGOT_PASSWORD_SECRET")?),
            access_token_secret: Secret::new(dotenvy::var("ACCESS_TOKEN_SECRET")?),
            refresh_token_secret: Secret::new(dotenvy::var("REFRESH_TOKEN_SECRET")?),

            email_confirmation_expiration_seconds: dotenvy::var(
                "EMAIL_CONFIRMATION_EXPIRATION_SECONDS",
            )?
            .parse()?,
            forgot_password_expiration_seconds: dotenvy::var("FORGOT_PASSWORD_EXPIRATION_SECONDS")?
                .parse()?,
            access_token_expiration_seconds: dotenvy::var("ACCESS_TOKEN_EXPIRATION_SECONDS")?
                .parse()?,
            refresh_token_expiration_seconds: dotenvy::var("REFRESH_TOKEN_EXPIRATION_SECONDS")?
                .parse()?,

            database_url: Secret::new(dotenvy::var("DATABASE_URL")?),
            redis_url: Secret::new(dotenvy::var("REDIS_URL")?),

            mailgun_baseurl: dotenvy::var("MAILGUN_BASEURL")?,
            mailgun_domain: Secret::new(dotenvy::var("MAILGUN_DOMAIN")?),
            mailgun_api_key: Secret::new(dotenvy::var("MAILGUN_API_KEY")?),

            aws_access_key_id: Secret::new(dotenvy::var("AWS_ACCESS_KEY_ID")?),
            aws_access_key_secret: Secret::new(dotenvy::var("AWS_ACCESS_KEY_SECRET")?),
            aws_s3_bucket_name: dotenvy::var("AWS_S3_BUCKET_NAME")?,
            aws_s3_presigning_url_expiration_seconds: dotenvy::var("AWS_S3_PRESIGNING_URL_EXPIRATION_SECONDS")?.parse()?,

            dashboard_baseurl: dotenvy::var("DASHBOARD_BASEURL")?,
        })
    }

    pub fn with_os_environment_vars() -> Result<Configuration, ConfigError> {
        Ok(Configuration {
            address: env::var("ADDRESS")?,

            password_secret: Secret::new(env::var("PASSWORD_SECRET")?),
            email_confirmation_secret: Secret::new(env::var("EMAIL_CONFIRMATION_SECRET")?),
            forgot_password_secret: Secret::new(env::var("FORGOT_PASSWORD_SECRET")?),
            access_token_secret: Secret::new(env::var("ACCESS_TOKEN_SECRET")?),
            refresh_token_secret: Secret::new(env::var("REFRESH_TOKEN_SECRET")?),

            email_confirmation_expiration_seconds: env::var(
                "EMAIL_CONFIRMATION_EXPIRATION_SECONDS",
            )?
            .parse()?,
            forgot_password_expiration_seconds: env::var("FORGOT_PASSWORD_EXPIRATION_SECONDS")?
                .parse()?,
            access_token_expiration_seconds: env::var("ACCESS_TOKEN_EXPIRATION_SECONDS")?
                .parse()?,
            refresh_token_expiration_seconds: env::var("REFRESH_TOKEN_EXPIRATION_SECONDS")?
                .parse()?,

            database_url: Secret::new(env::var("DATABASE_URL")?),
            redis_url: Secret::new(env::var("REDIS_URL")?),

            mailgun_baseurl: env::var("MAILGUN_BASEURL")?,
            mailgun_domain: Secret::new(env::var("MAILGUN_DOMAIN")?),
            mailgun_api_key: Secret::new(env::var("MAILGUN_API_KEY")?),

            aws_access_key_id: Secret::new(env::var("AWS_ACCESS_KEY_ID")?),
            aws_access_key_secret: Secret::new(env::var("AWS_ACCESS_KEY_SECRET")?),
            aws_s3_bucket_name: env::var("AWS_S3_BUCKET_NAME")?,
            aws_s3_presigning_url_expiration_seconds: env::var("AWS_S3_PRESIGNING_URL_EXPIRATION_SECONDS")?.parse()?,


            dashboard_baseurl: env::var("DASHBOARD_BASEURL")?,
        })
    }
}
