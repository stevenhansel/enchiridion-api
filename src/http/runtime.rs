use std::{net::TcpListener, sync::Arc};

use actix_cors::Cors;
use actix_web::{dev::{Server, ServerHandle}, web, App, HttpServer};

use crate::features::{
    announcement::AnnouncementServiceInterface, auth::AuthServiceInterface,
    building::BuildingServiceInterface, device::DeviceServiceInterface,
    floor::FloorServiceInterface, request::RequestServiceInterface, role::RoleServiceInterface,
    user::UserServiceInterface,
};

use super::routes::{dashboard_routes, device_routes};

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
    ) -> Result<Self, std::io::Error> {
        let role_svc = web::Data::new(role_service.clone());
        let building_svc = web::Data::new(building_service.clone());
        let user_svc = web::Data::new(user_service.clone());
        let auth_svc = web::Data::new(auth_service.clone());
        let floor_svc = web::Data::new(floor_service.clone());
        let device_svc = web::Data::new(device_service.clone());
        let request_svc = web::Data::new(request_service.clone());
        let announcement_svc = web::Data::new(announcement_service.clone());
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
                .service(device_routes())
                .service(dashboard_routes(auth_service.clone()))
        })
        .listen(listener)?
        .disable_signals()
        .run();

        Ok(WebServer { server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn get_handle(&self) -> ServerHandle {
        self.server.handle()
    }
}
