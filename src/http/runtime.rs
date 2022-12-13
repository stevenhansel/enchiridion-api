use std::{net::TcpListener, sync::Arc};

use actix::Addr;
use actix_cors::Cors;
use actix_web::{
    dev::{Server, ServerHandle},
    web, App, HttpServer,
};
use tokio::sync::mpsc;

use crate::{
    features::{
        announcement::AnnouncementServiceInterface, auth::AuthServiceInterface,
        building::BuildingServiceInterface, device::DeviceServiceInterface,
        device_status::socket::StatusSocketServer, floor::FloorServiceInterface,
        livestream::socket::LivestreamSocketServer, request::RequestServiceInterface,
        role::RoleServiceInterface, user::UserServiceInterface,
    },
    shutdown::Shutdown,
};

use super::{
    routes::{dashboard_routes, device_routes},
    socket_routes,
};

pub struct WebServer {
    pub server: Server,
}

impl WebServer {
    pub fn build(
        listener: TcpListener,
        role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
        building_service: Arc<dyn BuildingServiceInterface + Send + Sync + 'static>,
        user_service: Arc<dyn UserServiceInterface + Send + Sync + 'static>,
        auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
        floor_service: Arc<dyn FloorServiceInterface + Send + Sync + 'static>,
        device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
        request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
        announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
        status_socket_server_addr: Addr<StatusSocketServer>,
        livestream_socket_server_addr: Addr<LivestreamSocketServer>,
    ) -> Result<Self, std::io::Error> {
        let role_svc = web::Data::new(role_service.clone());
        let building_svc = web::Data::new(building_service.clone());
        let user_svc = web::Data::new(user_service.clone());
        let auth_svc = web::Data::new(auth_service.clone());
        let floor_svc = web::Data::new(floor_service.clone());
        let device_svc = web::Data::new(device_service.clone());
        let request_svc = web::Data::new(request_service.clone());
        let announcement_svc = web::Data::new(announcement_service.clone());
        let status_socket_srv = web::Data::new(status_socket_server_addr);
        let livestream_socket_srv = web::Data::new(livestream_socket_server_addr);

        let server = HttpServer::new(move || {
            let cors = Cors::permissive();

            App::new()
                .wrap(cors)
                .app_data(role_svc.clone())
                .app_data(building_svc.clone())
                .app_data(user_svc.clone())
                .app_data(auth_svc.clone())
                .app_data(floor_svc.clone())
                .app_data(device_svc.clone())
                .app_data(request_svc.clone())
                .app_data(announcement_svc.clone())
                .app_data(status_socket_srv.clone())
                .app_data(livestream_socket_srv.clone())
                // .wrap(Logger::default())
                .service(device_routes(device_service.clone()))
                .service(dashboard_routes(auth_service.clone()))
                .service(socket_routes())
        })
        .listen(listener)?
        .disable_signals()
        .run();

        Ok(WebServer { server })
    }

    pub async fn run(
        self,
        mut shutdown: Shutdown,
        _sender: mpsc::Sender<()>,
    ) -> Result<(), std::io::Error> {
        let handle = self.get_handle();

        let runtime = actix_web::rt::spawn(async move {
            println!("Server is starting on http://localhost:8080");

            if let Err(e) = self.server.await {
                eprintln!("Something when wrong when running the server: {:?}", e);
                return;
            }
        });

        let shutdown_listener = actix_web::rt::spawn(async move {
            let _ = shutdown.recv().await;

            handle.stop(true).await;
            println!("Web service finished shutting down");
        });

        runtime.await.unwrap();
        shutdown_listener.await.unwrap();

        Ok(())
    }

    pub fn get_handle(&self) -> ServerHandle {
        self.server.handle()
    }
}
