use std::net::TcpListener;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{guard, web, App, HttpResponse, HttpServer};
use serde::Serialize;
use tokio::sync::{mpsc, broadcast};

use crate::http::AuthenticationMiddlewareFactory;

use crate::features::{
    announcement::{http as announcement_http, AnnouncementServiceInterface},
    auth::{http as auth_http, AuthServiceInterface},
    building::{http as building_http, BuildingServiceInterface},
    device::{http as device_http, DeviceServiceInterface},
    floor::{http as floor_http, FloorServiceInterface},
    request::{http as request_http, RequestServiceInterface},
    role::{http as role_http, ApplicationPermission, RoleServiceInterface},
    user::{http as user_http, UserServiceInterface, UserStatus},
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

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "healthy".into(),
    })
}

pub fn assemble_server(
    listener: TcpListener,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    building_service: Arc<dyn BuildingServiceInterface + Send + Sync + 'static>,
    user_service: Arc<dyn UserServiceInterface + Send + Sync + 'static>,
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    floor_service: Arc<dyn FloorServiceInterface + Send + Sync + 'static>,
    device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
    request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
    announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
) -> Result<Server, std::io::Error> {
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
            .route("/", web::get().to(health_check))
            .service(
                web::scope("/device")
                    .route(
                        "/v1/announcements/{announcement_id}/media",
                        web::get().to(announcement_http::get_announcement_media_presigned_url),
                    )
                    .route(
                        "/v1/devices/{device_id}",
                        web::get().to(device_http::get_device_by_id),
                    )
                    .route(
                        "/v1/buildings",
                        web::get().to(building_http::list_buildings),
                    )
                    .route("/v1/floors", web::get().to(floor_http::list_floor))
                    .route("/v1/devices", web::post().to(device_http::create_device)),
            )
            .service(
                web::scope("/dashboard")
                    .route("/v1/auth/register", web::post().to(auth_http::register))
                    .route(
                        "/v1/auth/verification/{email}",
                        web::get().to(auth_http::send_email_confirmation),
                    )
                    .route(
                        "/v1/auth/verification",
                        web::get().to(auth_http::verify_email_confirmation_token),
                    )
                    .route(
                        "/v1/auth/verification",
                        web::put().to(auth_http::confirm_email),
                    )
                    .route("/v1/auth/login", web::post().to(auth_http::login))
                    .route("/v1/auth/refresh", web::put().to(auth_http::refresh_token))
                    .route("/v1/roles", web::get().to(role_http::list_role))
                    .route(
                        "/v1/auth/forgot-password",
                        web::get().to(auth_http::forgot_password),
                    )
                    .route(
                        "/v1/auth/reset-password",
                        web::put().to(auth_http::reset_password),
                    )
                    .service(
                        web::scope("/v1/me")
                            .service(
                                web::resource("/change-password")
                                    .guard(guard::Put())
                                    .wrap(AuthenticationMiddlewareFactory::new(
                                        auth_service.clone(),
                                    ))
                                    .to(auth_http::change_password),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(AuthenticationMiddlewareFactory::new(
                                        auth_service.clone(),
                                    ))
                                    .to(auth_http::me),
                            ),
                    )
                    .service(
                        web::scope("/v1/logout")
                            .wrap(AuthenticationMiddlewareFactory::new(auth_service.clone()))
                            .route("", web::get().to(auth_http::logout)),
                    )
                    .service(
                        web::scope("/v1/buildings")
                            .service(
                                web::resource("/{building_id}")
                                    .guard(guard::Put())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::UpdateBuilding)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(building_http::update),
                            )
                            .service(
                                web::resource("/{building_id}")
                                    .guard(guard::Delete())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::DeleteBuilding)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(building_http::delete),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::ViewListBuilding,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(building_http::list_buildings),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Post())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::CreateBuilding)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(building_http::create),
                            ),
                    )
                    .service(
                        web::scope("/v1/floors")
                            .service(
                                web::resource("/{floor_id}")
                                    .guard(guard::Put())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::UpdateFloor)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(floor_http::update_floor),
                            )
                            .service(
                                web::resource("/{floor_id}")
                                    .guard(guard::Delete())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::DeleteFloor)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(floor_http::delete_floor),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::ViewListFloor)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(floor_http::list_floor),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Post())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::CreateFloor)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(floor_http::create_floor),
                            ),
                    )
                    .service(
                        web::scope("/v1/devices")
                            .service(
                                web::resource("/{device_id}")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::ViewDeviceDetail,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(device_http::get_device_by_id),
                            )
                            .service(
                                web::resource("/{device_id}")
                                    .guard(guard::Put())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::UpdateDevice)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(device_http::update_device),
                            )
                            .service(
                                web::resource("/{device_id}")
                                    .guard(guard::Delete())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::DeleteDevice)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(device_http::delete_device),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::ViewListDevice)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(device_http::list_device),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Post())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::CreateDevice)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(device_http::create_device),
                            ),
                    )
                    .service(
                        web::scope("/v1/announcements")
                            .service(
                                web::resource("/{announcement_id}/media")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::ViewAnnouncementMedia,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(announcement_http::get_announcement_media_presigned_url),
                            )
                            .service(
                                web::resource("/{announcement_id}")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::ViewAnnouncementDetail,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(announcement_http::get_announcement_detail),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::ViewListAnnouncement,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(announcement_http::list_announcement),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Post())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::CreateAnnouncement,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(announcement_http::create_announcement),
                            ),
                    )
                    .service(
                        web::scope("/v1/requests")
                            .service(
                                web::resource("/{request_id}/approval")
                                    .guard(guard::Put())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::UpdateRequestApproval,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(request_http::update_request_approval),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::ViewListRequest)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(request_http::list_request),
                            ),
                    )
                    .service(
                        web::scope("/v1/users")
                            .service(
                                web::resource("/{user_id}/approval")
                                    .guard(guard::Put())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(
                                                ApplicationPermission::UpdateUserApproval,
                                            )
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(user_http::update_user_approval),
                            )
                            .service(
                                web::resource("")
                                    .guard(guard::Get())
                                    .wrap(
                                        AuthenticationMiddlewareFactory::new(auth_service.clone())
                                            .with_permission(ApplicationPermission::ViewListUser)
                                            .with_status(UserStatus::Approved)
                                            .with_require_email_confirmed(true),
                                    )
                                    .to(user_http::list_user),
                            ),
                    ),
            )
    })
    .listen(listener)?
    .disable_signals()
    .run();

    Ok(server)
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
    let server = match assemble_server(
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
            eprintln!("[error] Something when wrong when assembling the server: {:?}", e);
            return;
        }
    };
    let handle = server.handle();

    let runtime = tokio::spawn(async move {
        println!("[info] Server is starting on http://localhost:8080");

        if let Err(e) = server.await {
            eprintln!("[error] Something when wrong when running the server: {:?}", e);
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
