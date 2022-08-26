use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    announcement::{
        AnnouncementQueueInterface, AnnouncementRepositoryInterface, AnnouncementStatus,
    },
    auth::AuthRepositoryInterface,
    database::{DatabaseError, PaginationResult},
};

use super::{
    CreateRequestError, FindRequestParams, InsertRequestParams, ListRequestError, Request,
    RequestActionType, RequestRepositoryInterface, UpdateApprovalParams,
    UpdateRequestApprovalError,
};

pub struct ListRequestParams {
    pub page: i32,
    pub limit: i32,
    pub request_id: Option<i32>,
    pub announcement_id: Option<i32>,
    pub user_id: Option<i32>,
    pub action_type: Option<RequestActionType>,
    pub approved_by_lsc: Option<bool>,
    pub approved_by_bm: Option<bool>,
}

pub struct CreateRequestParams {
    pub action: RequestActionType,
    pub description: String,
    pub announcement_id: i32,
    pub user_id: i32,
}

pub struct UpdateRequestApprovalParams {
    pub request_id: i32,
    pub approver_id: i32,
    pub approval: bool,
}

#[async_trait]
pub trait RequestServiceInterface {
    async fn list_request(
        &self,
        params: ListRequestParams,
    ) -> Result<PaginationResult<Request>, ListRequestError>;
    async fn create_request(&self, params: CreateRequestParams) -> Result<(), CreateRequestError>;

    async fn update_request_approval(
        &self,
        params: UpdateRequestApprovalParams,
    ) -> Result<(), UpdateRequestApprovalError>;
}

pub struct RequestService {
    _request_repository: Arc<dyn RequestRepositoryInterface + Send + Sync + 'static>,
    _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
    _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
    _auth_repository: Arc<dyn AuthRepositoryInterface + Send + Sync + 'static>,
}

impl RequestService {
    pub fn new(
        _request_repository: Arc<dyn RequestRepositoryInterface + Send + Sync + 'static>,
        _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
        _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
        _auth_repository: Arc<dyn AuthRepositoryInterface + Send + Sync + 'static>,
    ) -> Self {
        RequestService {
            _request_repository,
            _announcement_repository,
            _announcement_queue,
            _auth_repository,
        }
    }
}
#[async_trait]
impl RequestServiceInterface for RequestService {
    async fn list_request(
        &self,
        params: ListRequestParams,
    ) -> Result<PaginationResult<Request>, ListRequestError> {
        match self
            ._request_repository
            .find(FindRequestParams {
                page: params.page,
                limit: params.limit,
                request_id: params.request_id,
                announcement_id: params.announcement_id,
                user_id: params.user_id,
                action_type: params.action_type,
                approved_by_lsc: params.approved_by_lsc,
                approved_by_bm: params.approved_by_bm,
            })
            .await
        {
            Ok(result) => Ok(result),
            Err(_) => Err(ListRequestError::InternalServerError),
        }
    }

    async fn create_request(&self, params: CreateRequestParams) -> Result<(), CreateRequestError> {
        if let Err(e) = self
            ._request_repository
            .insert(InsertRequestParams {
                action: params.action,
                description: params.description,
                announcement_id: params.announcement_id,
                user_id: params.user_id,
            })
            .await
        {
            match e {
                sqlx::Error::Database(db_error) => {
                    if let Some(code) = db_error.code() {
                        let code = code.to_string();
                        if code == DatabaseError::ForeignKeyError.to_string() {
                            return Err(CreateRequestError::EntityNotFound(
                                "Announcement or User not found".into(),
                            ));
                        }
                    }

                    return Err(CreateRequestError::InternalServerError);
                }
                _ => return Err(CreateRequestError::InternalServerError),
            }
        }

        Ok(())
    }

