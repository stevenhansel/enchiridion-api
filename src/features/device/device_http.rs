use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::Serialize;

use crate::http::{device_middleware, HttpErrorResponse};

use super::{DeviceErrorCode, DeviceServiceInterface, GetDeviceDetailByIdError};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDetailResponse {
    pub id: i32,
    pub access_key_id: String,
    pub name: String,
    pub location: String,
    pub floor_id: i32,
    pub building_id: i32,
    pub description: String,
    pub active_announcements: i32,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn me(
    device_service: web::Data<Arc<dyn DeviceServiceInterface + Send + Sync + 'static>>,
    auth: device_middleware::DeviceAuthenticationContext,
) -> HttpResponse {
    let device_id = match device_middleware::get_device_id(auth) {
        Ok(id) => id,
        Err(e) => return device_middleware::parse_device_authentication_middleware_error(e),
    };

    let result = match device_service.get_device_detail_by_id(device_id).await {
        Ok(res) => res,
        Err(e) => match e {
            GetDeviceDetailByIdError::DeviceNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    DeviceErrorCode::DeviceNotFound.to_string(),
                    vec![message.into()],
                ))
            }
            GetDeviceDetailByIdError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    DeviceErrorCode::InternalServerError.to_string(),
                    vec![GetDeviceDetailByIdError::InternalServerError.to_string()],
                ))
            }
        },
    };

    HttpResponse::Ok().json(DeviceDetailResponse {
        id: result.id,
        access_key_id: result.access_key_id,
        name: result.name.into(),
        location: result.location.into(),
        floor_id: result.floor_id,
        building_id: result.building_id,
        description: result.description.into(),
        active_announcements: result.active_announcements,
        created_at: result.created_at.to_rfc3339(),
        updated_at: result.updated_at.to_rfc3339(),
    })
}
