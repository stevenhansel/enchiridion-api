use std::sync::Arc;

use async_trait::async_trait;

use crate::building::{BuildingRepositoryInterface, InsertBuildingParams, UpdateBuildingParams};
use crate::database::DatabaseError;

use super::{domain::Building, BuildingError};

pub struct CreateParams {
    pub name: String,
    pub color: String,
}

pub struct UpdateParams {
    pub id: i32,
    pub name: String,
    pub color: String,
}

#[async_trait]
pub trait BuildingServiceInterface {
    async fn get_buildings(&self) -> Result<Vec<Building>, BuildingError>;
    async fn create(&self, params: CreateParams) -> Result<i32, BuildingError>;
    async fn update(&self, params: UpdateParams) -> Result<i32, BuildingError>;
    async fn delete_by_id(&self, id: i32) -> Result<i32, BuildingError>;
}

pub struct BuildingService {
    _building_repository: Arc<dyn BuildingRepositoryInterface + Send + Sync + 'static>,
}

impl BuildingService {
    pub fn new(
        _building_repository: Arc<dyn BuildingRepositoryInterface + Send + Sync + 'static>,
    ) -> BuildingService {
        BuildingService {
            _building_repository,
        }
    }
}

#[async_trait]
impl BuildingServiceInterface for BuildingService {
    async fn get_buildings(&self) -> Result<Vec<Building>, BuildingError> {
        let buildings = match self._building_repository.find_buildings().await {
            Ok(buildings) => buildings,
            Err(e) => match e {
                _ => return Err(BuildingError::InternalServerError),
            },
        };

        Ok(buildings)
    }

    async fn create(&self, params: CreateParams) -> Result<i32, BuildingError> {
        let id = match self
            ._building_repository
            .create(InsertBuildingParams {
                name: params.name,
                color: params.color,
            })
            .await
        {
            Ok(id) => id,
            Err(e) => match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(BuildingError::BuildingNameAlreadyExists(
                                "Building Name is already registered in our system".into(),
                            ));
                        }
                    }

                    return Err(BuildingError::InternalServerError);
                }
                sqlx::Error::RowNotFound => {
                    return Err(BuildingError::BuildingNotFound("Building not found".into()));
                }
                _ => return Err(BuildingError::InternalServerError),
            },
        };

        Ok(id)
    }

    async fn update(&self, params: UpdateParams) -> Result<i32, BuildingError> {
        let id = match self
            ._building_repository
            .update(UpdateBuildingParams {
                id: params.id,
                name: params.name,
                color: params.color,
            })
            .await
        {
            Ok(id) => id,
            Err(e) => match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(BuildingError::BuildingNameAlreadyExists(
                                "Building Name is already registered in our system".into(),
                            ));
                        }
                    }

                    return Err(BuildingError::InternalServerError);
                }
                sqlx::Error::RowNotFound => {
                    return Err(BuildingError::BuildingNotFound("Building not found".into()));
                }
                _ => return Err(BuildingError::InternalServerError),
            },
        };

        Ok(id)
    }

    async fn delete_by_id(&self, id: i32) -> Result<i32, BuildingError> {
        let id = match self._building_repository.delete_by_id(id).await {
            Ok(id) => id,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(BuildingError::BuildingNotFound("Building not found".into()));
                }
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(BuildingError::BuildingCascadeConstraint(
                                "Unable to delete building because it still have existing floors".into(),
                            ));
                        }
                    }

                    return Err(BuildingError::InternalServerError);
                }
                _ => return Err(BuildingError::InternalServerError),
            },
        };

        Ok(id)
    }
}
