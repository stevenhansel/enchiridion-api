use std::net::TcpListener;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};

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
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    building_service: Arc<dyn BuildingServiceInterface + Send + Sync + 'static>,
    user_service: Arc<dyn UserServiceInterface + Send + Sync + 'static>,
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    floor_service: Arc<dyn FloorServiceInterface + Send + Sync + 'static>,
    device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
    request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
    announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
) -> Result<(), std::io::Error> {
    let (notify_shutdown, _) = broadcast::channel::<()>(1);

    let shutdown_1 = Shutdown::new(notify_shutdown.subscribe());
    let shutdown_2 = Shutdown::new(notify_shutdown.subscribe());

    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);
    let shutdown_complete_tx_1 = shutdown_complete_tx.clone();
    let shutdown_complete_tx_2 = shutdown_complete_tx.clone();

    let announcement_service_1 = announcement_service.clone();
    let announcement_service_2 = announcement_service.clone();

    tokio::spawn(async move {
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
        ) {
            Ok(server) => server,
            Err(e) => {
                eprintln!(
                    "[error] Something went wrong when building the server: {:?}",
                    e
                );
                return;
            }
        };

        if let Err(e) = server.run(shutdown_1, shutdown_complete_tx_1).await {
            eprintln!(
                "[error] Something went wrong when running the server: {:?}",
                e
            );
            return;
        }
    });

    tokio::spawn(async move {
        scheduler::run(shutdown_2, shutdown_complete_tx_2, announcement_service_2).await;
    });

    let signal_listener = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
    });

    signal_listener.await.unwrap();

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}