    async fn update_request_approval(
        &self,
        params: UpdateRequestApprovalParams,
    ) -> Result<(), UpdateRequestApprovalError> {
        let request = match self._request_repository.find_one(params.request_id).await {
            Ok(data) => data,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(UpdateRequestApprovalError::RequestNotFound(
                        "Request not found".into(),
                    ))
                }
                _ => return Err(UpdateRequestApprovalError::InternalServerError),
            },
        };

        let approver = match self
            ._auth_repository
            .find_one_auth_entity_by_id(params.approver_id)
            .await
        {
            Ok(entity) => entity,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(UpdateRequestApprovalError::UserNotFound(
                        "Approver not found".into(),
                    ))
                }
                _ => return Err(UpdateRequestApprovalError::InternalServerError),
            },
        };

        let approval_whitelist: Vec<&str> = vec!["LSC", "BM"];
        if !approval_whitelist.contains(&approver.role.name.as_str()) {
            return Err(UpdateRequestApprovalError::UserForbiddenToApprove(
                "User is not allowed to approve a request".into(),
            ));
        }

        let mut approved_by_lsc = request.approved_by_lsc;
        let mut lsc_approver = request.lsc_approver;
        let mut approved_by_bm = request.approved_by_bm;
        let mut bm_approver = request.bm_approver;

        if approver.role.name.as_str() == "LSC" {
            if request.approved_by_lsc == Some(true) || request.approved_by_lsc == Some(false) {
                return Err(UpdateRequestApprovalError::RequestAlreadyApproved(
                    "Request already approved".into(),
                ));
            }

            approved_by_lsc = Some(params.approval);
            lsc_approver = Some(approver.id);
        } else if approver.role.name.as_str() == "BM" {
            if request.approved_by_bm == Some(true) || request.approved_by_bm == Some(false) {
                return Err(UpdateRequestApprovalError::RequestAlreadyApproved(
                    "Request already approved".into(),
                ));
            }

            approved_by_bm = Some(params.approval);
            bm_approver = Some(approver.id);
        }

        let announcement = match self
            ._announcement_repository
            .find_one(request.announcement_id)
            .await
        {
            Ok(data) => data,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(UpdateRequestApprovalError::AnnouncementNotFound(
                        "Announcement not found".into(),
                    ))
                }
                _ => return Err(UpdateRequestApprovalError::InternalServerError),
            },
        };

        if request.action == RequestActionType::Create {
            if announcement.status != AnnouncementStatus::WaitingForApproval {
                return Err(UpdateRequestApprovalError::InvalidAnnouncementStatus(
                    "Announcement status should be Waiting for Approval".into(),
                ));
            }

            if let Err(_) = self
                ._request_repository
                .update_approval(UpdateApprovalParams {
                    request_id: request.id,
                    approved_by_lsc,
                    approved_by_bm,
                    lsc_approver,
                    bm_approver,
                })
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }

            if approved_by_bm == Some(true) && approved_by_lsc == Some(true) {
                if let Err(_) = self
                    ._announcement_repository
                    .update_status(announcement.id, AnnouncementStatus::Active)
                    .await
                {
                    return Err(UpdateRequestApprovalError::InternalServerError);
                }
                let announcement_queue = self._announcement_queue.clone();

                let device_ids: Vec<i32> = announcement
                    .devices
                    .into_iter()
                    .map(|device| device.id)
                    .collect();
                let announcement_id = announcement.id;
                if let Err(_) = announcement_queue
                    .synchronize_create_announcement_action_to_devices(device_ids, announcement_id)
                {
                    // TODO: handle the Error
                };
                // TODO: sync to devices queue
            } else if approved_by_bm == Some(false) || approved_by_lsc == Some(false) {
                if let Err(_) = self
                    ._announcement_repository
                    .update_status(announcement.id, AnnouncementStatus::Rejected)
                    .await
                {
                    return Err(UpdateRequestApprovalError::InternalServerError);
                }
            }
        } else if request.action == RequestActionType::Delete {
            // TODO: handle delete here
        }

        Ok(())
    }
}
