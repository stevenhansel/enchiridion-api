use std::sync::Arc;

use argon2::{password_hash::PasswordHasher, Argon2};
use async_trait::async_trait;
use rand::distributions::{Alphanumeric, DistString};

use crate::{
    database::{DatabaseError, PaginationResult},
    features::AnnouncementQueueInterface,
};

use super::{
    CreateDeviceError, DeleteDeviceError, Device, DeviceDetail, DeviceRepositoryInterface,
    GetDeviceDetailByIdError, InsertDeviceParams, ListDeviceError, ListDeviceParams,
    ResyncDeviceError, UpdateDeviceError, UpdateDeviceParams,
};

pub struct CreateDeviceParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
}

pub struct UpdateDeviceInfoParams {
    pub name: String,
    pub description: String,
    pub floor_id: i32,
}

pub struct CreateDeviceResult {
    pub id: i32,
    pub access_key_id: String,
    pub secret_access_key: String,
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
    async fn create_device(
        &self,
        params: CreateDeviceParams,
    ) -> Result<CreateDeviceResult, CreateDeviceError>;
    async fn update_device_info(
        &self,
        device_id: i32,
        params: UpdateDeviceInfoParams,
    ) -> Result<(), UpdateDeviceError>;
    async fn delete_device(&self, device_id: i32) -> Result<(), DeleteDeviceError>;
    async fn resync(&self, device_id: i32) -> Result<(), ResyncDeviceError>;
}

pub struct DeviceService {
    _device_repository: Arc<dyn DeviceRepositoryInterface + Send + Sync + 'static>,
    _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
}

impl DeviceService {
    pub fn new(
        _device_repository: Arc<dyn DeviceRepositoryInterface + Send + Sync + 'static>,
        _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
    ) -> Self {
        DeviceService {
            _device_repository,
            _announcement_queue,
        }
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
            Err(e) => {
                println!("e: {}", e);
                Err(ListDeviceError::InternalServerError)
            }
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

    async fn create_device(
        &self,
        params: CreateDeviceParams,
    ) -> Result<CreateDeviceResult, CreateDeviceError> {
        let access_key_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 48);

        let secret_access_key = Alphanumeric.sample_string(&mut rand::thread_rng(), 48);
        let secret_access_key_salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

        let secret_access_key_hash = match Argon2::default()
            .hash_password(secret_access_key.as_bytes(), &secret_access_key_salt)
        {
            Ok(p) => p.serialize(),
            Err(_) => return Err(CreateDeviceError::InternalServerError),
        };

        let id = match self
            ._device_repository
            .insert(InsertDeviceParams {
                name: params.name.clone(),
                description: params.description.clone(),
                floor_id: params.floor_id,
                access_key_id: access_key_id.clone(),
                secret_access_key: secret_access_key_hash.to_string(),
                secret_access_key_salt: secret_access_key_salt.clone(),
            })
            .await
        {
            Ok(id) => id,
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
        };

        Ok(CreateDeviceResult {
            id,
            access_key_id,
            secret_access_key,
        })
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

    async fn resync(&self, device_id: i32) -> Result<(), ResyncDeviceError> {
        if let Err(e) = self._device_repository.find_one(device_id).await {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(ResyncDeviceError::DeviceNotFound(
                        "Unable to find the device in the system",
                    ))
                }
                _ => return Err(ResyncDeviceError::InternalServerError),
            }
        }

        let announcement_ids = match self
            ._device_repository
            .find_announcement_ids_in_device(device_id)
            .await
        {
            Ok(ids) => ids,
            Err(_) => return Err(ResyncDeviceError::InternalServerError),
        };

        if let Err(_) = self._announcement_queue.resync(device_id, announcement_ids) {
            return Err(ResyncDeviceError::InternalServerError);
        }

        Ok(())
    }
}
