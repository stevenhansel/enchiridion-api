use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::building::service::{CreateParams, UpdateParams};

use crate::http::{ApiValidationError, HttpErrorResponse};

use super::service::BuildingServiceInterface;
use super::{BuildingError, BuildingErrorCode};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingJson {
    id: i32,
    name: String,
    color: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBuildingsResponse {
    contents: Vec<BuildingJson>,
}

pub async fn list_buildings(
    building_service: web::Data<Arc<dyn BuildingServiceInterface + Send + Sync + 'static>>,
) -> HttpResponse {
    let result = building_service.get_buildings().await;

    let result = match result {
        Ok(buildings) => buildings,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                BuildingErrorCode::InternalServerError.to_string(),
                vec![BuildingError::InternalServerError.to_string()],
            ))
        }
    };

    HttpResponse::Ok().json(ListBuildingsResponse {
        contents: result
            .into_iter()
            .map(|building| BuildingJson {
                id: building.id,
                name: building.name,
                color: building.color,
            })
            .collect(),
    })
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateBody {
    #[validate(length(min = 1, message = "name: Name must not be empty"))]
    name: String,
    #[validate(length(
        min = 7,
        max = 7,
        message = "color: Color can only have at least 8 characters"
    ))]
    color: String,
}

pub async fn create(
    body: web::Json<CreateBody>,
    building_service: web::Data<Arc<dyn BuildingServiceInterface + Send + Sync + 'static>>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = building_service
        .create(CreateParams {
            name: body.name.to_string(),
            color: body.color.to_string(),
        })
        .await
    {
        match e {
            BuildingError::BuildingNameAlreadyExists(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    BuildingErrorCode::BuildingNameAlreadyExists.to_string(),
                    vec![message],
                ))
            }
            _ => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    BuildingErrorCode::InternalServerError.to_string(),
                    vec![BuildingError::InternalServerError.to_string()],
                ))
            }
        }
    };

    HttpResponse::NoContent().finish()
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    #[validate(length(min = 1, message = "name: Name must not be empty"))]
    name: String,
    #[validate(length(
        min = 7,
        max = 7,
        message = "color: Color can only have at least 7 characters"
    ))]
    color: String,
}

pub async fn update(
    building_service: web::Data<Arc<dyn BuildingServiceInterface + Send + Sync + 'static>>,
    path: web::Path<i32>,
    body: web::Json<UpdateBody>,
) -> HttpResponse {
    let building_id = path.into_inner();

    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = building_service
        .update(UpdateParams {
            id: building_id,
            name: body.name.to_string(),
            color: body.color.to_string(),
        })
        .await
    {
        match e {
            BuildingError::BuildingNameAlreadyExists(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    BuildingErrorCode::BuildingNameAlreadyExists.to_string(),
                    vec![message],
                ))
            }
            BuildingError::BuildingNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    BuildingErrorCode::BuildingNotFound.to_string(),
                    vec![message],
                ))
            }
            _ => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    BuildingErrorCode::InternalServerError.to_string(),
                    vec![BuildingError::InternalServerError.to_string()],
                ))
            }
        }
    };

    HttpResponse::NoContent().finish()
}

pub async fn delete(
    building_service: web::Data<Arc<dyn BuildingServiceInterface + Send + Sync + 'static>>,
    path: web::Path<i32>,
) -> HttpResponse {
    let building_id = path.into_inner();

    if let Err(e) = building_service.delete_by_id(building_id).await {
        match e {
            BuildingError::BuildingNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    BuildingErrorCode::BuildingNotFound.to_string(),
                    vec![message],
                ))
            }
            _ => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    BuildingErrorCode::InternalServerError.to_string(),
                    vec![BuildingError::InternalServerError.to_string()],
                ))
            }
        }
    };

    HttpResponse::NoContent().finish()
}
