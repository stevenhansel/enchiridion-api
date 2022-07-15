use actix_web::{web, HttpResponse};
use serde::Deserialize;
use shaku_actix::Inject;
use validator::Validate;

use crate::building::service::{CreateParams, UpdateParams};
use crate::container::Container;
use crate::http::{ApiValidationError, HttpErrorResponse};

use super::service::BuildingServiceInterface;
use super::{BuildingError, BuildingErrorCode};

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
    building_service: Inject<Container, dyn BuildingServiceInterface>,
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
    .await {
        match e {
            BuildingError::BuildingNameAlreadyExists(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    BuildingErrorCode::BuildingNameAlreadyExists.to_string(),
                    vec![message],
                ))
            }
            _ => return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                BuildingErrorCode::InternalServerError.to_string(),
                vec![BuildingError::InternalServerError.to_string()],
            ))
        }
    };

    HttpResponse::NoContent().finish()
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    id: i32,
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
    body: web::Json<UpdateBody>,
    building_service: Inject<Container, dyn BuildingServiceInterface>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = building_service
        .update(UpdateParams {
            id: body.id,
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
            _ => return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                BuildingErrorCode::InternalServerError.to_string(),
                vec![BuildingError::InternalServerError.to_string()],
            ))
        }
    };

    HttpResponse::NoContent().finish()
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBody {
    id: i32,
}

pub async fn delete(
    body: web::Json<DeleteBody>,
    building_service: Inject<Container, dyn BuildingServiceInterface>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = building_service
        .delete_by_id(body.id)
        .await
    {
        match e {
            BuildingError::BuildingNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    BuildingErrorCode::BuildingNotFound.to_string(),
                    vec![message],
                ))
            }
            _ => return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                BuildingErrorCode::InternalServerError.to_string(),
                vec![BuildingError::InternalServerError.to_string()],
            ))
        }
    };

    HttpResponse::NoContent().finish()
}
