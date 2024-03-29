use std::net::TcpListener;
use std::sync::Arc;
use std::{env, process};

use enchiridion_api::cloud_storage::LocalAdapter;
use enchiridion_api::features::livestream::repository::LivestreamRepository;
use enchiridion_api::features::livestream::service::LivestreamService;
use enchiridion_api::features::media::repository::MediaRepository;
use enchiridion_api::features::media::service::MediaService;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use enchiridion_api::{
    cloud_storage,
    config::Configuration,
    email,
    features::{
        announcement::{AnnouncementQueue, AnnouncementRepository, AnnouncementService},
        auth::{AuthRepository, AuthService},
        building::{BuildingRepository, BuildingService},
        device::{DeviceRepository, DeviceService},
        floor::{FloorRepository, FloorService},
        request::{RequestRepository, RequestService},
        role::RoleService,
        user::{UserRepository, UserService},
        AuthServiceInterface, DeviceServiceInterface,
    },
    startup::run,
};

#[actix_web::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let environment = match env::var("ENVIRONMENT") {
        Ok(env) => env,
        Err(_) => "development".into(),
    };
    let config = match environment.as_str() {
        "production" => {
            Configuration::with_os_environment_vars().expect("[error] Failed to read configuration")
        }
        _ => Configuration::with_env_file().expect("[error] Failed to read configuration"),
    };

    let pool = PgPool::connect(config.database_url.expose_secret())
        .await
        .unwrap();

    let redis_config =
        deadpool_redis::Config::from_url(config.redis_url.expose_secret().to_string());
    let redis_pool = redis_config
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("[error] Failed to open redis connection");

    let mailgun_adapter = email::MailgunAdapter::new(
        config.mailgun_baseurl.clone(),
        config.mailgun_domain.clone(),
        config.mailgun_api_key.clone(),
    );
    let email_client = email::Client::new(Box::new(mailgun_adapter));

    let local_adapter = LocalAdapter::new(config.static_base_url.clone());
    let cloud_storage = cloud_storage::Client::new(Box::new(local_adapter));

    let building_repository = Arc::new(BuildingRepository::new(pool.clone()));
    let user_repository = Arc::new(UserRepository::new(pool.clone()));
    let auth_repository = Arc::new(AuthRepository::new(
        pool.clone(),
        redis_pool.clone(),
        config.clone(),
    ));
    let floor_repository = Arc::new(FloorRepository::new(pool.clone()));
    let device_repository = Arc::new(DeviceRepository::new(pool.clone(), redis_pool.clone()));
    let announcement_repository = Arc::new(AnnouncementRepository::new(pool.clone()));
    let request_repository = Arc::new(RequestRepository::new(pool.clone()));
    let livestream_repository = Arc::new(LivestreamRepository::new(pool.clone()));
    let media_repository = Arc::new(MediaRepository::new(pool.clone()));

    let announcement_queue = Arc::new(AnnouncementQueue::new(redis_pool.clone()));

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
    let device_service = Arc::new(DeviceService::new(
        device_repository.clone(),
        announcement_queue.clone(),
    ));
    let request_service = Arc::new(RequestService::new(
        announcement_queue.clone(),
        request_repository.clone(),
        announcement_repository.clone(),
        auth_repository.clone(),
        device_repository.clone(),
    ));
    let announcement_service = Arc::new(AnnouncementService::new(
        announcement_repository.clone(),
        announcement_queue.clone(),
        request_service.clone(),
        cloud_storage,
    ));
    let livestream_service = Arc::new(LivestreamService::new(livestream_repository));

    // TODO: Make cloud storage Arc so that it can be cloneable
    let local_adapter = LocalAdapter::new(config.static_base_url.clone());
    let cloud_storage = cloud_storage::Client::new(Box::new(local_adapter));

    let media_service = Arc::new(MediaService::new(media_repository, cloud_storage));

    auth_service.seed_default_user().await.unwrap_or_else(|e| {
        println!("Something when wrong when seeding the default user: {}", e);
        process::exit(1);
    });

    device_service
        .synchronize_device_status()
        .await
        .unwrap_or_else(|e| {
            println!(
                "Something when wrong when synchronizing the device status: {:?}",
                e
            );
            process::exit(1);
        });

    run(
        TcpListener::bind(config.address)?,
        redis_pool.clone(),
        role_service.clone(),
        building_service.clone(),
        user_service.clone(),
        auth_service.clone(),
        floor_service.clone(),
        device_service.clone(),
        request_service.clone(),
        announcement_service.clone(),
        livestream_service.clone(),
        media_service.clone(),
    )
    .await
}
