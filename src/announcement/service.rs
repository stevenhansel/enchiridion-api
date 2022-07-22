use std::sync::Arc;

use async_trait::async_trait;

use super::{AnnouncementRepositoryInterface, CreateAnnouncementError};

pub struct CreateAnnouncementParams {}

#[async_trait]
pub trait AnnouncementServiceInterface {
    async fn create_announcement(&self, params: CreateAnnouncementParams) -> Result<(), CreateAnnouncementError>;
}

pub struct AnnouncementService {
    _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
}

impl AnnouncementService {
    pub fn new(_announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static> ) -> Self {
        AnnouncementService { _announcement_repository }
    }
}

#[async_trait]
impl AnnouncementServiceInterface for AnnouncementService {
    async fn create_announcement(&self, params: CreateAnnouncementParams) -> Result<(), CreateAnnouncementError> {
        Ok(())
    }
}
