use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::{
    features::{announcement::CreateAnnouncementError, media::domain::MediaType},
    http::{
        derive_authentication_middleware_error, derive_user_id, device_middleware,
        validate_date_format, ApiValidationError, AuthenticationContext, HttpErrorResponse,
        API_VALIDATION_ERROR_CODE,
    },
};

use super::{
    AnnouncementErrorCode, AnnouncementServiceInterface, AnnouncementStatus,
    AnnouncementStatusObject, CreateAnnouncementParams, GetAnnouncementDetailError,
    GetAnnouncementMediaPresignedURLError, ListAnnouncementError, ListAnnouncementParams,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAnnouncementBody {
    pub title: String,
    pub media_id: i32,
    pub start_date: String,
    pub end_date: String,
    pub notes: String,
    pub device_ids: Vec<i32>,
}

fn parse_announcement_date_input(raw: String) -> Option<chrono::DateTime<chrono::Utc>> {
    let naive_date = match NaiveDate::parse_from_str(raw.as_str(), "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return None,
    };
    let naive_time = NaiveTime::from_hms(0, 0, 0);
    let naive_date_time = NaiveDateTime::new(naive_date, naive_time);

    Some(chrono::Utc.from_utc_datetime(&naive_date_time))
}

pub async fn create_announcement(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    body: web::Json<CreateAnnouncementBody>,
) -> HttpResponse {
    let user_id = match derive_user_id(auth) {
        Ok(id) => id,
        Err(e) => return derive_authentication_middleware_error(e),
    };

    let start_date = match parse_announcement_date_input(body.start_date.clone()) {
        Some(date) => date,
        None => {
            return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                "API_VALIDATION_ERROR".to_string(),
                vec!["Start date is invalid".to_string()],
            ))
        }
    };

    let end_date = match parse_announcement_date_input(body.end_date.clone()) {
        Some(date) => date,
        None => {
            return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                "API_VALIDATION_ERROR".to_string(),
                vec!["End date is invalid".to_string()],
            ))
        }
    };

    if let Err(e) = announcement_service
        .create_announcement(CreateAnnouncementParams {
            title: body.title.clone(),
            media_id: body.media_id,
            notes: body.notes.clone(),
            device_ids: body.device_ids.clone(),
            user_id,
            start_date,
            end_date,
        })
        .await
    {
        match e {
            CreateAnnouncementError::UserNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::UserNotFound.to_string(),
                    vec![message],
                ))
            }
            CreateAnnouncementError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::InternalServerError.to_string(),
                    vec![CreateAnnouncementError::InternalServerError.to_string()],
                ))
            }
        }
    };

    HttpResponse::NoContent().finish()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAnnouncementQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub query: Option<String>,
    pub status: Option<AnnouncementStatus>,
    pub user_id: Option<i32>,
    pub device_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub populate_media: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAnnouncementResponse {
    count: i32,
    total_pages: i32,
    has_next: bool,
    contents: Vec<ListAnnouncementContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAnnouncementContent {
    id: i32,
    title: String,
    start_date: String,
    end_date: String,
    status: AnnouncementStatusObject,
    author: AnnouncementAuthorObject,
    media: String,
    media_type: MediaType,
    media_duration: Option<f64>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementAuthorObject {
    id: i32,
    name: String,
}

pub async fn list_announcement(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    query_params: web::Query<ListAnnouncementQueryParams>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let start_date: Option<chrono::DateTime<chrono::Utc>> =
        if let Some(start_date) = &query_params.start_date {
            if let Ok(date) = validate_date_format(start_date.as_str(), "%Y-%m-%d") {
                Some(date)
            } else {
                return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                    API_VALIDATION_ERROR_CODE.to_string(),
                    vec!["start_date must follow yyyy-mm-dd format".to_string()],
                ));
            }
        } else {
            None
        };

    let end_date: Option<chrono::DateTime<chrono::Utc>> =
        if let Some(end_date) = &query_params.end_date {
            if let Ok(date) = validate_date_format(end_date.as_str(), "%Y-%m-%d") {
                Some(date)
            } else {
                return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                    API_VALIDATION_ERROR_CODE.to_string(),
                    vec!["end_date must follow yyyy-mm-dd format".to_string()],
                ));
            }
        } else {
            None
        };

    let mut page = 1;
    if let Some(raw_page) = query_params.page {
        page = raw_page;
    }

    let mut limit = 25;
    if let Some(raw_limit) = query_params.limit {
        limit = raw_limit;
    }

    let result = match announcement_service
        .list_announcement(ListAnnouncementParams {
            page,
            limit,
            query: query_params.query.clone(),
            status: query_params.status.clone(),
            user_id: query_params.user_id.clone(),
            device_id: query_params.device_id.clone(),
            populate_media: query_params.populate_media.clone(),
            start_date,
            end_date,
        })
        .await
    {
        Ok(res) => res,
        Err(e) => match e {
            ListAnnouncementError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::InternalServerError.to_string(),
                    vec![ListAnnouncementError::InternalServerError.to_string()],
                ))
            }
        },
    };

    let contents = result
        .contents
        .into_iter()
        .map(|row| ListAnnouncementContent {
            id: row.id,
            title: row.title,
            start_date: row.start_date.to_rfc3339(),
            end_date: row.end_date.to_rfc3339(),
            status: row.status.object(),
            author: AnnouncementAuthorObject {
                id: row.user_id,
                name: row.user_name,
            },
            media: row.media,
            media_type: row.media_type,
            media_duration: row.media_duration,
            created_at: row.created_at.to_rfc3339(),
        })
        .collect();

    HttpResponse::Ok().json(ListAnnouncementResponse {
        total_pages: result.total_pages,
        count: result.count,
        has_next: result.has_next,
        contents,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAnnouncementDetailResponse {
    id: i32,
    title: String,
    media: String,
    notes: String,
    status: AnnouncementStatusObject,
    author: AnnouncementAuthorObject,
    start_date: String,
    end_date: String,
    devices: Vec<GetAnnouncementDetailDevice>,
    created_at: String,
    updated_at: String,
    media_type: MediaType,
    media_duration: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAnnouncementDetailDevice {
    id: i32,
    name: String,
    description: String,
    floor_id: i32,
}

pub async fn get_announcement_detail(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    announcement_id: web::Path<i32>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let announcement_id = announcement_id.into_inner();

    let result = match announcement_service
        .get_announcement_detail(announcement_id)
        .await
    {
        Ok(result) => result,
        Err(e) => match e {
            GetAnnouncementDetailError::AnnouncementNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::AnnouncementNotFound.to_string(),
                    vec![message],
                ))
            }
            GetAnnouncementDetailError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::InternalServerError.to_string(),
                    vec![GetAnnouncementDetailError::InternalServerError.to_string()],
                ))
            }
        },
    };

    HttpResponse::Ok().json(GetAnnouncementDetailResponse {
        id: result.id,
        title: result.title,
        media: result.media,
        media_type: result.media_type,
        media_duration: result.media_duration,
        notes: result.notes,
        status: result.status.object(),
        author: AnnouncementAuthorObject {
            id: result.user_id,
            name: result.user_name,
        },
        start_date: result.start_date.to_rfc3339(),
        end_date: result.end_date.to_rfc3339(),
        devices: result
            .devices
            .into_iter()
            .map(|row| GetAnnouncementDetailDevice {
                id: row.id,
                name: row.name,
                description: row.description,
                floor_id: row.floor_id,
            })
            .collect(),
        created_at: result.created_at.to_rfc3339(),
        updated_at: result.updated_at.to_rfc3339(),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAnnouncementMediaPresignedURLResponse {
    filename: String,
    media: String,
}

pub async fn get_announcement_media_presigned_url_dashboard(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    announcement_id: web::Path<i32>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let obj = match announcement_service
        .get_announcement_media_presigned_url(announcement_id.into_inner())
        .await
    {
        Ok(media) => media,
        Err(e) => match e {
            GetAnnouncementMediaPresignedURLError::AnnouncementNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::AnnouncementNotFound.to_string(),
                    vec![message],
                ))
            }
            GetAnnouncementMediaPresignedURLError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::InternalServerError.to_string(),
                    vec![GetAnnouncementMediaPresignedURLError::InternalServerError.to_string()],
                ))
            }
        },
    };

    HttpResponse::Ok().json(GetAnnouncementMediaPresignedURLResponse {
        filename: obj.filename,
        media: obj.media,
    })
}

pub async fn get_announcement_media_presigned_url_device(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: device_middleware::DeviceAuthenticationContext,
    announcement_id: web::Path<i32>,
) -> HttpResponse {
    if let Err(e) = device_middleware::get_device_id(auth) {
        return device_middleware::parse_device_authentication_middleware_error(e);
    }

    let obj = match announcement_service
        .get_announcement_media_presigned_url(announcement_id.into_inner())
        .await
    {
        Ok(media) => media,
        Err(e) => match e {
            GetAnnouncementMediaPresignedURLError::AnnouncementNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::AnnouncementNotFound.to_string(),
                    vec![message],
                ))
            }
            GetAnnouncementMediaPresignedURLError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AnnouncementErrorCode::InternalServerError.to_string(),
                    vec![GetAnnouncementMediaPresignedURLError::InternalServerError.to_string()],
                ))
            }
        },
    };

    HttpResponse::Ok().json(GetAnnouncementMediaPresignedURLResponse {
        filename: obj.filename,
        media: obj.media,
    })
}
