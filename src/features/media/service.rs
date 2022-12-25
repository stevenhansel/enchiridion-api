use std::sync::Arc;

use async_trait::async_trait;

use crate::cloud_storage::{self, TmpFile};

use super::{
    domain::MediaType,
    error::CreateMediaError,
    repository::{InsertMediaParams, MediaRepositoryInterface},
};

pub struct CreateMediaParams {
    pub media: TmpFile,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
    // TODO: add crop coordinates,
}

#[async_trait]
pub trait MediaServiceInterface: Send + Sync + 'static {
    async fn create_media(&self, params: CreateMediaParams) -> Result<i32, CreateMediaError>;
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
    async fn create_media(&self, params: CreateMediaParams) -> Result<i32, CreateMediaError> {
        let path = params.media.key();
        // Command::new("ffmpeg");

        // TODO: if media_type video crop video according to the coordinates
        if self._cloud_storage.upload(params.media).await.is_err() {
            return Err(CreateMediaError::Unknown);
        }

        let id = self
            ._media_repository
            .insert(InsertMediaParams {
                path,
                media_type: params.media_type,
                media_duration: params.media_duration,
            })
            .await?;

        Ok(id)
    }
}
