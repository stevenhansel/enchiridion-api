use std::sync::Arc;

use async_trait::async_trait;

use crate::database::{DatabaseError, PaginationResult};

use super::{
    CreateRequestError, FindRequestParams, InsertRequestParams, ListRequestError, Request,
    RequestActionType, RequestRepositoryInterface,
};
/**
* create -> metadata: null
* delete -> metadata: null
* change_date -> tba
* change_content -> tba
* change devices -> tba
*/
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

#[async_trait]
pub trait RequestServiceInterface {
    async fn list_request(
        &self,
        params: ListRequestParams,
    ) -> Result<PaginationResult<Request>, ListRequestError>;
    async fn create_request(&self, params: CreateRequestParams) -> Result<(), CreateRequestError>;

    // TODO: approve/reject
    // async fn update_request_approval() -> Result<(), UpdateRequestApprovalError>;
}

pub struct RequestService {
    _request_repository: Arc<dyn RequestRepositoryInterface + Send + Sync + 'static>,
}

impl RequestService {
    pub fn new(
        _request_repository: Arc<dyn RequestRepositoryInterface + Send + Sync + 'static>,
    ) -> Self {
        RequestService {
            _request_repository,
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
            Err(e) => Err(ListRequestError::InternalServerError),
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
}
