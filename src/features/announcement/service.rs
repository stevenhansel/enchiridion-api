use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    cloud_storage::{self, TmpFile},
    database::{DatabaseError, PaginationResult},
    features::request::{CreateRequestParams, RequestActionType, RequestServiceInterface},
};

use super::{
    Announcement, AnnouncementDetail, AnnouncementMediaObject, AnnouncementRepositoryInterface,
    AnnouncementStatus, CreateAnnouncementError, FindListAnnouncementParams,
    GetAnnouncementDetailError, GetAnnouncementMediaPresignedURLError,
    HandleScheduledAnnouncementsError, InsertAnnouncementParams, ListAnnouncementError,
};

pub struct ListAnnouncementParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub status: Option<AnnouncementStatus>,
    pub user_id: Option<i32>,
    pub device_id: Option<i32>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

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
    async fn list_announcement(
        &self,
        params: ListAnnouncementParams,
    ) -> Result<PaginationResult<Announcement>, ListAnnouncementError>;
    async fn get_announcement_detail(
        &self,
        announcement_id: i32,
    ) -> Result<AnnouncementDetail, GetAnnouncementDetailError>;
    async fn create_announcement(
        &self,
        params: CreateAnnouncementParams,
    ) -> Result<(), CreateAnnouncementError>;
    async fn get_announcement_media_presigned_url(
        &self,
        announcement_id: i32,
    ) -> Result<AnnouncementMediaObject, GetAnnouncementMediaPresignedURLError>;
    async fn handle_waiting_for_approval_announcements(
        &self,
    ) -> Result<(), HandleScheduledAnnouncementsError>;
    async fn handle_waiting_for_sync_announcements(
        &self,
    ) -> Result<(), HandleScheduledAnnouncementsError>;
    async fn handle_active_announcements(&self) -> Result<(), HandleScheduledAnnouncementsError>;
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
    async fn list_announcement(
        &self,
        params: ListAnnouncementParams,
    ) -> Result<PaginationResult<Announcement>, ListAnnouncementError> {
        match self
            ._announcement_repository
            .find(FindListAnnouncementParams {
                page: params.page,
                limit: params.limit,
                query: params.query.clone(),
                status: params.status.clone(),
                user_id: params.user_id.clone(),
                device_id: params.device_id.clone(),
                start_date_gte: params.start_date.clone(),
                end_date_lte: params.end_date.clone(),

                start_date_lt: None,
            })
            .await
        {
            Ok(result) => Ok(result),
            Err(_) => {
                return Err(ListAnnouncementError::InternalServerError);
            }
        }
    }

    async fn get_announcement_detail(
        &self,
        announcement_id: i32,
    ) -> Result<AnnouncementDetail, GetAnnouncementDetailError> {
        let mut result = match self
            ._announcement_repository
            .find_one(announcement_id)
            .await
        {
            Ok(result) => result,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(GetAnnouncementDetailError::AnnouncementNotFound(
                        "Announcement not found".into(),
                    ))
                }
                _ => return Err(GetAnnouncementDetailError::InternalServerError),
            },
        };

        result.media = match self._cloud_storage.get_object(result.media).await {
            Ok(uri) => uri,
            Err(_) => return Err(GetAnnouncementDetailError::InternalServerError),
        };

        Ok(result)
    }

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
            .create_request(CreateRequestParams {
                action: RequestActionType::Create,
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

        Ok(())
    }

    async fn get_announcement_media_presigned_url(
        &self,
        announcement_id: i32,
    ) -> Result<AnnouncementMediaObject, GetAnnouncementMediaPresignedURLError> {
        let result = match self
            ._announcement_repository
            .find_one(announcement_id)
            .await
        {
            Ok(result) => result,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(GetAnnouncementMediaPresignedURLError::AnnouncementNotFound(
                        "Announcement not found".into(),
                    ))
                }
                _ => return Err(GetAnnouncementMediaPresignedURLError::InternalServerError),
            },
        };

        let media = match self._cloud_storage.get_object(result.media.clone()).await {
            Ok(uri) => uri,
            Err(_) => return Err(GetAnnouncementMediaPresignedURLError::InternalServerError),
        };

        let splits: Vec<String> = result
            .media
            .clone()
            .split("/")
            .map(|m| m.to_string())
            .collect();
        let filename = splits[1].clone();

        Ok(AnnouncementMediaObject { filename, media })
    }

    async fn handle_waiting_for_approval_announcements(
        &self,
    ) -> Result<(), HandleScheduledAnnouncementsError> {
        println!("bp 1");
        let announcement_ids = match self
            ._announcement_repository
            .find_expired_waiting_for_approval_announcement_ids(chrono::Utc::now())
            .await
        {
            Ok(ids) => ids,
            Err(_) => {
                return Err(HandleScheduledAnnouncementsError::InternalServerError);
            }
        };

        println!("bp 2");
       if announcement_ids.len() == 0 {
            return Ok(());
        }

        println!("bp 3");


        if let Err(_) = self
            ._announcement_repository
            .batch_update_status(announcement_ids.clone(), AnnouncementStatus::Rejected)
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        println!("bp 4");


        if let Err(_) = self
            ._request_service
            .batch_reject_requests_from_announcement_ids(announcement_ids.clone())
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        println!("bp 5");


        Ok(())
    }

    async fn handle_waiting_for_sync_announcements(
        &self,
    ) -> Result<(), HandleScheduledAnnouncementsError> {
        // TODO: fetch announcements that have waiting_for_sync status and call
        // announcement_sync_queue service
        Ok(())
    }

    async fn handle_active_announcements(&self) -> Result<(), HandleScheduledAnnouncementsError> {
        Ok(())
    }
}
