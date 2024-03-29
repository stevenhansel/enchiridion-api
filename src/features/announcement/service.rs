use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    cloud_storage,
    database::{DatabaseError, PaginationResult},
    features::{
        request::{CreateRequestParams, RequestActionType, RequestServiceInterface},
        AnnouncementQueueInterface,
    },
};

use super::{
    Announcement, AnnouncementDetail, AnnouncementMediaObject, AnnouncementRepositoryInterface,
    AnnouncementStatus, CountAnnouncementParams, CreateAnnouncementError,
    FindListAnnouncementParams, GetAnnouncementDetailError, GetAnnouncementMediaPresignedURLError,
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
    pub populate_media: Option<bool>,
}

pub struct CreateAnnouncementParams {
    pub title: String,
    pub media_id: i32,
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
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), HandleScheduledAnnouncementsError>;
    async fn handle_waiting_for_sync_announcements(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), HandleScheduledAnnouncementsError>;
    async fn handle_active_announcements(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), HandleScheduledAnnouncementsError>;
}

pub struct AnnouncementService {
    _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
    _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
    _request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
    _cloud_storage: cloud_storage::Client,
}

impl AnnouncementService {
    pub fn new(
        _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
        _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
        _request_service: Arc<dyn RequestServiceInterface + Send + Sync + 'static>,
        _cloud_storage: cloud_storage::Client,
    ) -> Self {
        AnnouncementService {
            _announcement_repository,
            _announcement_queue,
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
        let mut repo_params = FindListAnnouncementParams::default()
            .page(params.page)
            .limit(params.limit);

        if let Some(query) = params.query {
            repo_params = repo_params.query(query);
        }
        if let Some(status) = params.status {
            repo_params = repo_params.status(status);
        }
        if let Some(user_id) = params.user_id {
            repo_params = repo_params.user_id(user_id);
        }
        if let Some(device_id) = params.device_id {
            repo_params = repo_params.device_id(device_id);
        }
        if let Some(start_date) = params.start_date {
            repo_params = repo_params.start_date_gte(start_date);
        }
        if let Some(end_date) = params.end_date {
            repo_params = repo_params.end_date_lte(end_date);
        }

        let mut announcements = match self._announcement_repository.find(repo_params).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("e: {:?}", e);
                return Err(ListAnnouncementError::InternalServerError);
            }
        };

        if let Some(populate_media) = params.populate_media {
            if populate_media {
                let mut contents: Vec<Announcement> = vec![];
                for content in announcements.contents {
                    let media_object =
                        match self.get_announcement_media_presigned_url(content.id).await {
                            Ok(media) => media,
                            Err(_) => return Err(ListAnnouncementError::InternalServerError),
                        };

                    contents.push(Announcement {
                        media: media_object.media,
                        ..content
                    });
                }

                announcements.contents = contents;
            }
        }

