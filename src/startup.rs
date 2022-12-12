use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use actix::{Actor, Recipient};
use device_status::socket::{StatusMessage, StatusSocketServer};
use tokio::sync::{broadcast, mpsc};

use crate::features::livestream::definition::{LivestreamDeviceMap, LivestreamSessionMap};
use crate::features::livestream::service::LivestreamServiceInterface;
use crate::features::livestream::socket::LivestreamSocketServer;
use crate::features::{device_status, livestream};
use crate::shutdown::Shutdown;
use crate::{http::WebServer, scheduler};

use crate::features::{
    announcement::AnnouncementServiceInterface, auth::AuthServiceInterface,
    building::BuildingServiceInterface, device::DeviceServiceInterface,
    floor::FloorServiceInterface, request::RequestServiceInterface, role::RoleServiceInterface,
    user::UserServiceInterface,
};

pub async fn run(
    listener: TcpListener,
    redis: deadpool_redis::Pool,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    building_service: Arc<dyn BuildingServiceInterface + Send + Sync + 'static>,
    user_service: Arc<dyn UserServiceInterface + Send + Sync + 'static>,
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    floor_service: Arc<dyn FloorServiceInterface + Send + Sync + 'static>,
    device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
    request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
    announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
    livestream_service: Arc<dyn LivestreamServiceInterface>,
) -> Result<(), std::io::Error> {
    let (notify_shutdown, _) = broadcast::channel::<()>(1);

    let shutdown_1 = Shutdown::new(notify_shutdown.subscribe());
    let shutdown_2 = Shutdown::new(notify_shutdown.subscribe());
    let shutdown_3 = Shutdown::new(notify_shutdown.subscribe());
    let shutdown_4 = Shutdown::new(notify_shutdown.subscribe());

    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);
    let shutdown_complete_tx_1 = shutdown_complete_tx.clone();
    let shutdown_complete_tx_2 = shutdown_complete_tx.clone();
    let shutdown_complete_tx_3 = shutdown_complete_tx.clone();
    let shutdown_complete_tx_4 = shutdown_complete_tx.clone();

    let announcement_service_1 = announcement_service.clone();
    let announcement_service_2 = announcement_service.clone();

    let device_status_sessions: Arc<Mutex<HashMap<usize, Recipient<StatusMessage>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let device_status_devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let device_status_sessions_1 = device_status_sessions.clone();
    let device_status_sessions_2 = device_status_sessions.clone();

    let device_status_devices_1 = device_status_devices.clone();
    let device_status_devices_2 = device_status_devices.clone();

    let device_status_socket_srv =
        StatusSocketServer::new(device_status_sessions_1, device_status_devices_1).start();

    let livestream_sessions: LivestreamSessionMap = Arc::new(Mutex::new(HashMap::new()));
    let livestream_devices: LivestreamDeviceMap = Arc::new(Mutex::new(HashMap::new()));

    let livestream_sessions_1 = livestream_sessions.clone();
    let livestream_sessions_2 = livestream_sessions.clone();

    let livestream_devices_1 = livestream_devices.clone();
    let livestream_devices_2 = livestream_devices.clone();

    let livestream_socket_srv =
        LivestreamSocketServer::new(livestream_sessions_1, livestream_devices_1).start();

    let redis_1 = redis.clone();
    let redis_2 = redis.clone();

    actix_web::rt::spawn(async move {
        let server = match WebServer::build(
            listener,
            role_service,
            building_service,
            user_service,
            auth_service,
            floor_service,
            device_service,
            request_service,
            announcement_service_1,
            device_status_socket_srv,
            livestream_socket_srv,
        ) {
            Ok(server) => server,
            Err(e) => {
                eprintln!("Something went wrong when building the server: {:?}", e);
                return;
            }
        };

        if let Err(e) = server.run(shutdown_1, shutdown_complete_tx_1).await {
            eprintln!("Something went wrong when running the server: {:?}", e);
            return;
        }
    });

    actix_web::rt::spawn(async move {
        scheduler::run(shutdown_2, shutdown_complete_tx_2, announcement_service_2).await;
    });

    actix_web::rt::spawn(async move {
        device_status::listener::run(
            shutdown_3,
            shutdown_complete_tx_3,
            redis_1,
            device_status_sessions_2,
            device_status_devices_2,
        )
        .await;
    });

    actix_web::rt::spawn(async move {
        livestream::listener::run(
            shutdown_4,
            shutdown_complete_tx_4,
            redis_2,
            livestream_service,
            livestream_sessions_2,
            livestream_devices_2,
        )
        .await;
    });

    let signal_listener = actix_web::rt::spawn(async move {
        actix_web::rt::signal::ctrl_c().await.unwrap();
    });

    signal_listener.await.unwrap();

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}
