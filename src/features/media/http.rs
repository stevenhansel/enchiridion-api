use std::{str::FromStr, sync::Arc};

use actix_multipart::Multipart;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use futures::StreamExt;
use serde::Serialize;

use crate::{
    cloud_storage::TmpFile,
    http::{
        derive_authentication_middleware_error, derive_user_id, AuthenticationContext,
        HttpErrorResponse,
    },
};

use super::{
    domain::MediaType,
    error::{CreateMediaError, MediaErrorCode},
    service::{CreateMediaParams, MediaServiceInterface},
};

pub struct CreateMediaMultipart {
    pub media: TmpFile,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CreateMediaResponse {
    pub id: i32,
}

pub async fn parse_create_announcement_multipart(
    mut payload: Multipart,
) -> Result<CreateMediaMultipart, String> {
    let mut media: Option<TmpFile> = None;
    let mut media_type: Option<MediaType> = None;
    let mut media_duration: Option<f64> = None;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(item) => item,
            Err(e) => return Err(e.to_string()),
        };

        if field.name() == "media" {
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

            let media_type_str = match std::str::from_utf8(&chunks[0]) {
                Ok(media_type) => media_type,
                Err(e) => return Err(e.to_string()),
            };

            if let Ok(media_type_res) = MediaType::from_str(media_type_str) {
                media_type = Some(media_type_res);
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

            if let Ok(duration) = std::str::from_utf8(&chunks[0]) {
                if duration != "null" {
                    media_duration = Some(duration.to_string().parse::<f64>().unwrap());
                }
            }
        }
    }

    let media = match media {
        Some(media) => media,
        None => return Err("media is required".into()),
    };
    let media_type = match media_type {
        Some(media_type) => media_type,
        None => return Err("media type is required".into()),
    };

    Ok(CreateMediaMultipart {
        media,
        media_type,
        media_duration,
    })
}

pub async fn upload(
    media_service: web::Data<Arc<dyn MediaServiceInterface>>,
    auth: AuthenticationContext,
    multipart: Multipart,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let form = match parse_create_announcement_multipart(multipart).await {
        Ok(result) => result,
        Err(message) => {
            return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                "API_VALIDATION_ERROR".into(),
                vec![message],
            ))
        }
    };

    let id = match media_service
        .create_media(CreateMediaParams {
            media: form.media,
            media_type: form.media_type,
            media_duration: form.media_duration,
        })
        .await
    {
        Ok(id) => id,
        Err(e) => {
            println!("{:?}", e);
            match e {
                CreateMediaError::Database(_) | CreateMediaError::Unknown => {
                    return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                        MediaErrorCode::InternalServerError.to_string(),
                        vec![MediaErrorCode::InternalServerError.to_string()],
                    ))
                }
            }
        }
    };

    HttpResponse::Created().json(CreateMediaResponse { id })
}
