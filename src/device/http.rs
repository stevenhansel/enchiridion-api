use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use validator::Validate;

use crate::http::{ApiValidationError, HttpErrorResponse};

use super::{CreateDeviceParams, DeviceServiceInterface, CreateDeviceError, DeviceErrorCode};

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeviceBody {
    #[validate(length(min = 1, message = "name: name must not be empty"))]
    name: String,
    #[validate(length(min = 1, message = "description: description must not be empty"))]
    description: String,
    #[validate(range(min = 1, message = "floorId: id must be greater than 1"))]
    floor_id: i32,
    is_linked: bool,
}

pub async fn create_device(
    device_service: web::Data<Arc<dyn DeviceServiceInterface + Send + Sync + 'static>>,
    body: web::Json<CreateDeviceBody>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = device_service
        .create_device(CreateDeviceParams {
            name: body.name.clone(),
            description: body.description.clone(),
            floor_id: body.floor_id,
            is_linked: body.is_linked,
        })
        .await
    {
        match e {
            CreateDeviceError::DeviceAlreadyExists(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    DeviceErrorCode::DeviceAlreadyExists.to_string(),
                    vec![message],
                ))
            }
            CreateDeviceError::FloorNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    DeviceErrorCode::FloorNotFound.to_string(),
                    vec![message],
                ))
            }
            CreateDeviceError::InternalServerError => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    DeviceErrorCode::InternalServerError.to_string(),
                    vec![DeviceErrorCode::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}
