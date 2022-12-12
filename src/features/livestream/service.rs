use std::sync::Arc;

use async_trait::async_trait;

use super::{
    definition::LivestreamMessagePayload, error::InsertLivestreamError,
    repository::LivestreamRepositoryInterface,
};

#[async_trait]
pub trait LivestreamServiceInterface: Send + Sync + 'static {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), InsertLivestreamError>;
}

pub struct LivestreamService {
    _livestream_repository: Arc<dyn LivestreamRepositoryInterface>,
}

impl LivestreamService {
    pub fn new(_livestream_repository: Arc<dyn LivestreamRepositoryInterface>) -> Self {
        LivestreamService {
            _livestream_repository,
        }
    }
}

#[async_trait]
impl LivestreamServiceInterface for LivestreamService {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), InsertLivestreamError> {
        Ok(self._livestream_repository.insert(message).await?)
    }
}
