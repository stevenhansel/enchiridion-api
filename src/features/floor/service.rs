use std::sync::Arc;

use async_trait::async_trait;

use super::{
    CreateFloorError, DeleteFloorError, FindFloorParams, Floor, FloorRepositoryInterface,
    InsertFloorParams, ListFloorError, UpdateFloorError, UpdateFloorParams,
};

use crate::database::{DatabaseError, PaginationResult};

pub struct ListFloorParams {
    pub page: i32,
    pub limit: i32,
    pub building_id: Option<i32>,
    pub query: Option<String>,
}

pub struct CreateFloorParams {
    pub name: String,
    pub building_id: i32,
}

pub struct UpdateFloorInfoParams {
    pub name: String,
    pub building_id: i32,
}

#[async_trait]
pub trait FloorServiceInterface {
    async fn list_floor(
        &self,
        params: ListFloorParams,
    ) -> Result<PaginationResult<Floor>, ListFloorError>;
    async fn create_floor(&self, params: CreateFloorParams) -> Result<(), CreateFloorError>;
    async fn update_floor(
        &self,
        floor_id: i32,
        params: UpdateFloorInfoParams,
    ) -> Result<(), UpdateFloorError>;
    async fn delete_floor(&self, floor_id: i32) -> Result<(), DeleteFloorError>;
}

pub struct FloorService {
    _floor_repository: Arc<dyn FloorRepositoryInterface + Send + Sync + 'static>,
}

impl FloorService {
    pub fn new(
        _floor_repository: Arc<dyn FloorRepositoryInterface + Send + Sync + 'static>,
    ) -> Self {
        FloorService { _floor_repository }
    }
}

#[async_trait]
impl FloorServiceInterface for FloorService {
    async fn list_floor(
        &self,
        params: ListFloorParams,
    ) -> Result<PaginationResult<Floor>, ListFloorError> {
        match self
            ._floor_repository
            .find(FindFloorParams {
                page: params.page,
                limit: params.limit,
                building_id: params.building_id.clone(),
                query: params.query.clone(),
            })
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => match e {
                _ => {
                    println!("e: {}", e);
                    return Err(ListFloorError::InternalServerError);
                }
            },
        }
    }

    async fn create_floor(&self, params: CreateFloorParams) -> Result<(), CreateFloorError> {
        if let Err(e) = self
            ._floor_repository
            .insert(InsertFloorParams {
                name: params.name.clone(),
                building_id: params.building_id,
            })
            .await
        {
            match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::UniqueConstraintError.to_string() {
                            return Err(CreateFloorError::FloorAlreadyExists(
                                format!(
                                    "Floor with the name of {} already exists",
                                    params.name.clone()
                                )
                                .into(),
                            ));
                        } else if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(CreateFloorError::BuildingNotFound(
                                "Building not found".into(),
                            ));
                        }
                    }
                    return Err(CreateFloorError::InternalServerError);
                }
                _ => return Err(CreateFloorError::InternalServerError),
            }
        }

        Ok(())
    }

    async fn update_floor(
        &self,
        floor_id: i32,
        params: UpdateFloorInfoParams,
    ) -> Result<(), UpdateFloorError> {
        if let Err(e) = self
            ._floor_repository
            .update(
                floor_id,
                UpdateFloorParams {
                    name: params.name.clone(),
                    building_id: params.building_id,
                },
            )
            .await
        {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(UpdateFloorError::FloorNotFound("Floor not found".into()))
                }
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(UpdateFloorError::BuildingNotFound(
                                "Building not found".into(),
                            ));
                        }
                    }
                    return Err(UpdateFloorError::InternalServerError);
                }
                _ => return Err(UpdateFloorError::InternalServerError),
            }
        }

        Ok(())
    }

    async fn delete_floor(&self, floor_id: i32) -> Result<(), DeleteFloorError> {
        if let Err(e) = self._floor_repository.delete(floor_id).await {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(DeleteFloorError::FloorNotFound("Floor not found".into()))
                }
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(DeleteFloorError::DeviceCascadeConstraint(
                                "Floor still have devices can't delete".into(),
                            ));
                        }
                    }
                    return Err(DeleteFloorError::InternalServerError);
                }
                _ => return Err(DeleteFloorError::InternalServerError),
            }
        }

        Ok(())
    }
}
