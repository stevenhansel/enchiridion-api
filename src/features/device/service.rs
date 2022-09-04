use std::sync::Arc;

use async_trait::async_trait;

use crate::database::{DatabaseError, PaginationResult};

use super::{
    CreateDeviceError, DeleteDeviceError, Device, DeviceDetail, DeviceRepositoryInterface,
    GetDeviceDetailByIdError, InsertDeviceParams, ListDeviceError, ListDeviceParams,
    UpdateDeviceError, UpdateDeviceParams,
};

pub struct CreateDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
    pub is_linked: bool,
}

pub struct UpdateDeviceInfoParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
}

#[async_trait]
pub trait DeviceServiceInterface {
    async fn list_device(
        &self,
        params: ListDeviceParams,
    ) -> Result<PaginationResult<Device>, ListDeviceError>;
    async fn get_device_detail_by_id(
        &self,
        device_id: i32,
    ) -> Result<DeviceDetail, GetDeviceDetailByIdError>;
    async fn create_device(&self, params: CreateDeviceParams) -> Result<i32, CreateDeviceError>;
    async fn update_device_info(
        &self,
        device_id: i32,
        params: UpdateDeviceInfoParams,
    ) -> Result<(), UpdateDeviceError>;
    async fn delete_device(&self, device_id: i32) -> Result<(), DeleteDeviceError>;
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
    async fn list_device(
        &self,
        params: ListDeviceParams,
    ) -> Result<PaginationResult<Device>, ListDeviceError> {
        match self._device_repository.find(params).await {
            Ok(result) => Ok(result),
            Err(_) => Err(ListDeviceError::InternalServerError),
        }
    }

    async fn get_device_detail_by_id(
        &self,
        device_id: i32,
    ) -> Result<DeviceDetail, GetDeviceDetailByIdError> {
        match self._device_repository.find_one(device_id).await {
            Ok(result) => Ok(result),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Err(GetDeviceDetailByIdError::DeviceNotFound(
                    "Device not found".into(),
                )),
                _ => Err(GetDeviceDetailByIdError::InternalServerError),
            },
        }
    }

    async fn create_device(&self, params: CreateDeviceParams) -> Result<i32, CreateDeviceError> {
        match self
            ._device_repository
            .insert(InsertDeviceParams {
                name: params.name.clone(),
                description: params.description.clone(),
                floor_id: params.floor_id,
                is_linked: params.is_linked,
            })
            .await
        {
            Ok(id) => Ok(id),
            Err(e) => match e {
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
            },
        }
    }

    async fn update_device_info(
        &self,
        device_id: i32,
        params: UpdateDeviceInfoParams,
    ) -> Result<(), UpdateDeviceError> {
        if let Err(e) = self
            ._device_repository
            .update(
                device_id,
                UpdateDeviceParams {
                    name: params.name.clone(),
                    description: params.description.clone(),
                    floor_id: params.floor_id,
                },
            )
            .await
        {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(UpdateDeviceError::DeviceNotFound("Device not found".into()))
                }
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(UpdateDeviceError::DeviceAlreadyExists(
                                format!(
                                    "Device with the name of {} already exists",
                                    params.name.clone()
                                )
                                .into(),
                            ));
                        } else if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(UpdateDeviceError::FloorNotFound("Floor not found".into()));
                        }
                    }
                    return Err(UpdateDeviceError::InternalServerError);
                }
                _ => return Err(UpdateDeviceError::InternalServerError),
            }
        }

        Ok(())
    }

    async fn delete_device(&self, device_id: i32) -> Result<(), DeleteDeviceError> {
        if let Err(e) = self._device_repository.delete(device_id).await {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(DeleteDeviceError::DeviceNotFound("Device not found".into()))
                }
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(DeleteDeviceError::DeviceCascadeConstraint(
                                "Unable to delete device because it still have existing announcements".into(),
                            ));
                        }
                    }

                    return Err(DeleteDeviceError::InternalServerError);
                }
                _ => return Err(DeleteDeviceError::InternalServerError),
            }
        }

        Ok(())
    }
}
