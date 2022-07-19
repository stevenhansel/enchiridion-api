use std::sync::Arc;

use async_trait::async_trait;

use crate::database::DatabaseError;

use super::{CreateDeviceError, DeviceRepositoryInterface, InsertDeviceParams};

pub struct CreateDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
    pub is_linked: bool,
}

#[async_trait]
pub trait DeviceServiceInterface {
    async fn create_device(&self, params: CreateDeviceParams) -> Result<(), CreateDeviceError>;
}

pub struct DeviceService {
    _device_repository: Arc<dyn DeviceRepositoryInterface + Send + Sync + 'static>,
}

impl DeviceService {
    pub fn new(
        _device_repository: Arc<dyn DeviceRepositoryInterface + Send + Sync + 'static>,
    ) -> Self {
        DeviceService { _device_repository }
    }
}

#[async_trait]
impl DeviceServiceInterface for DeviceService {
    async fn create_device(&self, params: CreateDeviceParams) -> Result<(), CreateDeviceError> {
        if let Err(e) = self
            ._device_repository
            .insert(InsertDeviceParams {
                name: params.name.clone(),
                description: params.description.clone(),
                floor_id: params.floor_id,
                is_linked: params.is_linked,
            })
            .await
        {
            match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(CreateDeviceError::DeviceAlreadyExists(
                                format!(
                                    "Device with the name of {} already exists",
                                    params.name.clone()
                                )
                                .into(),
                            ));
                        } else if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(CreateDeviceError::FloorNotFound("Floor not found".into()));
                        }
                    }
                    return Err(CreateDeviceError::InternalServerError);
                }
                _ => return Err(CreateDeviceError::InternalServerError),
            }
        }

        Ok(())
    }
}
