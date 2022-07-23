use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    announcement::CreateAnnouncementError,
    cloud_storage::TmpFile,
    http::{
        derive_authentication_middleware_error, derive_user_id, AuthenticationContext,
        HttpErrorResponse,
    },
};

use super::{
    AnnouncementErrorCode, AnnouncementServiceInterface, AnnouncementStatus,
    AnnouncementStatusObject, CreateAnnouncementParams, GetAnnouncementDetailError,
    ListAnnouncementError, ListAnnouncementParams,
};

#[derive(Debug)]
pub struct CreateAnnouncementFormData {
    pub title: String,
    pub media: TmpFile,
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

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(item) => item,
            Err(e) => return Err(e.to_string()),
        };

        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => return Err(e.to_string()),
            };

            if field.name() == "title" {
                title = match std::str::from_utf8(&chunk) {
                    Ok(title) => Some(title.to_string()),
                    Err(e) => return Err(e.to_string()),
                }
            } else if field.name() == "startDate" {
                let date_string = match std::str::from_utf8(&chunk) {
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
                let date_string = match std::str::from_utf8(&chunk) {
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
                notes = match std::str::from_utf8(&chunk) {
                    Ok(notes) => Some(notes.to_string()),
                    Err(e) => return Err(e.to_string()),
                }
            } else if field.name() == "deviceIds" {
                let raw_ids = match std::str::from_utf8(&chunk) {
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
                let now = chrono::Utc::now().timestamp().to_string();
                let tmp = TmpFile::new(
                    now,
                    field.content_type().subtype().to_string(),
                    "announcement".into(),
                );

                let path = tmp.path.clone();
                if let Err(e) = web::block(move || TmpFile::write(path, chunk))
                    .await
                    .unwrap()
                {
                    return Err(e.to_string());
                };

                media = Some(tmp);
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

    Ok(CreateAnnouncementFormData {
        title,
        start_date,
        end_date,
        notes,
        device_ids,
        media,
    })
}

pub async fn create_announcement(
    announcement_service: web::Data<Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext<'_>,
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

    if let Err(e) = announcement_service
        .create_announcement(CreateAnnouncementParams {
            title: form.title.clone(),
            media: form.media.to_owned(),
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
    auth: AuthenticationContext<'_>,
    query_params: web::Query<ListAnnouncementQueryParams>,
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

    let result = match announcement_service
        .list_announcement(ListAnnouncementParams {
            page,
            limit,
            query: query_params.query.clone(),
            status: query_params.status.clone(),
            user_id: query_params.user_id.clone(),
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
    auth: AuthenticationContext<'_>,
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
