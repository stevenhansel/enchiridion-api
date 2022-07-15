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
}

impl Configuration {
    pub fn for_development() -> Result<Configuration, ConfigError> {
        Ok(Configuration {
            address: dotenvy::var("ADDRESS")?,
            password_secret: Secret::new(dotenvy::var("PASSWORD_SECRET")?),
            database_url: Secret::new(dotenvy::var("DATABASE_URL")?),
        })
    }

    pub fn for_deployment() -> Result<Configuration, ConfigError> {
        Ok(Configuration {
            address: env::var("ADDRESS")?,
            password_secret: Secret::new(env::var("PASSWORD_SECRET")?),
            database_url: Secret::new(env::var("DATABASE_URL")?),
        })
    }
}
