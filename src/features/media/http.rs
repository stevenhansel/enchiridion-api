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
    domain::{CropArgs, MediaType},
    error::{CreateMediaError, MediaErrorCode},
    service::{CreateMediaParams, MediaServiceInterface},
};

pub struct CreateMediaMultipart {
    pub media: TmpFile,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
    pub crop_args: Option<CropArgs>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMediaResponse {
    pub id: i32,
    pub path: String,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
}

pub async fn parse_create_announcement_multipart(
    mut payload: Multipart,
) -> Result<CreateMediaMultipart, String> {
    let mut media: Option<TmpFile> = None;
    let mut media_type: Option<MediaType> = None;
    let mut media_duration: Option<f64> = None;
    let mut crop_args: Option<CropArgs> = None;

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
        } else if field.name() == "mediaType" {
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
        } else if field.name() == "mediaDuration" {
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
        } else if field.name() == "crop" {
            let mut chunks: Vec<Bytes> = vec![];
            while let Some(chunk) = field.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };

                chunks.push(chunk);
            }

            if let Ok(raw) = std::str::from_utf8(&chunks[0]) {
                if raw != "null" {
                    if let Ok(args) = serde_json::from_str::<CropArgs>(raw) {
                        crop_args = Some(args);
                    }
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
        crop_args,
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

    let result = match media_service
        .create_media(CreateMediaParams {
            media: form.media,
            media_type: form.media_type,
            media_duration: form.media_duration,
            crop_args: form.crop_args,
        })
        .await
    {
        Ok(id) => id,
        Err(e) => match e {
            CreateMediaError::Database(_) | CreateMediaError::Unknown => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    MediaErrorCode::InternalServerError.to_string(),
                    vec![MediaErrorCode::InternalServerError.to_string()],
                ))
            }
        },
    };

    HttpResponse::Created().json(CreateMediaResponse {
        id: result.id,
        path: result.path,
        media_type: result.media_type,
        media_duration: result.media_duration,
    })
}