        Ok(announcements)
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
        let announcement_id = match self
            ._announcement_repository
            .insert(InsertAnnouncementParams {
                title: params.title.clone(),
                notes: params.notes.clone(),
                start_date: params.start_date,
                end_date: params.end_date,
                device_ids: params.device_ids,
                user_id: params.user_id,
                media_id: params.media_id,
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
            .create_request(CreateRequestParams::new(
                RequestActionType::Create,
                params.notes.clone(),
                announcement_id,
                params.user_id,
            ))
            .await
        {
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
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), HandleScheduledAnnouncementsError> {
        let announcement_ids = match self
            ._announcement_repository
            .find_expired_waiting_for_approval_announcement_ids(now)
            .await
        {
            Ok(ids) => ids,
            Err(_) => {
                return Err(HandleScheduledAnnouncementsError::InternalServerError);
            }
        };

        if announcement_ids.len() == 0 {
            return Ok(());
        }

        if let Err(_) = self
            ._announcement_repository
            .batch_update_status(announcement_ids.clone(), AnnouncementStatus::Rejected)
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        if let Err(_) = self
            ._request_service
            .batch_reject_requests_from_announcement_ids(announcement_ids.clone())
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        Ok(())
    }

    async fn handle_waiting_for_sync_announcements(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), HandleScheduledAnnouncementsError> {
        let count = match self
            ._announcement_repository
            .count(
                CountAnnouncementParams::default()
                    .status(AnnouncementStatus::WaitingForSync)
                    .start_date_gte(now),
            )
            .await
        {
            Ok(count) => count,
            Err(_) => {
                return Err(HandleScheduledAnnouncementsError::InternalServerError);
            }
        };

        if count == 0 {
            return Ok(());
        }

        let announcements = match self
            ._announcement_repository
            .find(
                FindListAnnouncementParams::default()
                    .limit(count)
                    .status(AnnouncementStatus::WaitingForSync)
                    .start_date_gte(now),
            )
            .await
        {
            Ok(data) => data,
            Err(_) => return Err(HandleScheduledAnnouncementsError::InternalServerError),
        };

        let announcement_data: Vec<(i32, String, Option<f64>)> = announcements
            .contents
            .into_iter()
            .map(|announcement| {
                (
                    announcement.id,
                    announcement.media_type.to_string(),
                    announcement.media_duration,
                )
            })
            .collect();

        let announcement_ids: Vec<i32> = announcement_data.iter().map(|data| data.0).collect();

        let announcement_device_map = match self
            ._announcement_repository
            .find_announcement_device_map(announcement_ids.clone())
            .await
        {
            Ok(map) => map,
            Err(_) => return Err(HandleScheduledAnnouncementsError::InternalServerError),
        };

        for (id, media_type, media_duration) in &announcement_data {
            let device_ids = match announcement_device_map.get(id) {
                Some(ids) => ids,
                None => return Err(HandleScheduledAnnouncementsError::InternalServerError),
            };

            if let Err(_) = self
                ._announcement_queue
                .create(
                    device_ids.clone(),
                    *id,
                    media_type.to_string(),
                    *media_duration,
                )
                .await
            {
                return Err(HandleScheduledAnnouncementsError::InternalServerError);
            }
        }

        if let Err(_) = self
            ._announcement_repository
            .batch_update_status(announcement_ids.clone(), AnnouncementStatus::Active)
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        Ok(())
    }

    async fn handle_active_announcements(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), HandleScheduledAnnouncementsError> {
        let count = match self
            ._announcement_repository
            .count(
                CountAnnouncementParams::default()
                    .status(AnnouncementStatus::Active)
                    .end_date_lte(now),
            )
            .await
        {
            Ok(count) => count,
            Err(_) => {
                return Err(HandleScheduledAnnouncementsError::InternalServerError);
            }
        };

        if count == 0 {
            return Ok(());
        }

        let announcements = match self
            ._announcement_repository
            .find(
                FindListAnnouncementParams::default()
                    .limit(count)
                    .status(AnnouncementStatus::Active)
                    .end_date_lte(now),
            )
            .await
        {
            Ok(data) => data,
            Err(_) => return Err(HandleScheduledAnnouncementsError::InternalServerError),
        };

        let announcement_ids: Vec<i32> = announcements
            .contents
            .into_iter()
            .map(|announcement| announcement.id)
            .collect();

        let announcement_device_map = match self
            ._announcement_repository
            .find_announcement_device_map(announcement_ids.clone())
            .await
        {
            Ok(map) => map,
            Err(_) => return Err(HandleScheduledAnnouncementsError::InternalServerError),
        };

        for id in &announcement_ids {
            let device_ids = match announcement_device_map.get(id) {
                Some(ids) => ids,
                None => return Err(HandleScheduledAnnouncementsError::InternalServerError),
            };

            if let Err(_) = self
                ._announcement_queue
                .delete(device_ids.clone(), *id)
                .await
            {
                return Err(HandleScheduledAnnouncementsError::InternalServerError);
            }
        }

        if let Err(_) = self
            ._announcement_repository
            .batch_update_status(announcement_ids.clone(), AnnouncementStatus::Done)
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        if let Err(_) = self
            ._request_service
            .batch_reject_requests_from_announcement_ids(announcement_ids)
            .await
        {
            return Err(HandleScheduledAnnouncementsError::InternalServerError);
        }

        Ok(())
    }
}
