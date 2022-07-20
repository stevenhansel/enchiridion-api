use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::{
    CreateFloorError, FloorErrorCode, FloorServiceInterface, ListFloorError, ListFloorParams,
};

use crate::http::{
    derive_authentication_middleware_error, derive_user_id, ApiValidationError,
    AuthenticationContext, HttpErrorResponse,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFloorQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub query: Option<String>,
    pub building_id: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFloorResponse {
    count: i32,
    total_pages: i32,
    has_next: bool,
    contents: Vec<ListFloorContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFloorContent {
    id: i32,
    name: String,
    building: ListFloorBuildingContent,
    devices: Vec<ListFloorDeviceContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFloorBuildingContent {
    pub id: i32,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFloorDeviceContent {
    pub id: i32,
    pub name: String,
    pub description: String,
    // pub total_announcements: i32,
}

pub async fn list_floor(
    floor_service: web::Data<Arc<dyn FloorServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext<'_>,
    query_params: web::Query<ListFloorQueryParams>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let mut page = 1;
    if let Some(raw_page) = query_params.page {
        page = raw_page;
    }

    let mut limit = 25;
    if let Some(raw_limit) = query_params.limit {
        limit = raw_limit;
    }

    let result = match floor_service
        .list_floor(ListFloorParams {
            page,
            limit,
            building_id: query_params.building_id.clone(),
            query: query_params.query.clone(),
        })
        .await
    {
        Ok(result) => result,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                ListFloorError::InternalServerError.to_string(),
                vec![ListFloorError::InternalServerError.to_string()],
            ))
        }
    };

    let contents = result
        .contents
        .into_iter()
        .map(|row| ListFloorContent {
            id: row.id,
            name: row.name.clone(),
            building: ListFloorBuildingContent {
                id: row.building.id,
                name: row.building.name,
                color: row.building.color,
            },
            devices: row
                .devices
                .into_iter()
                .map(|row| ListFloorDeviceContent {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                })
                .collect(),
        })
        .collect();

    HttpResponse::Ok().json(ListFloorResponse {
        count: result.count,
        total_pages: result.total_pages,
        has_next: result.has_next,
        contents,
    })
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFloorBody {
    #[validate(length(min = 1, message = "name: name must not be empty"))]
    name: String,
    #[validate(range(min = 1, message = "buildingId: id must be greater than 1"))]
    building_id: i32,
}

pub async fn create_floor(
    floor_service: web::Data<Arc<dyn FloorServiceInterface + Send + Sync + 'static>>,
    body: web::Json<CreateFloorBody>,
) -> HttpResponse {
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

pub async fn update_floor() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn delete_floor() -> HttpResponse {
    HttpResponse::NoContent().finish()
}
