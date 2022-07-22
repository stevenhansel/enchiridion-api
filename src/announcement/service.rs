use std::sync::Arc;

use async_trait::async_trait;

use crate::cloud_storage::{self, TmpFile};

use super::{AnnouncementRepositoryInterface, CreateAnnouncementError};

pub struct CreateAnnouncementParams {
    pub media: TmpFile,
}

#[async_trait]
pub trait AnnouncementServiceInterface {
    async fn create_announcement(
        &self,
        params: CreateAnnouncementParams,
    ) -> Result<(), CreateAnnouncementError>;
}

pub struct AnnouncementService {
    _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
    _cloud_storage: cloud_storage::Client,
}

impl AnnouncementService {
    pub fn new(
        _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
        _cloud_storage: cloud_storage::Client,
    ) -> Self {
        AnnouncementService {
            _announcement_repository,
            _cloud_storage,
        }
    }
}

#[async_trait]
impl AnnouncementServiceInterface for AnnouncementService {
    async fn create_announcement(
        &self,
        params: CreateAnnouncementParams,
    ) -> Result<(), CreateAnnouncementError> {
        let key = params.media.key.clone();
        let name = params.media.name().clone();
        let url = match self._cloud_storage.upload(params.media).await {
            Ok(res) => res,
            Err(e) => return Err(CreateAnnouncementError::InternalServerError),
        };

        let uri = match self
            ._cloud_storage
            .get_object(format!("{}/{}", key, name))
            .await {
                Ok(uri) => uri,
                Err(e) => return Err(CreateAnnouncementError::InternalServerError),
            };
        println!("presigned: {}", uri);
        Ok(())
    }
}
