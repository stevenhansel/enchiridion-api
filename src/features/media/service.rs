use std::sync::Arc;

use async_trait::async_trait;

use crate::cloud_storage::{self, TmpFile};

use super::{
    domain::{CreateMediaResult, CropArgs, MediaType},
    error::CreateMediaError,
    repository::{InsertMediaParams, MediaRepositoryInterface},
};

pub struct CreateMediaParams {
    pub media: TmpFile,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
    pub crop_args: Option<CropArgs>,
}

#[async_trait]
pub trait MediaServiceInterface: Send + Sync + 'static {
    async fn create_media(
        &self,
        params: CreateMediaParams,
    ) -> Result<CreateMediaResult, CreateMediaError>;
}

pub struct MediaService {
    _media_repository: Arc<dyn MediaRepositoryInterface>,
    _cloud_storage: cloud_storage::Client,
}

impl MediaService {
    pub fn new(
        _media_repository: Arc<dyn MediaRepositoryInterface>,
        _cloud_storage: cloud_storage::Client,
    ) -> Self {
        MediaService {
            _media_repository,
            _cloud_storage,
        }
    }
}

#[async_trait]
impl MediaServiceInterface for MediaService {
    async fn create_media(
        &self,
        params: CreateMediaParams,
    ) -> Result<CreateMediaResult, CreateMediaError> {
        let mut media = params.media;
        if let Some(args) = params.crop_args {
            media = match media.crop(args).await {
                Ok(m) => m,
                Err(_) => return Err(CreateMediaError::Unknown),
            };
        }

        let path = media.key();

        if self._cloud_storage.upload(media).await.is_err() {
            println!("error 1");
            return Err(CreateMediaError::Unknown);
        }

        let id = self
            ._media_repository
            .insert(InsertMediaParams {
                path: path.clone(),
                media_type: params.media_type.clone(),
                media_duration: params.media_duration,
            })
            .await?;

        let path = match self._cloud_storage.get_object(path).await {
            Ok(path) => path,
            Err(_) => return Err(CreateMediaError::Unknown),
        };

        Ok(CreateMediaResult {
            id,
            path,
            media_type: params.media_type,
            media_duration: params.media_duration,
        })
    }
}
