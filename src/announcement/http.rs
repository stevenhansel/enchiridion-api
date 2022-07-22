use std::{
    fs::{create_dir_all, File},
    io::Write,
    sync::Arc,
};

use actix_multipart::Multipart;
use actix_web::{
    web::{self, Bytes},
    Error, HttpResponse,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use futures::StreamExt;

use crate::{announcement::CreateAnnouncementError, http::HttpErrorResponse};
use crate::{announcement::CreateAnnouncementParams, cloud_storage::TmpFile};

use super::AnnouncementServiceInterface;

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
    multipart: Multipart,
) -> HttpResponse {
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
            media: form.media.to_owned(),
        })
        .await
    {
        match e {
            CreateAnnouncementError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    CreateAnnouncementError::InternalServerError.to_string(),
                    vec![CreateAnnouncementError::InternalServerError.to_string()],
                ))
            }
        }
    };
    println!("form: {:?}", form);

    HttpResponse::NoContent().finish()
}
