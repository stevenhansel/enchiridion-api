use async_trait::async_trait;

#[async_trait]
pub trait LivestreamRepositoryInterface: Send + Sync + 'static {}

pub struct LivestreamRepository {}
