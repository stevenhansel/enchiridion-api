use std::sync::Arc;

use async_trait::async_trait;

use crate::database::DatabaseError;

use super::{
    CreateRequestError, InsertRequestParams, RequestActionType, RequestRepositoryInterface,
};
/**
* create -> metadata: null
* delete -> metadata: null
* change_date -> tba
* change_content -> tba
* change devices -> tba
*/

pub struct CreateRequestParams {
    pub action: RequestActionType,
    pub description: String,
    pub announcement_id: i32,
    pub user_id: i32,
}

#[async_trait]
pub trait RequestServiceInterface {
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
