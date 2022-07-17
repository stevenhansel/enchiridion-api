use std::env;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use secrecy::ExposeSecret;
use sqlx::PgPool;

use enchiridion_api::startup::run;

use enchiridion_api::config::Configuration;
use enchiridion_api::email;

use enchiridion_api::auth::{AuthRepository, AuthService};
use enchiridion_api::building::{BuildingRepository, BuildingService};
use enchiridion_api::role::{RoleRepository, RoleService};
use enchiridion_api::user::{UserService, UserRepository};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let environment = match env::var("ENVIRONMENT") {
        Ok(env) => env,
        Err(_) => "development".into(),
    };
    let config = match environment.as_str() {
        "production" => {
            Configuration::with_os_environment_vars().expect("Failed to read configuration")
        }
        _ => Configuration::with_env_file().expect("Failed to read configuration"),
    };

    let pool = PgPool::connect(config.database_url.expose_secret())
        .await
        .unwrap();

    let redis_instance = redis::Client::open(config.redis_url.expose_secret().to_string())
        .expect("Failed to create redis instance");
    let redis_connection = Arc::new(Mutex::new(
        redis_instance
            .get_connection()
            .expect("Failed to open redis connection"),
    ));

    let mailgun_adapter = email::MailgunAdapter::new(
        config.mailgun_baseurl.clone(),
        config.mailgun_domain.clone(),
        config.mailgun_api_key.clone(),
    );
    let email_client = email::Client::new(Box::new(mailgun_adapter));

    let role_repository = Arc::new(RoleRepository::new(pool.clone()));
    let building_repository = Arc::new(BuildingRepository::new(pool.clone()));
    let user_repository = Arc::new(UserRepository::new(pool.clone()));
    let auth_repository = Arc::new(AuthRepository::new(
        pool.clone(),
        redis_connection.clone(),
        config.clone(),
    ));

    let role_service = Arc::new(RoleService::new(role_repository.clone()));
    let building_service = Arc::new(BuildingService::new(building_repository.clone()));
    let user_service = Arc::new(UserService::new(user_repository.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_repository.clone(),
        auth_repository.clone(),
        email_client,
        config.clone(),
    ));

    let listener = TcpListener::bind(config.address)?;
    run(
        listener,
        role_service.clone(),
        building_service.clone(),
        user_service.clone(),
        auth_service.clone(),
    )?
    .await
}
