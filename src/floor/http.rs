use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use validator::Validate;

use super::{CreateFloorError, FloorErrorCode, FloorServiceInterface};

use crate::http::{ApiValidationError, HttpErrorResponse};

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFloorBody {
    #[validate(length(min = 1, message = "name: Name must not be empty"))]
    name: String,
    #[validate(range(min = 1, message = "buildingId: id must be greater than 1"))]
    building_id: i32,
}

pub async fn create_floor(
    floor_service: web::Data<Arc<dyn FloorServiceInterface + Send + Sync + 'static>>,
    body: web::Json<CreateFloorBody>,
) -> HttpResponse {
    println!("start");
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = floor_service
        .create_floor(super::CreateFloorParams {
            name: body.name.clone(),
            building_id: body.building_id,
        })
        .await
    {
        match e {
            CreateFloorError::FloorAlreadyExists(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    FloorErrorCode::FloorAlreadyExists.to_string(),
                    vec![message],
                ))
            }
            CreateFloorError::BuildingNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    FloorErrorCode::BuildingNotFound.to_string(),
                    vec![message],
                ))
            }
            CreateFloorError::InternalServerError => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    FloorErrorCode::InternalServerError.to_string(),
                    vec![FloorErrorCode::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}
