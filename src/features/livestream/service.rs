use async_trait::async_trait;

#[async_trait]
pub trait LivestreamServiceInterface: Send + Sync + 'static {}

pub struct LivestreamService {}
