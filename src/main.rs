use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::{env, process};

use aws_config::meta::region::RegionProviderChain;

use enchiridion_api::announcement::{
    AnnouncementQueue, AnnouncementRepository, AnnouncementService,
};
use enchiridion_api::cloud_storage::s3::S3Adapter;
use enchiridion_api::device::{DeviceRepository, DeviceService};
use enchiridion_api::floor::{FloorRepository, FloorService};
use enchiridion_api::request::{RequestRepository, RequestService};
use secrecy::ExposeSecret;
use sqlx::PgPool;

use enchiridion_api::startup::run;

use enchiridion_api::config::Configuration;
use enchiridion_api::{cloud_storage, email};

use enchiridion_api::auth::{
    AuthRepository, AuthService, AuthServiceInterface, SeedDefaultUserError,
};
use enchiridion_api::building::{BuildingRepository, BuildingService};
use enchiridion_api::role::RoleService;
use enchiridion_api::user::{UserRepository, UserService};

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

    let s3_credentials = aws_sdk_s3::Credentials::new(
        config.aws_access_key_id.expose_secret(),
        config.aws_access_key_secret.expose_secret(),
        None,
        None,
        "enchiridion_api",
    );

    let region_provider = RegionProviderChain::first_try("ap-southeast-1");
    let aws_configuration = aws_config::from_env()
        .credentials_provider(s3_credentials)
        .region(region_provider)
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&aws_configuration);
    let s3_adapter = S3Adapter::new(s3_client, config.aws_s3_bucket_name.clone());
    let cloud_storage = cloud_storage::Client::new(Box::new(s3_adapter));

    let building_repository = Arc::new(BuildingRepository::new(pool.clone()));
    let user_repository = Arc::new(UserRepository::new(pool.clone()));
    let auth_repository = Arc::new(AuthRepository::new(
        pool.clone(),
        redis_connection.clone(),
        config.clone(),
    ));
    let floor_repository = Arc::new(FloorRepository::new(pool.clone()));
    let device_repository = Arc::new(DeviceRepository::new(pool.clone()));
    let announcement_repository = Arc::new(AnnouncementRepository::new(pool.clone()));
    let request_repository = Arc::new(RequestRepository::new(pool.clone()));

    let announcement_queue = Arc::new(AnnouncementQueue::new(redis_connection.clone()));

    let role_service = Arc::new(RoleService::new());
    let building_service = Arc::new(BuildingService::new(building_repository.clone()));
    let user_service = Arc::new(UserService::new(user_repository.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_repository.clone(),
        auth_repository.clone(),
        role_service.clone(),
        email_client,
        config.clone(),
    ));
    let floor_service = Arc::new(FloorService::new(floor_repository.clone()));
    let device_service = Arc::new(DeviceService::new(device_repository.clone()));
    let request_service = Arc::new(RequestService::new(
        request_repository.clone(),
        announcement_repository.clone(),
        announcement_queue.clone(),
        auth_repository.clone(),
    ));
    let announcement_service = Arc::new(AnnouncementService::new(
        announcement_repository.clone(),
        request_service.clone(),
        cloud_storage,
    ));

    auth_service
        .seed_default_user()
        .await
        .unwrap_or_else(|e| match e {
            SeedDefaultUserError::InternalServerError => process::exit(1),
            _ => {}
        });

    let listener = TcpListener::bind(config.address)?;
    run(
        listener,
        role_service.clone(),
        building_service.clone(),
        user_service.clone(),
        auth_service.clone(),
        floor_service.clone(),
        device_service.clone(),
        request_service.clone(),
        announcement_service.clone(),
    )?
    .await
}
