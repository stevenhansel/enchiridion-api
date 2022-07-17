use std::env;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use secrecy::ExposeSecret;
use sqlx::PgPool;

use enchiridion_api::startup::run;

use enchiridion_api::config::Configuration;
use enchiridion_api::container::Container;
use enchiridion_api::email;

use enchiridion_api::auth::{
    AuthRepository, AuthRepositoryParameters, AuthService, AuthServiceParameters,
};
use enchiridion_api::building::{BuildingRepository, BuildingService};
use enchiridion_api::role::{RoleRepository, RoleService};
use enchiridion_api::user::{UserRepository, UserRepositoryParameters};

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

    let mailgun_adapter = email::MailgunAdapter::new(
        config.mailgun_baseurl.clone(),
        config.mailgun_domain.clone(),
        config.mailgun_api_key.clone(),
    );
    let email_client = email::Client::new(Box::new(mailgun_adapter));

    let role_repository = Arc::new(RoleRepository::new(pool.clone()));
    let building_repository = Arc::new(BuildingRepository::new(pool.clone()));

    let role_service = Arc::new(RoleService::new(role_repository.clone()));
    let building_service = Arc::new(BuildingService::new(building_repository.clone()));

    let container = Container::builder()
        .with_component_parameters::<UserRepository>(UserRepositoryParameters { _db: pool.clone() })
        .with_component_parameters::<AuthRepository>(AuthRepositoryParameters {
            _db: pool.clone(),
            _redis: Arc::new(Mutex::new(
                redis_instance
                    .get_connection()
                    .expect("Failed to open redis connection"),
            )),
            _configuration: config.clone(),
        })
        .with_component_parameters::<AuthService>(AuthServiceParameters {
            _configuration: config.clone(),
            _email: email_client,
        })
        .build();

    let listener = TcpListener::bind(config.address)?;
    run(
        listener,
        container,
        role_service.clone(),
        building_service.clone(),
    )?
    .await
}
