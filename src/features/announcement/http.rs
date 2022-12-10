use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    cloud_storage::TmpFile,
    features::announcement::CreateAnnouncementError,
    http::{
        derive_authentication_middleware_error, derive_user_id, device_middleware,
        validate_date_format, AuthenticationContext, HttpErrorResponse, API_VALIDATION_ERROR_CODE,
    },
};

use super::{
    AnnouncementErrorCode, AnnouncementServiceInterface, AnnouncementStatus,
    AnnouncementStatusObject, CreateAnnouncementParams, GetAnnouncementDetailError,
    GetAnnouncementMediaPresignedURLError, ListAnnouncementError, ListAnnouncementParams,
};

#[derive(Debug)]
pub struct CreateAnnouncementFormData {
    pub title: String,
    pub media: TmpFile,
    pub media_type: String,
    pub media_duration: Option<f32>,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub notes: String,
    pub device_ids: Vec<i32>,
}

pub async fn parse_create_announcement_multipart(
    mut payload: Multipart,
) -> Result<CreateAnnouncementFormData, String> {
    let mut title: Option<String> = None;
    let mut start_date: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut end_date: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut notes: Option<String> = None;
    let mut device_ids: Option<Vec<i32>> = None;
    let mut media: Option<TmpFile> = None;
    let mut media_type: Option<String> = None;
    let mut media_duration: Option<f32> = None;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(item) => item,
            Err(e) => return Err(e.to_string()),
        };

        if field.name() == "title" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            title = match std::str::from_utf8(&chunks[0]) {
                Ok(title) => Some(title.to_string()),
                Err(e) => return Err(e.to_string()),
            }
        } else if field.name() == "startDate" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            let date_string = match std::str::from_utf8(&chunks[0]) {
                Ok(title) => title.to_string(),
                Err(e) => return Err(e.to_string()),
            };
            let naive_date = match NaiveDate::parse_from_str(date_string.as_str(), "%Y-%m-%d") {
                Ok(date) => date,
                Err(e) => return Err(e.to_string()),
            };
            let naive_time = NaiveTime::from_hms(0, 0, 0);
            let naive_date_time = NaiveDateTime::new(naive_date, naive_time);

            start_date = Some(chrono::Utc.from_utc_datetime(&naive_date_time));
        } else if field.name() == "endDate" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            let date_string = match std::str::from_utf8(&chunks[0]) {
                Ok(title) => title.to_string(),
                Err(e) => return Err(e.to_string()),
            };
            let naive_date = match NaiveDate::parse_from_str(date_string.as_str(), "%Y-%m-%d") {
                Ok(date) => date,
                Err(e) => return Err(e.to_string()),
            };
            let naive_time = NaiveTime::from_hms(0, 0, 0);
            let naive_date_time = NaiveDateTime::new(naive_date, naive_time);

            end_date = Some(chrono::Utc.from_utc_datetime(&naive_date_time));
        } else if field.name() == "notes" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            notes = match std::str::from_utf8(&chunks[0]) {
                Ok(notes) => Some(notes.to_string()),
                Err(e) => return Err(e.to_string()),
            }
        } else if field.name() == "deviceIds" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            let raw_ids = match std::str::from_utf8(&chunks[0]) {
                Ok(notes) => notes.to_string(),
                Err(e) => return Err(e.to_string()),
            };

            let mut ids: Vec<i32> = vec![];
            for raw_id in raw_ids.split(",") {
                let id: i32 = match raw_id.parse() {
                    Ok(id) => id,
                    Err(e) => return Err(e.to_string()),
                };
                ids.push(id);
            }

            device_ids = Some(ids);
        } else if field.name() == "media" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }
            let now = chrono::Utc::now().timestamp().to_string();
            let tmp = TmpFile::new(
                now,
                field.content_type().subtype().to_string(),
                "announcement".into(),
            );
            let path = tmp.path.clone();

            if let Err(e) = web::block(move || TmpFile::write(path, chunks))
                .await
                .unwrap()
            {
                return Err(e.to_string());
            };

            media = Some(tmp);
        } else if field.name() == "media_type" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            media_type = match std::str::from_utf8(&chunks[0]) {
                Ok(media_type) => Some(media_type.to_string()),
                Err(e) => return Err(e.to_string()),
            }
        } else if field.name() == "media_duration" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            media_duration = match std::str::from_utf8(&chunks[0]) {
                Ok(media_duration) => Some(media_duration.to_string().parse::<f32>().unwrap()),
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    let title = match title {
        Some(title) => title,
        None => return Err("title is required".into()),
    };
    let start_date = match start_date {
        Some(start_date) => start_date,
        None => return Err("startDate is required".into()),
    };
    let end_date = match end_date {
        Some(end_date) => end_date,
        None => return Err("endDate is required".into()),
    };
    let notes = match notes {
        Some(notes) => notes,
        None => return Err("notes is required".into()),
    };
    let device_ids = match device_ids {
        Some(ids) => ids,
        None => return Err("deviceIds is required".into()),
    };
    let media = match media {
        Some(media) => media,
        None => return Err("media is required".into()),
    };
    let media_type = match media_type {
        Some(media_type) => media_type,
        None => return Err("media type is required".into()),
    };
    let media_duration = match media_duration {
        Some(media_duration) => media_duration,
        None => return Err("media duration is required".into()),
    };

    Ok(CreateAnnouncementFormData {
        title,
        start_date,
        end_date,
        notes,
        device_ids,
        media,
        media_type,
        media_duration,
    })
}

pub async fn create_announcement(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    multipart: Multipart,
) -> HttpResponse {
    let user_id = match derive_user_id(auth) {
        Ok(id) => id,
        Err(e) => return derive_authentication_middleware_error(e),
    };

    let form = match parse_create_announcement_multipart(multipart).await {
        Ok(result) => result,
        Err(message) => {
            return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                "API_VALIDATION_ERROR".into(),
                vec![message],
            ))
        }
    };

    let today = chrono::Utc::today().and_hms(0, 0, 0);
    if form.start_date < today {
        return HttpResponse::BadRequest().json(HttpErrorResponse::new(
            "API_VALIDATION_ERROR".into(),
            vec![
                "Start date of the announcement must be greater than or equal to today".to_string(),
            ],
        ));
    }

    if form.start_date >= form.end_date {
        return HttpResponse::BadRequest().json(HttpErrorResponse::new(
            "API_VALIDATION_ERROR".into(),
            vec!["End date of the announcement must be greater than the start date of the announcement".to_string()],
        ));
    }

    if let Err(e) = announcement_service
        .create_announcement(CreateAnnouncementParams {
            title: form.title.clone(),
            media: form.media.to_owned(),
            media_type: form.media_type,
            media_duration: form.media_duration,
            start_date: form.start_date,
            end_date: form.end_date,
            notes: form.notes.clone(),
            device_ids: form.device_ids,
            user_id,
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
    media_type: String,
    media_duration: Option<f32>,
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
