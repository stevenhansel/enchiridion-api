use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    database::{DatabaseError, PaginationResult},
    features::{
        announcement::{
            AnnouncementQueueInterface, AnnouncementRepositoryInterface, AnnouncementStatus,
        },
        auth::AuthRepositoryInterface,
        AnnouncementDetail, DeviceRepositoryInterface,
    },
};

use super::{
    BatchRejectRequestsFromAnnouncementIdsError, CreateRequestError, FindRequestParams,
    InsertRequestParams, ListRequestError, Request, RequestActionType, RequestApproval,
    RequestRepositoryInterface, UpdateApprovalParams, UpdateRequestApprovalError,
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

    pub extended_end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub new_device_ids: Option<Vec<i32>>,
}

impl CreateRequestParams {
    pub fn new(
        action: RequestActionType,
        description: String,
        announcement_id: i32,
        user_id: i32,
    ) -> Self {
        CreateRequestParams {
            action,
            description,
            announcement_id,
            user_id,

            extended_end_date: None,
            new_device_ids: None,
        }
    }

    pub fn extended_end_date(mut self, extended_end_date: chrono::DateTime<chrono::Utc>) -> Self {
        self.extended_end_date = Some(extended_end_date);
        self
    }

    pub fn new_device_ids(mut self, new_device_ids: Vec<i32>) -> Self {
        self.new_device_ids = Some(new_device_ids);
        self
    }
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
    async fn handle_update_request_approval_create(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError>;
    async fn handle_update_request_approval_extend_date(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError>;
    async fn handle_update_request_approval_delete(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError>;
    async fn handle_update_request_approval_change_devices(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError>;
    async fn batch_reject_requests_from_announcement_ids(
        &self,
        announcement_ids: Vec<i32>,
    ) -> Result<(), BatchRejectRequestsFromAnnouncementIdsError>;
}

pub struct RequestService {
    _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
    _request_repository: Arc<dyn RequestRepositoryInterface + Send + Sync + 'static>,
    _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
    _auth_repository: Arc<dyn AuthRepositoryInterface + Send + Sync + 'static>,
    _device_repository: Arc<dyn DeviceRepositoryInterface + Send + Sync + 'static>,
}

impl RequestService {
    pub fn new(
        _announcement_queue: Arc<dyn AnnouncementQueueInterface + Send + Sync + 'static>,
        _request_repository: Arc<dyn RequestRepositoryInterface + Send + Sync + 'static>,
        _announcement_repository: Arc<dyn AnnouncementRepositoryInterface + Send + Sync + 'static>,
        _auth_repository: Arc<dyn AuthRepositoryInterface + Send + Sync + 'static>,
        _device_repository: Arc<dyn DeviceRepositoryInterface + Send + Sync + 'static>,
    ) -> Self {
        RequestService {
            _announcement_queue,
            _request_repository,
            _announcement_repository,
            _auth_repository,
            _device_repository,
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
        let announcement = match self
            ._announcement_repository
            .find_one(params.announcement_id)
            .await
        {
            Ok(announcement) => announcement,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(CreateRequestError::AnnouncementNotFound(
                        "Unable to find the corresponding announcement",
                    ));
                }
                _ => return Err(CreateRequestError::InternalServerError),
            },
        };

        if (params.action == RequestActionType::ExtendDate
            || params.action == RequestActionType::Delete
            || params.action == RequestActionType::ChangeDevices)
            && announcement.status != AnnouncementStatus::Active
        {
            return Err(CreateRequestError::InvalidAnnouncementStatus(
                "Unable to create request, announcement status must be active",
            ));
        }

        let mut insert_params = InsertRequestParams::new(
            params.action,
            params.description,
            params.announcement_id,
            params.user_id,
        );
        if let Some(extended_end_date) = params.extended_end_date {
            if extended_end_date <= announcement.end_date {
                return Err(CreateRequestError::InvalidExtendedEndDate(
                    "Extended end date must be after the current announcement end date",
                ));
            }

            insert_params = insert_params.extended_end_date(extended_end_date);
        }
        if let Some(new_device_ids) = params.new_device_ids {
            let exists = match self._device_repository.exists(&new_device_ids).await {
                Ok(exists) => exists,
                Err(_) => return Err(CreateRequestError::InternalServerError),
            };
            if !exists {
                return Err(CreateRequestError::InvalidDeviceIds(
                    "Invalid device ids, some of the ids don't exist in the system",
                ));
            }

            insert_params = insert_params.new_device_ids(new_device_ids)
        }

        if let Err(e) = self._request_repository.insert(insert_params).await {
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

        let approval_whitelist: Vec<&str> = vec!["lsc", "bm", "admin"];
        if !approval_whitelist.contains(&approver.role.value) {
            return Err(UpdateRequestApprovalError::UserForbiddenToApprove(
                "User is not allowed to approve a request".into(),
            ));
        }

        let mut approved_by_lsc = request.approved_by_lsc;
        let mut lsc_approver = request.lsc_approver;
        let mut approved_by_bm = request.approved_by_bm;
        let mut bm_approver = request.bm_approver;

        if approver.role.value == "lsc" {
            if request.approved_by_lsc.is_some() {
                return Err(UpdateRequestApprovalError::RequestAlreadyApproved(
                    "Request already approved".into(),
                ));
            }

            approved_by_lsc = Some(params.approval);
            lsc_approver = Some(approver.id);
        } else if approver.role.value == "bm" {
            if request.approved_by_bm.is_some() {
                return Err(UpdateRequestApprovalError::RequestAlreadyApproved(
                    "Request already approved".into(),
                ));
            }

            approved_by_bm = Some(params.approval);
            bm_approver = Some(approver.id);
        } else if approver.role.value == "admin" {
            if request.approved_by_lsc.is_some() && request.approved_by_bm.is_some() {
                return Err(UpdateRequestApprovalError::RequestAlreadyApproved(
                    "Request already approved".into(),
                ));
            }

            if request.approved_by_lsc.is_none() {
                approved_by_lsc = Some(params.approval);
                lsc_approver = Some(approver.id);
            }
            if request.approved_by_bm.is_none() {
                approved_by_bm = Some(params.approval);
                bm_approver = Some(approver.id);
            }
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

        let approval = RequestApproval {
            approved_by_bm,
            approved_by_lsc,
            bm_approver,
            lsc_approver,
        };

        match request.action {
            RequestActionType::Create => {
                self.handle_update_request_approval_create(announcement, request, approval)
                    .await
            }
            RequestActionType::Delete => {
                self.handle_update_request_approval_delete(announcement, request, approval)
                    .await
            }
            RequestActionType::ExtendDate => {
                self.handle_update_request_approval_extend_date(announcement, request, approval)
                    .await
            }
            RequestActionType::ChangeDevices => {
                self.handle_update_request_approval_change_devices(announcement, request, approval)
                    .await
            }
        }
    }

    async fn handle_update_request_approval_create(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError> {
        if announcement.status != AnnouncementStatus::WaitingForApproval {
            return Err(UpdateRequestApprovalError::InvalidAnnouncementStatus(
                "Announcement status should be Waiting for Approval".into(),
            ));
        }

        if approval.approved_by_bm == Some(true) && approval.approved_by_lsc == Some(true) {
            let today = chrono::Utc::today().and_hms(0, 0, 0);
            if today == announcement.start_date {
                let device_ids: Vec<i32> = announcement
                    .devices
                    .into_iter()
                    .map(|device| device.id)
                    .collect();

                if let Err(_) = self
                    ._announcement_repository
                    .update_status(announcement.id, AnnouncementStatus::Active)
                    .await
                {
                    return Err(UpdateRequestApprovalError::InternalServerError);
                }

                if let Err(_) = self
                    ._announcement_queue
                    .create(
                        device_ids,
                        announcement.id,
                        announcement.media_type.to_string(),
                        announcement.media_duration,
                    )
                    .await
                {
                    return Err(UpdateRequestApprovalError::InternalServerError);
                }
            } else {
                if let Err(_) = self
                    ._announcement_repository
                    .update_status(announcement.id, AnnouncementStatus::WaitingForSync)
                    .await
                {
                    return Err(UpdateRequestApprovalError::InternalServerError);
                }
            }
        } else if approval.approved_by_bm == Some(false) || approval.approved_by_lsc == Some(false)
        {
            if let Err(_) = self
                ._announcement_repository
                .update_status(announcement.id, AnnouncementStatus::Rejected)
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }
        }

        if let Err(_) = self
            ._request_repository
            .update_approval(UpdateApprovalParams {
                request_id: request.id,
                approved_by_lsc: approval.approved_by_lsc,
                approved_by_bm: approval.approved_by_bm,
                lsc_approver: approval.lsc_approver,
                bm_approver: approval.bm_approver,
            })
            .await
        {
            return Err(UpdateRequestApprovalError::InternalServerError);
        }

        Ok(())
    }

    async fn handle_update_request_approval_delete(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError> {
        if announcement.status != AnnouncementStatus::Active {
            return Err(UpdateRequestApprovalError::InvalidAnnouncementStatus(
                "Announcement status should be Active".into(),
            ));
        }

        if approval.approved_by_bm == Some(true) && approval.approved_by_lsc == Some(true) {
            let device_ids: Vec<i32> = announcement
                .devices
                .into_iter()
                .map(|device| device.id)
                .collect();

            if let Err(_) = self
                ._announcement_queue
                .delete(device_ids, announcement.id)
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }

            if let Err(_) = self
                ._announcement_repository
                .update_status(announcement.id, AnnouncementStatus::Canceled)
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }

            if let Err(_) = self
                .batch_reject_requests_from_announcement_ids(vec![announcement.id])
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }
        }

        if let Err(_) = self
            ._request_repository
            .update_approval(UpdateApprovalParams {
                request_id: request.id,
                approved_by_lsc: approval.approved_by_lsc,
                approved_by_bm: approval.approved_by_bm,
                lsc_approver: approval.lsc_approver,
                bm_approver: approval.bm_approver,
            })
            .await
        {
            return Err(UpdateRequestApprovalError::InternalServerError);
        }

        Ok(())
    }

    async fn handle_update_request_approval_extend_date(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError> {
        if announcement.status != AnnouncementStatus::Active {
            return Err(UpdateRequestApprovalError::InvalidAnnouncementStatus(
                "Announcement status should be Active".into(),
            ));
        }

        if approval.approved_by_bm == Some(true) && approval.approved_by_lsc == Some(true) {
            if let Err(_) = self
                ._announcement_repository
                .extend_end_date(announcement.id, request.metadata.extended_end_date.unwrap())
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }
        }

        if let Err(_) = self
            ._request_repository
            .update_approval(UpdateApprovalParams {
                request_id: request.id,
                approved_by_lsc: approval.approved_by_lsc,
                approved_by_bm: approval.approved_by_bm,
                lsc_approver: approval.lsc_approver,
                bm_approver: approval.bm_approver,
            })
            .await
        {
            return Err(UpdateRequestApprovalError::InternalServerError);
        }

        Ok(())
    }

    async fn handle_update_request_approval_change_devices(
        &self,
        announcement: AnnouncementDetail,
        request: Request,
        approval: RequestApproval,
    ) -> Result<(), UpdateRequestApprovalError> {
        if announcement.status != AnnouncementStatus::Active {
            return Err(UpdateRequestApprovalError::InvalidAnnouncementStatus(
                "Announcement status should be Active".into(),
            ));
        }

        if approval.approved_by_bm == Some(true) && approval.approved_by_lsc == Some(true) {
            let old_device_ids: Vec<i32> = announcement
                .devices
                .into_iter()
                .map(|device| device.id)
                .collect();

            let new_device_ids = request.metadata.new_device_ids.unwrap();

            let mut need_to_unsync_ids: Vec<i32> = Vec::new();
            for id in &old_device_ids {
                if !new_device_ids.contains(id) {
                    need_to_unsync_ids.push(*id);
                }
            }

            let mut need_to_sync_ids: Vec<i32> = Vec::new();
            for id in &new_device_ids {
                if !old_device_ids.contains(id) {
                    need_to_sync_ids.push(*id);
                }
            }
            if let Err(_) = self
                ._announcement_repository
                .update_announcement_target_devices(
                    announcement.id,
                    need_to_unsync_ids.clone(),
                    need_to_sync_ids.clone(),
                )
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }

            if let Err(_) = self
                ._announcement_queue
                .delete(need_to_unsync_ids.clone(), announcement.id)
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }

            if let Err(_) = self
                ._announcement_queue
                .create(
                    need_to_sync_ids.clone(),
                    announcement.id,
                    announcement.media_type.to_string(),
                    announcement.media_duration,
                )
                .await
            {
                return Err(UpdateRequestApprovalError::InternalServerError);
            }
        }

        if let Err(_) = self
            ._request_repository
            .update_approval(UpdateApprovalParams {
                request_id: request.id,
                approved_by_lsc: approval.approved_by_lsc,
                approved_by_bm: approval.approved_by_bm,
                lsc_approver: approval.lsc_approver,
                bm_approver: approval.bm_approver,
            })
            .await
        {
            return Err(UpdateRequestApprovalError::InternalServerError);
        }

        Ok(())
    }

    async fn batch_reject_requests_from_announcement_ids(
        &self,
        announcement_ids: Vec<i32>,
    ) -> Result<(), BatchRejectRequestsFromAnnouncementIdsError> {
        match self
            ._request_repository
            .batch_reject_requests_from_announcement_ids(announcement_ids)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(BatchRejectRequestsFromAnnouncementIdsError::InternalServerError),
        }
    }
}
