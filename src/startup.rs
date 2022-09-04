use std::net::TcpListener;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};

use crate::http::WebServer;

use crate::features::{
    announcement::AnnouncementServiceInterface, auth::AuthServiceInterface,
    building::BuildingServiceInterface, device::DeviceServiceInterface,
    floor::FloorServiceInterface, request::RequestServiceInterface, role::RoleServiceInterface,
    user::UserServiceInterface,
};

#[derive(Debug)]
pub struct Shutdown {
    shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.recv().await;
        self.shutdown = true;
    }
}

pub async fn run(
    mut shutdown: Shutdown,
    _sender: mpsc::Sender<()>,
    listener: TcpListener,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    building_service: Arc<dyn BuildingServiceInterface + Send + Sync + 'static>,
    user_service: Arc<dyn UserServiceInterface + Send + Sync + 'static>,
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    floor_service: Arc<dyn FloorServiceInterface + Send + Sync + 'static>,
    device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
    request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
    announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
) {
    let web_server = match WebServer::build(
        listener,
        role_service,
        building_service,
        user_service,
        auth_service,
        floor_service,
        device_service,
        request_service,
        announcement_service,
    ) {
        Ok(server) => server,
        Err(e) => {
            eprintln!(
                "[error] Something when wrong when assembling the server: {:?}",
                e
            );
            return;
        }
    };
    let handle = web_server.get_handle();
    let server = web_server.server;

    let runtime = tokio::spawn(async move {
        println!("[info] Server is starting on http://localhost:8080");

        if let Err(e) = server.await {
            eprintln!(
                "[error] Something when wrong when running the server: {:?}",
                e
            );
            return;
        }
    });

    let shutdown_listener = tokio::spawn(async move {
        let _ = shutdown.recv().await;

        handle.stop(true).await;
    });

    runtime.await.unwrap();
    shutdown_listener.await.unwrap();
}
