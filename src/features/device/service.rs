use std::str;
use std::sync::Arc;

use actix_web::http::header::HeaderMap;
use argon2::{password_hash::PasswordHasher, Argon2};
use async_trait::async_trait;
use lazy_static::lazy_static;
use lipsum::lipsum_words;
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;

use crate::{
    database::{DatabaseError, PaginationResult},
    features::AnnouncementQueueInterface,
};

use super::{
    AuthenticateDeviceError, CreateDeviceError, DeleteDeviceError, Device, DeviceAuthCache,
    DeviceDetail, DeviceRepositoryInterface, GetDeviceDetailByIdError, InsertDeviceParams,
    ListDeviceError, ListDeviceParams, ResyncDeviceError, UpdateDeviceError, UpdateDeviceParams,
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
    async fn authenticate(&self, headers: &HeaderMap) -> Result<i32, AuthenticateDeviceError>;
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
        lazy_static! {
            static ref WORD_REGEX: Regex = Regex::new(r"[a-zA-Z]+").unwrap();
        }
        let generate_random_word = |n: usize| {
            let words = lipsum_words(n).to_lowercase();
            let words: Vec<&str> = WORD_REGEX
                .find_iter(words.as_str())
                .map(|word| word.as_str())
                .collect();

            words.join("-")
        };
        let access_key_id = generate_random_word(4);

        let secret_access_key = generate_random_word(4);
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

        if let Err(_) = self
            ._announcement_queue
            .resync(device_id, announcement_ids)
            .await
        {
            return Err(ResyncDeviceError::InternalServerError);
        }

        Ok(())
    }

    async fn authenticate(&self, headers: &HeaderMap) -> Result<i32, AuthenticateDeviceError> {
        let get_header_value = |key: &'static str| match headers.get(key) {
            Some(value) => match value.to_str() {
                Ok(value) => Ok(value.to_string()),
                Err(_) => {
                    return Err(AuthenticateDeviceError::AuthenticationFailed(
                        "Authentication failed",
                    ))
                }
            },
            None => {
                return Err(AuthenticateDeviceError::AuthenticationFailed(
                    "Authentication failed",
                ))
            }
        };

        let access_key_id = get_header_value("access-key-id")?;
        let secret_access_key = get_header_value("secret-access-key")?;

        let device_auth: DeviceAuthCache = if let Ok(cache) = self
            ._device_repository
            .get_auth_cache(access_key_id.clone())
            .await
        {
            cache
        } else {
            let device = match self
                ._device_repository
                .find_one_by_access_key_id(access_key_id.clone())
                .await
            {
                Ok(device) => device,
                Err(e) => match e {
                    sqlx::Error::RowNotFound => {
                        return Err(AuthenticateDeviceError::DeviceNotFound(
                            "Unable to find the corresponding device in the system",
                        ))
                    }
                    _ => {
                        return Err(AuthenticateDeviceError::InternalServerError);
                    }
                },
            };

            let secret_access_key = match str::from_utf8(&device.secret_access_key) {
                Ok(v) => v,
                Err(e) => {
                    return Err(AuthenticateDeviceError::InternalServerError);
                }
            };

            let cache = DeviceAuthCache {
                device_id: device.id,
                secret_access_key: secret_access_key.to_string(),
                secret_access_key_salt: device.secret_access_key_salt,
            };

            if let Err(e) = self
                ._device_repository
                .set_auth_cache(access_key_id, cache.clone())
                .await
            {
                return Err(AuthenticateDeviceError::InternalServerError);
            }

            cache
        };

        let input_secret_access_key_hash = match Argon2::default().hash_password(
            secret_access_key.as_bytes(),
            &device_auth.secret_access_key_salt,
        ) {
            Ok(p) => p,
            Err(e) => {
                return Err(AuthenticateDeviceError::InternalServerError);
            }
        };

        if input_secret_access_key_hash.to_string() != device_auth.secret_access_key {
            return Err(AuthenticateDeviceError::AuthenticationFailed(
                "Authentication failed".into(),
            ));
        }

        Ok(device_auth.device_id)
    }
}
