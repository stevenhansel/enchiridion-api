use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::http::{ApiValidationError, HttpErrorResponse};

use super::{
    CreateDeviceError, CreateDeviceParams, DeviceErrorCode, DeviceServiceInterface,
    ListDeviceError, ListDeviceParams,
};

#[derive(Debug, Deserialize)]
pub struct ListDeviceQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub query: Option<String>,
    pub building_id: Option<i32>,
    pub floor_id: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDeviceResponse {
    count: i32,
    total_pages: i32,
    has_next: bool,
    contents: Vec<ListDeviceContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDeviceContent {
    pub id: i32,
    pub name: String,
    pub location: String,
    // pub active_announcements: i32,
    pub description: String,
}

pub async fn list_device(
    device_service: web::Data<Arc<dyn DeviceServiceInterface + Send + Sync + 'static>>,
    query_params: web::Query<ListDeviceQueryParams>,
) -> HttpResponse {
    let mut page = 1;
    if let Some(raw_page) = query_params.page {
        println!("got in");
        page = raw_page;
    }

    let mut limit = 25;
    if let Some(raw_limit) = query_params.limit {
        limit = raw_limit;
    }

    let result = match device_service
        .list_device(ListDeviceParams {
            page,
            limit,
            query: query_params.query.clone(),
            building_id: query_params.building_id.clone(),
            floor_id: query_params.floor_id.clone(),
        })
        .await
    {
        Ok(res) => res,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                ListDeviceError::InternalServerError.to_string(),
                vec![ListDeviceError::InternalServerError.to_string()],
            ))
        }
    };

    let contents = result
        .contents
        .iter()
        .map(|c| ListDeviceContent {
            id: c.id,
            name: c.name.to_string(),
            location: c.location.to_string(),
            // active_announcements: c.active_announcements,
            description: c.description.to_string(),
        })
        .collect();

    HttpResponse::Ok().json(ListDeviceResponse {
        count: result.count,
        total_pages: result.total_pages,
        has_next: result.has_next,
        contents,
    })
}

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
