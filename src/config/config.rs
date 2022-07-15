use std::env;
use std::error;
use std::fmt;

use ::secrecy::Secret;

#[derive(Debug)]
pub enum ConfigError {
    DevelopmentConfigError,
    DeploymentConfigError,
}

impl error::Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::DevelopmentConfigError => write!(f, "An error occurred in the .env file"),
            ConfigError::DeploymentConfigError => {
                write!(f, "An error occured in the system environment variable")
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

#[derive(Clone)]
pub struct Configuration {
    pub address: String,

    pub password_secret: Secret<String>,

    pub database_url: Secret<String>,

    pub mailgun_baseurl: String,
    pub mailgun_domain: Secret<String>,
    pub mailgun_api_key: Secret<String>,
}

impl Configuration {
    pub fn for_development() -> Result<Configuration, ConfigError> {
        Ok(Configuration {
            address: dotenvy::var("ADDRESS")?,

            password_secret: Secret::new(dotenvy::var("PASSWORD_SECRET")?),

            database_url: Secret::new(dotenvy::var("DATABASE_URL")?),

            mailgun_baseurl: dotenvy::var("MAILGUN_BASEURL")?,
            mailgun_domain: Secret::new(dotenvy::var("MAILGUN_DOMAIN")?),
            mailgun_api_key: Secret::new(dotenvy::var("MAILGUN_API_KEY")?),
        })
    }

    pub fn for_deployment() -> Result<Configuration, ConfigError> {
        Ok(Configuration {
            address: env::var("ADDRESS")?,

            password_secret: Secret::new(env::var("PASSWORD_SECRET")?),

            database_url: Secret::new(env::var("DATABASE_URL")?),

            mailgun_baseurl: env::var("MAILGUN_BASEURL")?,
            mailgun_domain: Secret::new(env::var("MAILGUN_DOMAIN")?),
            mailgun_api_key: Secret::new(env::var("MAILGUN_API_KEY")?),
        })
    }
}
