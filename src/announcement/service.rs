use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    cloud_storage::{self, TmpFile},
    database::DatabaseError,
    request::{CreateRequestParams, RequestServiceInterface},
};

use super::{AnnouncementRepositoryInterface, CreateAnnouncementError, InsertAnnouncementParams};

pub struct CreateAnnouncementParams {
    pub title: String,
    pub media: TmpFile,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub notes: String,
    pub device_ids: Vec<i32>,
    pub user_id: i32,
}

#[async_trait]
pub trait AnnouncementServiceInterface {
    async fn create_announcement(
        &self,
        params: CreateAnnouncementParams,
    ) -> Result<(), CreateAnnouncementError>;
}

pub struct AnnouncementService {
    _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
    _request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
    _cloud_storage: cloud_storage::Client,
}

impl AnnouncementService {
    pub fn new(
        _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
        _request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
        _cloud_storage: cloud_storage::Client,
    ) -> Self {
        AnnouncementService {
            _announcement_repository,
            _request_service,
            _cloud_storage,
        }
    }
}

#[async_trait]
impl AnnouncementServiceInterface for AnnouncementService {
    async fn create_announcement(
        &self,
        params: CreateAnnouncementParams,
    ) -> Result<(), CreateAnnouncementError> {
        // TODO: use db transaction if fail
        let media = params.media.key();

        let announcement_id = match self
            ._announcement_repository
            .insert(InsertAnnouncementParams {
                title: params.title.clone(),
                start_date: params.start_date,
                end_date: params.end_date,
                notes: params.notes.clone(),
                device_ids: params.device_ids,
                user_id: params.user_id,
                media,
            })
            .await
        {
            Ok(id) => id,
            Err(e) => match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(CreateAnnouncementError::UserNotFound(
                                "User not found".into(),
                            ));
                        }
                    }
                    return Err(CreateAnnouncementError::InternalServerError);
                }
                _ => return Err(CreateAnnouncementError::InternalServerError),
            },
        };

        if let Err(_) = self
            ._request_service
            .create_request_action_type_create(CreateRequestParams {
                description: params.notes.clone(),
                user_id: params.user_id,
                announcement_id,
            })
            .await
        {
            return Err(CreateAnnouncementError::InternalServerError);
        }

        if let Err(_) = self._cloud_storage.upload(params.media).await {
            return Err(CreateAnnouncementError::InternalServerError);
        }

        // let uri = match self
        //     ._cloud_storage
        //     .get_object(format!("{}/{}", key, name))
        //     .await {
        //         Ok(uri) => uri,
        //         Err(e) => return Err(CreateAnnouncementError::InternalServerError),
        //     };

        Ok(())
    }
}
